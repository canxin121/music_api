use std::collections::HashMap;

use sea_orm::{EntityTrait, IntoActiveModel as _};
use serde::{Deserialize, Serialize};

use crate::refactor::{data::get_db, server::kuwo};

use super::{quality::QualityVec, MusicServer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Music {
    pub from_db: bool,
    pub server: MusicServer,
    pub indentity: String,
    pub name: String,
    pub duration: Option<u64>,
    pub artist: String,
    pub artist_id: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub qualities: QualityVec,
    pub music_pic: String,
    pub artist_pic: Option<String>,
    pub album_pic: Option<String>,
}

impl Music{
    pub async fn save(&self) -> Result<(),anyhow::Error> {
        if self.from_db {
            return Ok(());
        }
        let db = get_db().await.ok_or(anyhow::anyhow!("Database is not inited"))?;
        match self.server{
            MusicServer::Kuwo => {
                let clone = self.clone();
                let model = kuwo::model::Model::from(clone);
                let active =  model.into_active_model();
                kuwo::model::Entity::insert(active).exec(&db).await?;
            },
            MusicServer::Netease => todo!(),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicAggregator(pub Vec<Music>);

impl MusicAggregator {
    // 用于标识一个音乐聚合的唯一性
    pub fn indentity(&self) -> String {
        format!(
            "{}-{}",
            self.0.first().unwrap().name,
            self.0.first().unwrap().artist
        )
    }

    // 直接转移所有权是为了ffi时规避rust的生命周期问题
    pub async fn search(
        selfs: Vec<MusicAggregator>,
        servers: Vec<MusicServer>,
        content: String,
        page: u32,
        size: u32,
    ) -> Result<Vec<Self>,Vec<Self>> {
        let mut map = {
            if !selfs.is_empty() {
                let pair = selfs
                    .into_iter()
                    .enumerate()
                    .collect::<Vec<(usize, MusicAggregator)>>();
                let map: HashMap<String, (usize, MusicAggregator)> = pair
                    .into_iter()
                    .map(|pair| (pair.1.indentity(), pair))
                    .collect::<HashMap<String, (usize, MusicAggregator)>>();
                map
            } else {
                HashMap::new()
            }
        };

        let mut success = false;
        if let Some(musics) = Music::search(servers, content, page, size).await.ok() {
            for music in musics {
                let identity = format!("{}-{}", music.name, music.artist);
                if let Some(pair) = map.get_mut(&identity) {
                    if !pair.1 .0.iter().any(|x| x.server == music.server) {
                        pair.1 .0.push(music);
                    }
                } else {
                    // 新的index
                    let index = map.len();
                    map.insert(identity, (index, music.into()));
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
            Err(aggs)
        }
    }

    pub async fn fetch_server(mut self, mut servers: Vec<MusicServer>) -> Self {
        servers.retain(|x| self.0.iter().any(|y| y.server == *x));
        if let Some(music) = self.0.first() {
            let content = format!("{} {}", music.name, music.artist);

            match Music::search(servers, content, 1, 5).await {
                Ok(musics) => {
                    if musics.is_empty() {
                        log::error!("Failed to fetch musics from servers: No musics found",);
                    }
                    self.0.extend(musics);
                    self
                }
                Err(e) => {
                    log::error!("Failed to fetch servers: {}", e);
                    self
                }
            }
        } else {
            self
        }
    }
}

impl Into<MusicAggregator> for Music {
    fn into(self) -> MusicAggregator {
        let mut vec = Vec::with_capacity(MusicServer::length());
        vec.push(self);
        MusicAggregator(vec)
    }
}
