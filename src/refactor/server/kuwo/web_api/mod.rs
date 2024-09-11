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
    use crate::refactor::{
        data::common::playlist::Playlist,
        server::{
            kuwo::web_api::{
                album::get_kuwo_music_album, info::get_kuwo_music_info, lyric::get_kuwo_lyric,
                music_list::get_kuwo_musics_of_music_list,
            },
            KuwoMusicModel,
        },
    };

    use super::{music::search_kuwo_musics, music_list::search_kuwo_music_list};

    async fn do_search_music() -> Vec<KuwoMusicModel> {
        search_kuwo_musics("米津玄师", 1, 30).await.unwrap()
    }

    async fn do_search_music_list() -> Vec<Playlist> {
        search_kuwo_music_list("米津玄师", 1, 30).await.unwrap()
    }

    #[tokio::test]
    async fn test_search_music() {
        let musics = do_search_music().await;
        println!("length:{}", musics.len());
        for music in musics {
            println!("{:?}", music);
        }
    }

    #[tokio::test]
    async fn test_search_music_list() {
        let result = do_search_music_list().await;
        let musiclists = result;
        println!("length:{}", musiclists.len());
        for musiclist in musiclists {
            println!("{:?}", musiclist);
        }
    }

    #[tokio::test]
    async fn test_get_musics_of_music_list() {
        let musiclist = do_search_music_list().await;

        let musicllist1 = musiclist.first().unwrap();
        let result = get_kuwo_musics_of_music_list(&musicllist1.identity, 1, 30).await;
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_get_album() {
        let musics = do_search_music().await;
        let music1 = &musics.iter().find(|m| m.album.is_some()).unwrap();
        let album_name = music1.album.as_ref().unwrap();
        let album_id = music1.album_id.as_ref().unwrap();
        let result = get_kuwo_music_album(album_id, album_name, 1, 30)
            .await
            .unwrap();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_lyric() {
        let music = do_search_music().await;
        let lyric = get_kuwo_lyric(&music.first().unwrap().music_id)
            .await
            .unwrap();
        println!("{}", lyric);
    }

    #[tokio::test]
    async fn test_info() {
        let music = do_search_music().await;
        let info = get_kuwo_music_info(&music.first().unwrap().music_id)
            .await
            .unwrap();
        println!("{:?}", info);
    }
}
