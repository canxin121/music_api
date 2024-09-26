use std::collections::HashMap;

use sea_orm::{prelude::Expr, Condition, EntityTrait, IntoActiveModel as _, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{music_aggregator, playlist_music_junction},
    server::{kuwo, netease},
};

use super::{
    artist::Artist, database::get_db, quality::Quality, server::MusicServer,
    utils::find_duplicate_music_agg,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Music {
    pub from_db: bool,
    pub server: MusicServer,
    pub identity: String,
    pub name: String,
    pub duration: Option<i64>,
    pub artists: Vec<Artist>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub qualities: Vec<Quality>,
    pub cover: Option<String>,
}

/// Music is subject to foreign key constraints and does not need to be deleted manually.
/// Additionally, Music is not intended to be used directly by external code, but rather through MusicAggregator,
/// so direct access to Music should be restricted.
impl Music {
    /// 允许外部调用更新音乐的功能
    pub async fn update_to_db(&self) -> anyhow::Result<Self> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        match self.server {
            MusicServer::Kuwo => {
                let model: kuwo::model::Model = self.clone().into();
                let mut active = model.into_active_model();
                active.name = Set(self.name.clone());
                active.album = Set(self.album.clone());
                active.album_id = Set(self.album_id.clone());
                active.artists = Set(self.artists.clone().into());
                active.duration = Set(self.duration);
                active.cover = Set(self.cover.clone());

                let model = kuwo::model::Entity::update(active).exec(&db).await?;
                return Ok(model.into_music(true));
            }
            MusicServer::Netease => {
                let model: netease::model::Model = self.clone().into();
                let mut active = model.into_active_model();
                active.name = Set(self.name.clone());
                active.album = Set(self.album.clone());
                active.album_id = Set(self.album_id.clone());
                active.artists = Set(self.artists.clone().into());
                active.duration = Set(self.duration);
                active.cover = Set(self.cover.clone());

                let model = netease::model::Entity::update(active).exec(&db).await?;
                return Ok(model.into_music(true));
            }
        }
    }

    pub async fn insert_to_db(&self) -> anyhow::Result<()> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;

        match self.server {
            MusicServer::Kuwo => {
                let clone = self.clone();
                let model = kuwo::model::Model::from(clone);
                let active = model.into_active_model();
                kuwo::model::Entity::insert(active)
                    .on_conflict_do_nothing()
                    .exec(&db)
                    .await?;
                return Ok(());
            }
            MusicServer::Netease => {
                let clone = self.clone();
                let model = netease::model::Model::from(clone);
                let active = model.into_active_model();
                netease::model::Entity::insert(active)
                    .on_conflict_do_nothing()
                    .exec(&db)
                    .await?;
                return Ok(());
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicAggregator {
    pub name: String,
    pub artist: String,
    pub from_db: bool,
    pub order: Option<i64>,
    pub musics: Vec<Music>,
    pub default_server: MusicServer,
}

impl MusicAggregator {
    pub fn identity(&self) -> String {
        format!("{}#+#{}", self.name, self.artist).to_lowercase()
    }

    pub fn from_music(music: Music) -> Self {
        MusicAggregator {
            name: music.name.clone(),
            artist: {
                let mut artists = music
                    .artists
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<String>>();
                artists.sort();
                artists.join("&")
            },
            from_db: music.from_db,
            default_server: music.server.clone(),
            musics: vec![music],
            order: None,
        }
    }

    pub async fn change_default_server_in_db(
        &self,
        server: MusicServer,
    ) -> Result<(), anyhow::Error> {
        if !self.from_db {
            return Err(anyhow::anyhow!(
                "Can't change default server in non-database music aggregator"
            ));
        }

        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        let agg = music_aggregator::Entity::find_by_id(self.identity())
            .one(&db)
            .await?
            .ok_or(anyhow::anyhow!("Music aggregator not found in db"))?;
        match server {
            MusicServer::Kuwo => {
                if agg.kuwo_music_id.is_none() {
                    return Err(anyhow::anyhow!("No Kuwo music in db"));
                }
            }
            MusicServer::Netease => {
                if agg.netease_music_id.is_none() {
                    return Err(anyhow::anyhow!("No Netease music in db"));
                }
            }
        }

        let mut active = agg.into_active_model();
        active.default_server = Set(server);
        music_aggregator::Entity::update(active).exec(&db).await?;
        Ok(())
    }

    /// Ignores depulicate error, but return the depulicated music_aggregator identity
    pub async fn save_to_db(&self) -> Result<Option<String>, anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;

        // todo: add more music server
        let kuwo_id = self
            .musics
            .iter()
            .find(|x| x.server == MusicServer::Kuwo)
            .and_then(|x| Some(x.identity.clone()));

        let netease_id = self
            .musics
            .iter()
            .find(|x| x.server == MusicServer::Netease)
            .and_then(|x| Some(x.identity.clone()));
        let mut duplicate_identity = None::<String>;

        if let Some(agg) = music_aggregator::Entity::find_by_id(self.identity())
            .one(&db)
            .await?
        {
            let should_update_kuwo = kuwo_id.is_some() && agg.kuwo_music_id.is_none();
            let should_update_netease = netease_id.is_some() && agg.netease_music_id.is_none();
            let mut active: music_aggregator::ActiveModel = agg.into_active_model();
            if should_update_kuwo {
                active.kuwo_music_id = Set(kuwo_id);
            }
            if should_update_netease {
                active.netease_music_id = Set(netease_id);
            }
            music_aggregator::Entity::update(active).exec(&db).await?;
        } else {
            let agg = music_aggregator::ActiveModel {
                identity: Set(self.identity()),
                kuwo_music_id: Set(kuwo_id),
                netease_music_id: Set(netease_id),
                default_server: Set(self
                    .musics
                    .first()
                    .ok_or(anyhow::anyhow!("No music"))?
                    .server
                    .clone()),
            };

            if let Err(e) = music_aggregator::Entity::insert(agg)
                .on_conflict_do_nothing()
                .exec_without_returning(&db)
                .await
            {
                if let sea_orm::DbErr::Exec(sea_orm::RuntimeErr::SqlxError(
                    sea_orm::SqlxError::Database(ref database_error),
                )) = &e
                {
                    let database_error_str = database_error.to_string();
                    // Sqlite Unique error
                    if database_error_str.contains("UNIQUE")
                        // MySql Unique error
                        ||database_error_str.contains("1062")
                        // PgSql Unique error
                        || database_error_str.contains("duplicate")
                    {
                        duplicate_identity = find_duplicate_music_agg(&db, self).await;
                    } else {
                        return Err(anyhow::anyhow!(e));
                    }
                } else {
                    return Err(anyhow::anyhow!(e));
                }
            }
        }

        for music in &self.musics {
            let _ = music.insert_to_db().await;
        }

        Ok(duplicate_identity)
    }

    pub async fn update_order_to_db(&self, playlist_id: i64) -> Result<(), anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        let junction = playlist_music_junction::Entity::find()
            .filter(
                Condition::all()
                    .add(
                        Expr::col(playlist_music_junction::Column::MusicAggregatorId)
                            .eq(self.identity()),
                    )
                    .add(Expr::col(playlist_music_junction::Column::PlaylistId).eq(playlist_id)),
            )
            .one(&db)
            .await?
            .ok_or(anyhow::anyhow!("Music aggregator not found in db"))?;

        let mut active = junction.into_active_model();
        active.order = Set(self.order.ok_or(anyhow::anyhow!("No order"))?);
        playlist_music_junction::Entity::update(active)
            .exec(&db)
            .await?;
        Ok(())
    }

    pub async fn del_from_db(&self) -> Result<(), anyhow::Error> {
        if !self.from_db {
            return Err(anyhow::anyhow!("Can't del non-database music aggregator"));
        }

        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        music_aggregator::Entity::delete_by_id(&self.identity())
            .exec(&db)
            .await?;
        Ok(())
    }

    /// take ownership
    pub async fn search_online(
        aggs: Vec<MusicAggregator>,
        servers: Vec<MusicServer>,
        content: String,
        page: i64,
        size: i64,
    ) -> Result<Vec<Self>, (Vec<Self>, String)> {
        if servers.is_empty() {
            return Err((aggs, "No servers provided".to_string()));
        }

        let mut map = {
            if !aggs.is_empty() {
                let pair = aggs
                    .into_iter()
                    .enumerate()
                    .collect::<Vec<(usize, MusicAggregator)>>();
                let map: HashMap<String, (usize, MusicAggregator)> = pair
                    .into_iter()
                    .map(|pair| (pair.1.identity(), pair))
                    .collect();
                map
            } else {
                HashMap::new()
            }
        };

        let mut success = false;
        if let Some(musics) = Music::search_online(servers, content, page, size)
            .await
            .ok()
        {
            for music in musics {
                let identity = format!("{}#+#{}", music.name, {
                    let mut artists = music
                        .artists
                        .iter()
                        .map(|x| x.name.clone())
                        .collect::<Vec<String>>();
                    artists.sort();
                    artists.join("&")
                })
                .to_lowercase();
                if let Some(pair) = map.get_mut(&identity) {
                    if !pair.1.musics.iter().any(|x| x.server == music.server) {
                        pair.1.musics.push(music);
                    }
                } else {
                    let index = map.len();
                    map.insert(
                        identity.clone(),
                        (
                            index,
                            MusicAggregator {
                                name: music.name.clone(),
                                artist: music
                                    .artists
                                    .iter()
                                    .map(|x| x.name.clone())
                                    .collect::<Vec<String>>()
                                    .join("&"),
                                from_db: false,
                                default_server: music.server.clone(),
                                musics: vec![music],
                                order: None,
                            },
                        ),
                    );
                }
            }
            success = true;
        }

        let mut pairs: Vec<(usize, MusicAggregator)> =
            map.into_iter().map(|(_, pair)| pair).collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let aggs = pairs.into_iter().map(|pair| pair.1).collect();

        if success {
            Ok(aggs)
        } else {
            Err((aggs, "Music search failed".to_string()))
        }
    }

    /// take ownership
    pub async fn fetch_server_online(
        mut self,
        mut servers: Vec<MusicServer>,
    ) -> Result<Self, (Self, String)> {
        servers.retain(|x| !self.musics.iter().any(|y| y.server == *x));

        if servers.is_empty() {
            return Err((self, "No more servers to fetch".to_string()));
        }
        match Music::search_online(servers.clone(), self.identity(), 1, 10).await {
            Ok(musics) => {
                if musics.is_empty() {
                    return Err((self, "No musics found from servers".to_string()));
                }
                for server in servers {
                    if let Some(music) = musics.iter().find(|x| {
                        x.server == server && x.name == self.name && {
                            let mut artists = x
                                .artists
                                .iter()
                                .map(|artist| artist.name.as_str())
                                .collect::<Vec<&str>>();
                            artists.sort();
                            artists.join("&") == self.artist
                        }
                    }) {
                        self.musics.push(music.clone());
                    }
                }
                Ok(self)
            }
            Err(e) => Err((self, format!("Failed to fetch servers: {}", e))),
        }
    }
}

#[cfg(test)]
mod test_music_aggregator {
    use sea_orm_migration::MigratorTrait as _;
    use serial_test::serial;

    use crate::data::{
        interface::{
            database::{get_db, set_db},
            music_aggregator::MusicAggregator,
            playlist::Playlist,
            server::MusicServer,
        },
        migrations::Migrator,
    };

    async fn re_init_db() {
        let _ = tracing_subscriber::fmt::try_init();
        let db_file = "./sample_data/test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();

        set_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
        Migrator::up(&get_db().await.unwrap(), None).await.unwrap();
    }

    pub async fn do_search(aggs: Vec<MusicAggregator>) -> Vec<MusicAggregator> {
        let servers = vec![MusicServer::Kuwo, MusicServer::Netease];
        let content = "米津玄师".to_string();
        let page = 1;
        let size = 5;
        let aggs = MusicAggregator::search_online(aggs, servers, content, page, size)
            .await
            .unwrap();
        aggs
    }

    #[tokio::test]
    pub async fn test_fetch_server_online() {
        let aggs = MusicAggregator::search_online(
            vec![],
            vec![MusicServer::Netease],
            "打上火花".to_string(),
            1,
            30,
        )
        .await
        .unwrap();
        let first = aggs.first().unwrap().clone();
        println!("{:#?}", first);
        let first = first
            .fetch_server_online(vec![MusicServer::Kuwo])
            .await
            .unwrap();
        println!("{:#?}", first);

        let aggs = MusicAggregator::search_online(
            vec![],
            vec![MusicServer::Kuwo],
            "打上火花".to_string(),
            1,
            30,
        )
        .await
        .unwrap();
        let first = aggs.first().unwrap().clone();
        println!("{:#?}", first);
    }

    #[tokio::test]
    #[serial]
    async fn test_save() {
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in aggs {
            agg.save_to_db().await.unwrap();
            println!("{:?}", agg);
        }
        let playlists = Playlist::search_online(
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "米津玄师".to_string(),
            1,
            5,
        )
        .await
        .unwrap();

        for playlist in playlists {
            let aggs = playlist.fetch_musics_online(1, 2333).await.unwrap();
            for agg in aggs {
                agg.save_to_db().await.unwrap();
                println!("{:?}", agg);
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_save_muti() {
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in &aggs {
            agg.save_to_db().await.unwrap();
        }
        let aggs = do_search(vec![]).await;
        for agg in aggs {
            agg.save_to_db().await.unwrap();
            println!("{:?}", agg);
        }
    }
}
