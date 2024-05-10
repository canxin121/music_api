pub use crate::kuwo::kuwo_search::KuwoSearch;
use sea_query::{InsertStatement, UpdateStatement};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;

pub mod kuwo;
pub mod music_list;
pub mod search_factory;
pub mod sql_store_actory;
pub mod util;

pub use music_list::MusicList;
pub use sqlx::any::install_default_drivers;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quality {
    pub short: String,
    pub level: Option<String>,
    pub bitrate: Option<u32>,
    pub format: Option<String>,
    pub size: Option<String>,
}

impl Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.short)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicInfo {
    pub id: i64,
    pub source: String,
    pub name: String,
    pub artist: Vec<String>,
    pub duration: Option<u32>,
    pub album: Option<String>,
    pub qualities: Vec<Quality>,
    pub default_quality: Option<Quality>,
    pub art_pic: Option<String>,
    pub lyric: Option<String>,
}

impl Display for MusicInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Source: {}, Artist: {}, Duration: {:?}, Album: {}, Qualities: {}, Art Pic: {:?}, HasLyric: {:?}",
            self.name,
            self.source,
            self.artist.join("&"),
            self.duration,
            self.album.as_ref().unwrap_or(&"Unknown".to_string()),
            self.qualities.iter().map(|i|i.to_string()).collect::<Vec<String>>().join(","),
            self.art_pic.as_ref().unwrap_or(&"None".to_string()),
            self.lyric.is_some()
        )
    }
}

pub trait MusicInfoTrait {
    // 常量用于区分音乐源
    fn source(&self) -> &'static str;
    // 此处的id为自定义歌单中的id，这里是借由构造时传入的,只需储存后返回
    fn get_music_id(&self) -> i64;
    // 获取音乐的信息
    fn get_music_info(&self) -> MusicInfo;
    // 获取额外的信息
    fn get_extra_into(&self, quality: &Quality) -> String;
    fn get_album_info(&self) -> Value;
    // unique kv
    fn get_primary_kv(&self) -> (String, String);
}

pub trait MusicTrait: MusicInfoTrait + Send + Store {}

pub type Music = Box<dyn MusicTrait + Send + Sync>;

pub trait SearchTrait {
    fn source_name(&self) -> String;

    fn search_song(
        &self,
        content: &str,
        page: u32,
        limit: u32,
    ) -> impl std::future::Future<Output = Result<Vec<Music>, anyhow::Error>> + Send;

    fn search_album(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Music>, anyhow::Error>> + Send;
}

pub trait Store {
    fn to_json(&self) -> Result<String, anyhow::Error>;
    fn to_sql_insert(&self) -> InsertStatement;
    fn to_sql_update(&self, info: &MusicInfo) -> Result<UpdateStatement, anyhow::Error>;
}
