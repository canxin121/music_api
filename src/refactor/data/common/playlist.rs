use serde::{Deserialize, Serialize};

use super::MusicServer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MusicListType {
    UserMusicList,
    Album,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayList {
    pub server: MusicServer,
    #[serde(rename = "type")]
    pub type_field: MusicListType,
    pub identity: String,
    pub name: String,
    pub summary: Option<String>,
    pub cover: Option<String>,
    pub creator: Option<String>,
    pub creator_id: Option<String>,
}
