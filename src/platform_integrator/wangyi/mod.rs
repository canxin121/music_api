mod encrypt;
mod request;
mod wy_album;
mod wy_lyric;
mod wy_music;
mod wy_music_detail;
mod wy_musiclist;
mod wy_search;

pub const WANGYI: &'static str = "WangYi";

pub use encrypt::weapi;
pub use wy_album::get_musics_from_album;
pub use wy_music::AlbumPayload;
pub use wy_music::WyMusic;
pub use wy_musiclist::{get_musics_from_music_list, search_music_list};
pub use wy_search::search_single_music;
