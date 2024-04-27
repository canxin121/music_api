pub use crate::kuwo::kuwo_search::KuwoSearch;
use sea_query::{InsertStatement, UpdateStatement};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod kuwo;
pub mod music_list;
pub mod search_factory;
pub mod sql_store_actory;
pub mod util;

pub use music_list::MusicList;
pub use sqlx::any::install_default_drivers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicInfo {
    pub id: i64,
    pub source: String,
    pub name: String,
    pub artist: Vec<String>,
    pub duration: Option<u32>,
    pub album: Option<String>,
    pub qualities: Vec<String>,
    pub default_quality: Option<String>,
    pub art_pic: Option<String>,
    pub lyric: Option<String>,
}

impl Display for MusicInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Source: {}, Artist: {}, Duration: {:?}, Album: {}, Qualities: {:?}, Art Pic: {:?}, HasLyric: {:?}",
            self.name,
            self.source,
            self.artist.join("&"),
            self.duration,
            self.album.as_ref().unwrap_or(&"Unknown".to_string()),
            self.qualities,
            self.art_pic.as_ref().unwrap_or(&"None".to_string()),
            self.lyric.is_some()
        )
    }
}

pub trait MusicInfoTrait {
    fn source(&self) -> &'static str;
    fn get_music_id(&self) -> i64;
    fn get_music_info(&self) -> MusicInfo;
    fn get_extra_into(&self, quality: &str) -> String;
    // unique kv
    fn get_primary_kv(&self) -> (String, String);
}

pub trait MusicTrait: MusicInfoTrait + Send + Store {}

pub type Music = Box<dyn MusicTrait + Send + Sync>;

pub trait SearchTrait {
    fn source_name(&self) -> String;
    fn search(
        &self,
        content: &str,
        page: u32,
        limit: u32,
    ) -> impl std::future::Future<Output = Result<Vec<Music>, anyhow::Error>> + Send;
}

pub trait Store {
    fn to_json(&self) -> Result<String, anyhow::Error>;
    fn to_sql_insert(&self) -> InsertStatement;
    fn to_sql_update(&self, info: &MusicInfo) -> Result<UpdateStatement, anyhow::Error>;
}
