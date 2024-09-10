pub mod album;
pub mod info;
pub mod lyric;
pub mod music;
pub mod music_list;
pub mod utils;

pub use album::get_kuwo_music_album;
pub use info::get_kuwo_music_info;
pub use lyric::get_kuwo_lyric;
pub use music::search_kuwo_musics;
pub use music_list::{get_kuwo_musics_of_music_list, search_kuwo_music_list};

#[cfg(test)]
mod kuwo_web_api_test {
    use crate::refactor::server::kuwo::web_api::{
        album::get_kuwo_music_album, info::get_kuwo_music_info, lyric::get_kuwo_lyric,
        music_list::get_kuwo_musics_of_music_list,
    };

    use super::{
        music::{search_kuwo_musics, KuwoMusics},
        music_list::{search_kuwo_music_list, SearchMusiclistResult},
    };

    async fn do_search_music() -> KuwoMusics {
        search_kuwo_musics("米津玄师", 1, 30).await.unwrap()
    }

    async fn do_search_music_list() -> SearchMusiclistResult {
        search_kuwo_music_list("米津玄师", 1, 30).await.unwrap()
    }

    #[tokio::test]
    async fn test_search_music() {
        let musics = do_search_music().await.abslist;
        println!("length:{}", musics.len());
        for music in musics {
            println!("{:?}", music);
        }
    }

    #[tokio::test]
    async fn test_search_music_list() {
        let result = do_search_music_list().await;
        let musiclists = result.abslist;
        println!("length:{}", musiclists.len());
        for musiclist in musiclists {
            println!("{:?}", musiclist);
        }
    }

    #[tokio::test]
    async fn test_get_musics_of_music_list() {
        let musiclist = do_search_music_list().await.abslist;

        let musicllist1 = musiclist.first().unwrap();
        let result = get_kuwo_musics_of_music_list(&musicllist1.playlistid, 1, 30).await;
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_get_album() {
        let musics = do_search_music().await.abslist;
        let music1 = &musics[2];
        let album_name = music1.album.as_str();
        let album_id = music1.albumid.as_str();
        let result = get_kuwo_music_album(album_id, album_name, 1, 30)
            .await
            .unwrap();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_lyric() {
        let music = do_search_music().await.abslist;
        let lyric = get_kuwo_lyric(&music.first().unwrap().musicrid)
            .await
            .unwrap();
        println!("{}", lyric);
    }

    #[tokio::test]
    async fn test_info() {
        let music = do_search_music().await.abslist;
        let info = get_kuwo_music_info(&music.first().unwrap().musicrid)
            .await
            .unwrap();
        println!("{:?}", info);
    }
}
