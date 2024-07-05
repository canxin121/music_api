use std::{fmt::Display, pin::Pin};

use sea_query::{InsertStatement, UpdateStatement};
use serde::{Deserialize, Serialize};

use crate::{music_list::MusicList, MusicAggregator};

pub type Music = Box<dyn MusicTrait + Send + Sync>;

impl Clone for Music {
    fn clone(&self) -> Self {
        self.clone_()
    }
}
impl Display for Music {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_music_info())
    }
}
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
    // 与歌曲/平台本身无关的id，代表的仅仅是其在当前 自定义歌单 中的id
    pub id: i64,
    // 歌曲的来源平台
    pub source: String,
    // 歌曲的名字
    pub name: String,
    // 歌曲的演唱者的集合
    pub artist: Vec<String>,
    // 歌曲的时长(s)
    pub duration: Option<u32>,
    // 歌曲的专辑的名称
    pub album: Option<String>,
    // 歌曲的可选音质
    pub qualities: Vec<Quality>,
    // 歌曲默认选取的音质，可以作为本地持久储存，来为实现每首歌的默认音质均可自定义的功能
    pub default_quality: Option<Quality>,
    // 歌曲的艺术照
    pub art_pic: Option<String>,
    // 歌曲的歌词
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
    fn clone_(&self) -> Music;
    // 常量用于区分音乐源
    fn source(&self) -> String;
    // 获取音乐的信息
    fn get_music_info(&self) -> MusicInfo;
    // 获取额外的信息
    fn get_extra_info(&self, quality: &Quality) -> String;
    // 用于sql储存唯一索引键值，k指储存的列名，v指本歌曲对应列的值
    fn get_primary_kv(&self) -> (String, String);
    // 获取歌词
    fn fetch_lyric(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + Send>>;
    // 获取album和其中的所有歌曲
    fn fetch_album(
        &self,
        page: u32,
        limit: u32,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<(MusicList, Vec<MusicAggregator>), anyhow::Error>,
                > + Send,
        >,
    >;
}

pub trait MusicTrait: MusicInfoTrait + Send + ObjectSafeStore {}

// 符合Object Safe的Store Trait
pub trait ObjectSafeStore {
    fn to_json(&self) -> Result<String, anyhow::Error>;
    // 生成将音乐插入数据库(原始音乐数据表)的Statement
    fn to_sql_insert(&self) -> InsertStatement;
    // 生成更新数据库(原始音乐数据表)中音乐数据的Statement
    fn to_sql_update(
        &self,
        info: &MusicInfo,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<UpdateStatement, anyhow::Error>> + Send + '_>,
    >;
}
