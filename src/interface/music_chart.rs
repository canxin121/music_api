use serde::{Deserialize, Serialize};

use super::server::MusicServer;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MusicChart {
    pub name: String,
    pub summary: Option<String>,
    pub cover: Option<String>,
    pub id: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MusicChartCollection {
    pub name: String,
    pub summary: Option<String>,
    pub charts: Vec<MusicChart>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServerMusicChartCollection {
    pub server: MusicServer,
    pub collections: Vec<MusicChartCollection>,
}
