use serde::{Deserialize, Serialize};

pub mod music;
pub mod playlist;
pub mod quality;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MusicServer {
    Kuwo,
    Netease,
}

impl MusicServer {
    pub fn length() -> usize {
        2
    }
}
