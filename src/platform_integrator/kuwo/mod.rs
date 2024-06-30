pub mod kuwo_album;
pub mod kuwo_lyric;
pub mod kuwo_music;
pub mod kuwo_music_info;
pub mod kuwo_music_list;
pub mod kuwo_pic;
pub mod kuwo_quality;
pub mod kuwo_search;
pub mod util;

pub const KUWO_KEY: &'static str = "Musicrid";
pub const KUWO: &'static str = "KuWo";
pub use kuwo_music_list::get_musics_of_music_list;
pub use kuwo_music_list::search_music_list;
pub use kuwo_search::search_single_music;
