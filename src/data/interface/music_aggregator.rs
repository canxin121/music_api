use std::collections::HashMap;

use sea_orm::{EntityTrait, IntoActiveModel as _, Set};
use serde::{Deserialize, Serialize};

use crate::{
    data::{get_db, models::music_aggregator},
    server::{kuwo, netease},
};

use super::{artist::Artist, quality::Quality, server::MusicServer, utils::is_artist_equal};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Music {
    pub from_db: bool,
    pub server: MusicServer,
    pub indentity: String,
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
        if !self.from_db {
            return Err(anyhow::anyhow!("Music not from db, can't update"));
        }

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
        if self.from_db {
            return Err(anyhow::anyhow!("Music from db, can't insert"));
        }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicAggregator {
    pub name: String,
    pub artist: String,
    pub from_db: bool,
    pub musics: Vec<Music>,
}

impl MusicAggregator {
    pub fn identity(&self) -> String {
        format!("{}#+#{}", self.name, self.artist)
    }

    pub fn from_music(music: Music) -> Self {
        MusicAggregator {
            name: music.name.clone(),
            artist: music
                .artists
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join("&"),
            from_db: music.from_db,
            musics: vec![music],
        }
    }

    pub async fn get_from_db() -> Result<Vec<Self>, anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        let aggs = music_aggregator::Entity::find().all(&db).await?;
        let mut result = Vec::new();
        for agg in aggs {
            result.push(agg.get_music_aggregator(&db).await?);
        }
        Ok(result)
    }

    pub async fn change_default_server_in_db(
        &mut self,
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

    pub async fn find_in_db(name: String, artist: String) -> Option<Self> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))
            .ok()?;
        let identity = format!("{}#+#{}", name, artist);
        let agg = music_aggregator::Entity::find_by_id(identity)
            .one(&db)
            .await
            .ok()?;

        if let Some(agg) = agg {
            agg.get_music_aggregator(&db).await.ok()
        } else {
            None
        }
    }

    pub async fn insert_to_db(&self) -> Result<(), anyhow::Error> {
        if self.from_db {
            return Err(anyhow::anyhow!(
                "Can't insert music aggregator from db into db."
            ));
        }

        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;

        // todo: add more music server
        let kuwo_id = self
            .musics
            .iter()
            .find(|x| x.server == MusicServer::Kuwo)
            .and_then(|x| Some(x.indentity.clone()));
        let netease_id = self
            .musics
            .iter()
            .find(|x| x.server == MusicServer::Netease)
            .and_then(|x| Some(x.indentity.clone()));

        if let Some(agg) = music_aggregator::Entity::find_by_id(self.identity())
            .one(&db)
            .await?
        {
            let mut active = agg.into_active_model();
            active.kuwo_music_id = Set(kuwo_id);
            active.netease_music_id = Set(netease_id);
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
            music_aggregator::Entity::insert(agg)
                .on_conflict_do_nothing()
                .exec(&db)
                .await?;
        }

        for music in &self.musics {
            music.insert_to_db().await?;
        }
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
        page: u32,
        size: u32,
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
                let identity = format!(
                    "{}#+#{}",
                    music.name,
                    music
                        .artists
                        .iter()
                        .map(|x| x.name.clone())
                        .collect::<Vec<String>>()
                        .join("&")
                );
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
                                musics: vec![music],
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
                        x.server == server
                            && x.name == self.name
                            && is_artist_equal(
                                x.artists
                                    .iter()
                                    .map(|a| a.name.as_str())
                                    .collect::<Vec<&str>>(),
                                self.artist.split('&').collect::<Vec<&str>>(),
                            )
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
        get_db,
        interface::{music_aggregator::MusicAggregator, server::MusicServer},
        migrations::Migrator,
        set_db,
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
    #[serial]
    async fn test_save() {
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in aggs {
            agg.insert_to_db().await.unwrap();
            println!("{:?}", agg);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_save_muti() {
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in &aggs {
            agg.insert_to_db().await.unwrap();
        }
        let aggs = do_search(vec![]).await;
        for agg in aggs {
            agg.insert_to_db().await.unwrap();
            println!("{:?}", agg);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get() {
        let _ = tracing_subscriber::fmt::try_init();
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in &aggs {
            agg.insert_to_db().await.unwrap();
        }

        let inserted_agg = MusicAggregator::get_from_db().await.unwrap();
        for agg in inserted_agg {
            println!("{:?}", agg);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_del() {
        let _ = tracing_subscriber::fmt::try_init();
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in &aggs {
            agg.insert_to_db().await.unwrap();
        }

        let inserted_agg = MusicAggregator::get_from_db().await.unwrap();
        for agg in inserted_agg {
            agg.del_from_db().await.unwrap();
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_fetch() {
        let agg = MusicAggregator {
            name: "Lemon".to_string(),
            artist: "米津玄師".to_string(),
            from_db: false,
            musics: vec![],
        };
        let servers = vec![MusicServer::Kuwo, MusicServer::Netease];
        let agg = agg.fetch_server_online(servers).await.unwrap();
        println!("{:?}", agg);
    }
}
