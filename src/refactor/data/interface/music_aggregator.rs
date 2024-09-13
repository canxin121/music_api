use std::collections::HashMap;

use sea_orm::{EntityTrait, IntoActiveModel as _, Set};
use serde::{Deserialize, Serialize};

use crate::refactor::{
    data::{get_db, models::music_aggregator},
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

/// Music被添加了外键约束， 无需手动删除
/// 同时不希望Music被外部直接使用，而是使用MusicAggregator，因此不能直接获得Music
impl Music {
    /// 从数据库中获取Music
    // pub async fn get_from_db(servers: Vec<MusicServer>) -> Result<Vec<Self>, anyhow::Error> {
    //     let db = get_db().await.expect("Database is not inited");
    //     let mut musics = Vec::new();
    //     for server in servers {
    //         match server {
    //             MusicServer::Kuwo => {
    //                 let models = kuwo::model::Entity::find().all(&db).await?;
    //                 for model in models {
    //                     musics.push(model.into());
    //                 }
    //             }
    //             MusicServer::Netease => todo!(),
    //             MusicServer::Database => todo!(),
    //         }
    //     }
    //     Ok(musics)
    // }

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
                active.artist = Set(self.artist.clone());
                active.artist_id = Set(self.artist_id.clone());
                active.duration = Set(self.duration);
                active.cover = Set(self.cover.clone());

                let model = kuwo::model::Entity::update(active).exec(&db).await?;
                return Ok(model.into_music(true));
            }
            MusicServer::Netease => todo!(),
        }
    }

    /// 将Music保存到数据库中对应的Server表中
    /// 如果来自数据库，则更新
    /// 如果不来自数据库，则插入
    pub(crate) async fn insert_to_db(&self) -> anyhow::Result<()> {
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
            MusicServer::Netease => todo!(),
        }
    }

    // 从数据库中删除对应的音乐
    // 如果Music不存在， 则不会进行任何操作
    // pub(crate) async fn del_from_db(&self) -> Result<(), anyhow::Error> {
    //     let db = get_db()
    //         .await
    //         .ok_or(anyhow::anyhow!("Database is not inited"))?;
    //     match self.server {
    //         MusicServer::Kuwo => {
    //             if let Err(e) = kuwo::model::Entity::delete_by_id(&self.indentity)
    //                 .exec(&db)
    //                 .await
    //             {
    //                 match e {
    //                     sea_orm::DbErr::RecordNotFound(_) => {}
    //                     other => return Err(anyhow::anyhow!("Failed to delete music: {}", other)),
    //                 }
    //             };
    //         }
    //         MusicServer::Netease => todo!(),
    //         MusicServer::Database => todo!(),
    //     }
    //     Ok(())
    // }
}

// #[cfg(test)]
// mod test_music {
//     use sea_orm_migration::MigratorTrait;

//     use crate::refactor::data::{
//         get_db, init_db,
//         interface::{music_aggregator::Music, MusicServer},
//         migrations::Migrator,
//     };

//     async fn re_init_db() {
//         let db_file = "./test.db";
//         let path = std::path::Path::new(db_file);
//         if path.exists() {
//             std::fs::remove_file(path).unwrap();
//         }
//         std::fs::File::create(path).unwrap();

//         init_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
//         Migrator::up(&get_db().await.unwrap(), None).await.unwrap();
//     }

//     #[tokio::test]
//     async fn test_all() {
//         re_init_db().await;
//         let musics = Music::search(vec![MusicServer::Kuwo], "Aimer".to_string(), 1, 30)
//             .await
//             .unwrap();
//         let len = musics.len();
//         println!("{:?}", musics);
//         let mut saved_musics = Vec::with_capacity(musics.len());
//         for music in musics {
//             let saved_music = music.save_to_db().await.unwrap();
//             saved_musics.push(saved_music);
//         }
//         println!("{:?}", saved_musics);
//         for saved_music in saved_musics {
//             saved_music.save_to_db().await.unwrap();
//         }
//         let musics = Music::get_from_db(vec![MusicServer::Kuwo]).await.unwrap();
//         assert!(musics.len() == len);
//         for music in musics {
//             music.del_from_db().await.unwrap();
//         }
//         assert!(
//             Music::get_from_db(vec![MusicServer::Kuwo])
//                 .await
//                 .unwrap()
//                 .len()
//                 == 0
//         );
//     }
// }

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
        format!("{}#+#{}", self.name, self.artist)
    }

    /// 从一个Music创建一个MusicAggregator
    pub fn from_music(music: Music) -> Self {
        MusicAggregator {
            name: music.name.clone(),
            artist: music.artist.clone(),
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
        // 先保存音乐

        let kuwo_id = self
            .musics
            .iter()
            .find(|x| x.server == MusicServer::Kuwo)
            .and_then(|x| Some(x.indentity.clone()));

        // 维护音乐聚合表
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
            music_aggregator::Entity::insert(agg)
                .on_conflict_do_nothing()
                .exec(&db)
                .await?;
        }

        // 维护分别的音乐表
        for music in &self.musics {
            music.insert_to_db().await?;
        }
        Ok(())
    }

    /// 从数据库中删除
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
                let identity = format!("{}#+#{}", music.name, music.artist);
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
    use serial_test::serial;

    use crate::refactor::data::{
        get_db,
        interface::{music_aggregator::MusicAggregator, MusicServer},
        migrations::Migrator,
        set_db,
    };

    async fn re_init_db() {
        let db_file = "./test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();

        set_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
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
    #[serial]
    pub async fn test_save() {
        re_init_db().await;
        let aggs = do_search(vec![]).await;
        for agg in aggs {
            agg.insert_to_db().await.unwrap();
            println!("{:?}", agg);
        }
    }

    #[tokio::test]
    #[serial]
    pub async fn test_save_muti() {
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
    pub async fn test_get() {
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
    pub async fn test_del() {
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
    pub async fn test_fetch() {
        let agg = MusicAggregator {
            name: "Lemon".to_string(),
            artist: "米津玄師".to_string(),
            from_db: false,
            musics: vec![],
        };
        let servers = vec![MusicServer::Kuwo];
        let agg = agg.fetch_server(servers).await.unwrap();
        println!("{:?}", agg);
    }
}
