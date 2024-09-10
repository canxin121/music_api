use serde::{Deserialize, Serialize};

pub mod playlist;
pub mod playlist_subscription;
pub mod quality;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MusicServer {
    Kuwo,
    Netease,
}
