use serde::{Deserialize, Serialize};

use super::{quality::QualityVec, MusicServer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Music {
    pub server: MusicServer,
    pub indentity: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub qualities: QualityVec,
    pub music_pic: String,
    pub artist_pic: Option<String>,
    pub album_pic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicAggregator(pub Vec<Music>);
