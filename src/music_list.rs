use crate::{
    factory::sql_factory::{ID, REFARTPIC, REFDESC, REFNAME},
    music_aggregator::MusicAggregator,
};
use futures::Future;
use sqlx::{any::AnyRow, Row};
use std::{fmt::Display, pin::Pin};

pub type MusicList = Box<dyn MusicListTrait + Send + Sync>;

pub trait MusicListTrait {
    fn source(&self) -> String;
    fn get_musiclist_info(&self) -> MusicListInfo;
    fn get_music_aggregators(
        &self,
        page: u32,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<MusicAggregator>, anyhow::Error>> + Send + '_>>;
}

impl Display for MusicList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_musiclist_info())
    }
}

#[derive(Debug, Clone)]
pub struct MusicListInfo {
    pub name: String,
    pub art_pic: String,
    pub desc: String,
    pub extra: Option<ExtraInfo>,
    pub id: i64,
}

#[derive(Debug, Clone)]
pub struct ExtraInfo {
    pub play_count: Option<u32>,
    pub music_count: Option<u32>,
}

impl MusicListInfo {
    pub fn from_row(row: AnyRow) -> Result<MusicListInfo, anyhow::Error> {
        Ok(MusicListInfo {
            id: row.try_get(ID.0)?,
            name: row.try_get(REFNAME.0).unwrap_or("获取失败".to_string()),
            art_pic: row.try_get(REFARTPIC.0).unwrap_or_default(),
            desc: row.try_get(REFDESC.0).unwrap_or("获取失败".to_string()),
            extra: None,
        })
    }
}

impl Display for MusicListInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}--({})--[{}]", self.name, self.desc, self.art_pic)
    }
}
