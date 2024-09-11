/// 需要确保所有interface能够轻松的被序列化和反序列化， ffi要规避rust的生命周期问题
/// 确保数据结构足够简单，不要涉及
///
use serde::{Deserialize, Serialize};

pub mod music_aggregator;
pub mod playlist;
pub mod quality;
pub mod utils;

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MusicServer {
    Kuwo,
    Netease,
    #[default]
    Database,
}

impl MusicServer {
    pub fn length() -> usize {
        2
    }
}
