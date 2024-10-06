use serde::{Deserialize, Serialize};

use super::server::MusicServer;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlaylistTag {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlaylistTagCollection {
    pub name: String,
    pub tags: Vec<PlaylistTag>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServerPlaylistTagCollection {
    pub server: MusicServer,
    pub collections: Vec<PlaylistTagCollection>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TagPlaylistOrder {
    Hot,
    New,
}
