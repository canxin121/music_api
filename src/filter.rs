use strsim::levenshtein;

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
            if levenshtein(&info.name, name) > 2 {
                return false;
            }
        }
        if !self.artist.is_empty() {
            let mut artist_flag = false;
            for artist in &self.artist {
                for artist_info in &info.artist {
                    if levenshtein(artist_info, artist) <= 2 {
                        artist_flag = true;
                        break;
                    }
                }
                if artist_flag {
                    break;
                }
            }
            if !artist_flag {
                return false;
            }
        }
        if let Some(album) = &self.album {
            if let Some(albulm_) = info.album.as_ref() {
                if levenshtein(albulm_, album) > 2 {
                    // Levenshtein distance threshold
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}
