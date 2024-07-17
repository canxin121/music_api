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
        if !self.artist.is_empty() && self.artist != info.artist {
            return false;
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
        true
    }
}

fn fuzzy_match(filter: &str, target: &str) -> bool {
    if filter.len() == target.len() {
        filter == target
    } else if filter.len() > target.len() {
        filter.contains(target)
    } else {
        target.contains(filter)
    }
}
