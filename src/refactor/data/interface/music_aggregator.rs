use std::collections::HashMap;

use sea_orm::{EntityTrait, IntoActiveModel as _, ModelTrait, Set};
use serde::{Deserialize, Serialize};

use crate::refactor::{
    data::{
        get_db,
        models::{music_aggregator, playlist},
    },
    server::kuwo,
};

use super::{quality::QualityVec, utils::is_artist_equal, MusicServer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Music {
    pub from_db: bool,
    pub server: MusicServer,
    pub indentity: String,
    pub name: String,
    pub duration: Option<i64>,
    pub artist: String,
    pub artist_id: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub qualities: QualityVec,
    pub cover: Option<String>,
}

impl Music {
    /// 从数据库中获取Music
    pub async fn get_from_db(servers: Vec<MusicServer>) -> Result<Vec<Self>, anyhow::Error> {
        let db = get_db().await.expect("Database is not inited");
        let mut musics = Vec::new();
        for server in servers {
            match server {
                MusicServer::Kuwo => {
                    let models = kuwo::model::Entity::find().all(&db).await?;
                    for model in models {
                        musics.push(model.into());
                    }
                }
                MusicServer::Netease => todo!(),
                MusicServer::Database => todo!(),
            }
        }
        Ok(musics)
    }

    /// 将Music保存到数据库中对应的Server表中
    /// 如果来自数据库，则更新
    /// 如果不来自数据库，则插入
    pub async fn save_to_db(&self) -> Result<Self, anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;

        match self.server {
            MusicServer::Kuwo => {
                if self.from_db {
                    let model = kuwo::model::Entity::find_by_id(&self.indentity)
                        .one(&db)
                        .await?
                        .ok_or(anyhow::anyhow!("Music from db, but not found"))?;
                    let mut active = model.into_active_model();
                    active.album = Set(self.album.clone());
                    active.album_id = Set(self.album_id.clone());
                    active.artist = Set(self.artist.clone());
                    active.artist_id = Set(self.artist_id.clone());
                    active.duration = Set(self.duration);
                    active.cover = Set(self.cover.clone());
                    // active.artist_pic = Set(self.artist_pic.clone());
                    // active.album_pic = Set(self.album_pic.clone());
                    let model = kuwo::model::Entity::update(active).exec(&db).await?;
                    return Ok(model.into());
                } else {
                    let model = kuwo::model::Entity::find_by_id(&self.indentity)
                        .one(&db)
                        .await?;
                    if model.is_some() {
                        return Ok(self.clone());
                    }
                    let clone = self.clone();
                    let music_id = clone.indentity.clone();
                    let model = kuwo::model::Model::from(clone);
                    let active = model.into_active_model();
                    kuwo::model::Entity::insert(active).exec(&db).await?;
                    let model = kuwo::model::Entity::find_by_id(music_id)
                        .one(&db)
                        .await?
                        .ok_or(anyhow::anyhow!("Failed to find music after insert"))?;
                    return Ok(model.into());
                }
            }
            MusicServer::Netease => todo!(),
            MusicServer::Database => todo!(),
        }
    }

    /// 从数据库中删除对应的音乐
    /// 如果Music不存在， 则不会进行任何操作
    pub async fn del_from_db(&self) -> Result<(), anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        match self.server {
            MusicServer::Kuwo => {
                if let Err(e) = kuwo::model::Entity::delete_by_id(&self.indentity)
                    .exec(&db)
                    .await
                {
                    match e {
                        sea_orm::DbErr::RecordNotFound(_) => {}
                        other => return Err(anyhow::anyhow!("Failed to delete music: {}", other)),
                    }
                };
            }
            MusicServer::Netease => todo!(),
            MusicServer::Database => todo!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_music {
    use sea_orm_migration::MigratorTrait;

    use crate::refactor::data::{
        get_db, init_db,
        interface::{music_aggregator::Music, MusicServer},
        migrations::Migrator,
    };

    async fn re_init_db() {
        let db_file = "./test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();

        init_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
        Migrator::up(&get_db().await.unwrap(), None).await.unwrap();
    }

    #[tokio::test]
    async fn test_all() {
        re_init_db().await;
        let musics = Music::search(vec![MusicServer::Kuwo], "Aimer".to_string(), 1, 30)
            .await
            .unwrap();
        let len = musics.len();
        println!("{:?}", musics);
        let mut saved_musics = Vec::with_capacity(musics.len());
        for music in musics {
            let saved_music = music.save_to_db().await.unwrap();
            saved_musics.push(saved_music);
        }
        println!("{:?}", saved_musics);
        for saved_music in saved_musics {
            saved_music.save_to_db().await.unwrap();
        }
        let musics = Music::get_from_db(vec![MusicServer::Kuwo]).await.unwrap();
        assert!(musics.len() == len);
        for music in musics {
            music.del_from_db().await.unwrap();
        }
        assert!(
            Music::get_from_db(vec![MusicServer::Kuwo])
                .await
                .unwrap()
                .len()
                == 0
        );
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
    /// identity是name和artist的组合, 是音乐聚合的唯一标识， 也是数据库中的主键
    pub fn identity(&self) -> String {
        format!("{} {}", self.name, self.artist)
    }

    /// 多个artist应该使用&连接
    pub async fn new(name: String, artist: String) -> Self {
        MusicAggregator {
            name,
            artist,
            from_db: false,
            musics: Vec::new(),
        }
    }

    /// 从一个Music创建一个MusicAggregator
    pub async fn from_music(music: Music) -> Self {
        MusicAggregator {
            name: music.name.clone(),
            artist: music.artist.clone(),
            from_db: music.from_db,
            musics: vec![music],
        }
    }

    /// 从数据库中获取MusicAggregator
    pub async fn get_from_db(playlist_id: Option<i64>) -> Result<Vec<Self>, anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        let aggs;
        if let Some(playlist_id) = playlist_id {
            let playlist = playlist::Entity::find_by_id(playlist_id)
                .one(&db)
                .await?
                .ok_or(anyhow::anyhow!("Playlist not found"))?;

            aggs = playlist
                .find_related(music_aggregator::Entity)
                .all(&db)
                .await?;
        } else {
            aggs = music_aggregator::Entity::find().all(&db).await?;
        }

        let mut musics: Vec<MusicAggregator> = Vec::with_capacity(aggs.len());

        for agg in aggs {
            let mut vec = Vec::with_capacity(MusicServer::length());
            /// TODO: 完成其他server的查询
            if let Some(model) = agg.find_related(kuwo::model::Entity).one(&db).await? {
                vec.push(model.into());
            }
            if !vec.is_empty() {
                let (name, artist) = agg.identity.split_once(' ').ok_or(anyhow::anyhow!(
                    "Failed to split name and artist from identity: {}",
                    agg.identity
                ))?;

                musics.push(MusicAggregator {
                    name: name.to_string(),
                    artist: artist.to_string(),
                    from_db: true,
                    musics: vec,
                });
            }
        }
        Ok(musics)
    }

    /// 保存到数据库中(有则更新，无则插入)
    /// 更新时仅考虑维护 junction table， 而不会修改music表中信息
    /// 如需修改music表中信息， 请使用Music的save_to_db方法
    pub async fn save_to_db(&self) -> Result<(), anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        // 先保存音乐

        /// TODO: 完成其他server的id的保存
        let kuwo_id = self
            .musics
            .iter()
            .find(|x| x.server == MusicServer::Kuwo)
            .and_then(|x| Some(x.indentity.clone()));

        // 如果有重复的id则更新
        if let Some(agg) = music_aggregator::Entity::find_by_id(self.identity())
            .one(&db)
            .await?
        {
            let mut active = agg.into_active_model();
            active.kuwo_music_id = Set(kuwo_id);
            music_aggregator::Entity::update(active).exec(&db).await?;
        // 否则插入
        } else {
            let agg = music_aggregator::ActiveModel {
                identity: Set(self.identity()),
                kuwo_music_id: Set(kuwo_id),
                netease_music_id: Set(None),
            };
            music_aggregator::Entity::insert(agg).exec(&db).await?;
        }

        for music in &self.musics {
            music.save_to_db().await?;
        }
        Ok(())
    }

    /// 从数据库中删除
    pub async fn del_from_db(&self) -> Result<(), anyhow::Error> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited"))?;
        music_aggregator::Entity::delete_by_id(&self.identity())
            .exec(&db)
            .await?;
        Ok(())
    }

    /// 直接转移所有权是为了ffi时规避rust的生命周期问题
    /// 搜索音乐聚合
    pub async fn search(
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
        if let Some(musics) = Music::search(servers, content, page, size).await.ok() {
            for music in musics {
                let identity = format!("{} {}", music.name, music.artist);
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
                                artist: music.artist.clone(),
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

    pub async fn fetch_server(
        mut self,
        mut servers: Vec<MusicServer>,
    ) -> Result<Self, (Self, String)> {
        servers.retain(|x| !self.musics.iter().any(|y| y.server == *x));

        if servers.is_empty() {
            return Err((self, "No more servers to fetch".to_string()));
        }
        match Music::search(servers.clone(), self.identity(), 1, 10).await {
            Ok(musics) => {
                if musics.is_empty() {
                    return Err((self, "No musics found from servers".to_string()));
                }
                for server in servers {
                    if let Some(music) = musics.iter().find(|x| {
                        x.server == server
                            && x.name == self.name
                            && is_artist_equal(&x.artist, &self.artist)
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

    use crate::refactor::data::{
        get_db, init_db,
        interface::{music_aggregator::MusicAggregator, MusicServer},
        migrations::Migrator,
    };

    async fn re_init_db() {
        let db_file = "./test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();

        init_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
        Migrator::up(&get_db().await.unwrap(), None).await.unwrap();
    }

    pub async fn do_search(aggs: Vec<MusicAggregator>) -> Vec<MusicAggregator> {
        let servers = vec![MusicServer::Kuwo];
        let content = "米津玄师".to_string();
        let page = 1;
        let size = 5;
        let aggs = MusicAggregator::search(aggs, servers, content, page, size)
            .await
            .unwrap();
        aggs
    }

    #[tokio::test]
    pub async fn test_save() {
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in aggs {
            agg.save_to_db().await.unwrap();
            println!("{:?}", agg);
        }
    }

    #[tokio::test]
    pub async fn test_save_muti() {
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

    #[tokio::test]
    pub async fn test_del() {
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in &aggs {
            agg.save_to_db().await.unwrap();
        }

        for agg in aggs {
            agg.del_from_db().await.unwrap();
        }
    }

    #[tokio::test]
    pub async fn test_fetch() {
        let agg = MusicAggregator::new("Lemon".to_string(), "米津玄師".to_string()).await;
        let servers = vec![MusicServer::Kuwo];
        let agg = agg.fetch_server(servers).await.unwrap();
        println!("{:?}", agg);
    }
}
