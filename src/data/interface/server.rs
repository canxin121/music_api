use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MusicServer {
    Kuwo,
    Netease,
}

impl MusicServer {
    pub fn length() -> usize {
        // todo: add more music server
        2
    }
}
