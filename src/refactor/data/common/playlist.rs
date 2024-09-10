use serde::{Deserialize, Serialize};

use super::MusicServer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaylistType {
    UserPlaylist,
    Album,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub server: MusicServer,
    #[serde(rename = "type")]
    pub type_field: PlaylistType,
    pub identity: String,
    pub name: String,
    pub summary: Option<String>,
    pub cover: Option<String>,
    pub creator: Option<String>,
    pub creator_id: Option<String>,
    pub play_time: Option<u128>,
    pub music_num: Option<u128>,
}
