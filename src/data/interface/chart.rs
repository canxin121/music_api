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
    pub server: MusicServer,
    pub charts: Vec<MusicChart>,
}
