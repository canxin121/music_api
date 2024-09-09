pub mod info;
pub mod lyric;
pub mod music_list;
pub mod search;
pub mod utils;

#[cfg(test)]
mod kuwo_web_api_test {
    use crate::refactor::adapter::kuwo::{
        model::KuwoMusics,
        web_api::{
            info::get_music_info, lyric::get_kuwo_lyric, music_list::get_kuwo_musics_of_music_list,
        },
    };

    use super::{
        music_list::{search_kuwo_music_list, SearchMusiclistResult},
        search::search_kuwo_musics,
    };

    async fn do_search_music() -> KuwoMusics {
        search_kuwo_musics("米津玄师", 1, 100).await.unwrap()
    }

    async fn do_search_music_list() -> SearchMusiclistResult {
        search_kuwo_music_list("米津玄师", 1, 100).await.unwrap()
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
        let info = get_music_info(&music.first().unwrap().musicrid)
            .await
            .unwrap();
        println!("{:?}", info);
    }
}
