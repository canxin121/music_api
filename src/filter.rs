use crate::MusicInfo;

pub trait MusicFilter {
    fn matches(&self, info: &MusicInfo) -> bool;
}

#[derive(Debug, Clone)]
pub struct MusicFuzzFilter {
    pub name: Option<String>,
    pub artist: Vec<String>,
    pub album: Option<String>,
}

unsafe impl Send for MusicFuzzFilter {}
unsafe impl Sync for MusicFuzzFilter {}

impl MusicFilter for MusicFuzzFilter {
    fn matches(&self, info: &MusicInfo) -> bool {
        if let Some(name) = &self.name {
            if !fuzzy_match(name, &info.name) {
                return false;
            }
        }
        if let Some(album) = &self.album {
            if let Some(info_album) = &info.album {
                if !fuzzy_match(album, info_album) {
                    return false;
                }
            } else {
                return false;
            }
        }
        // 先to_lowercase再比较
        let self_artist = self
            .artist
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<String>>();
        let info_artist = info
            .artist
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<String>>();

        for artist in &self_artist {
            if !info_artist.contains(&artist) {
                return false;
            }
        }
        true
    }
}

pub fn fuzzy_match(filter: &str, target: &str) -> bool {
    if filter.len() == target.len() {
        filter.to_lowercase() == target.to_lowercase()
    } else if filter.len() > target.len() {
        filter.to_lowercase().contains(&target.to_lowercase())
    } else {
        target.to_lowercase().contains(&filter.to_lowercase())
    }
}
