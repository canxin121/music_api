pub mod model;
pub mod web_api;

#[cfg(test)]
mod kuwo_test {
    use crate::refactor::adapter::kuwo::web_api::lyric::get_kuwo_lyric;

    use super::{model::KuwoMusics, web_api::search::search_kuwo_musics};

    async fn do_search() -> KuwoMusics {
        search_kuwo_musics("米津玄师", 1, 100).await.unwrap()
    }

    #[tokio::test]
    async fn test_search() {
        let musics = do_search().await.abslist;
        for music in musics {
            println!("{:?}", music);
        }
    }

    #[tokio::test]
    async fn test_lyric() {
        let music = do_search().await.abslist;
        let lyric = get_kuwo_lyric(&music.first().unwrap().musicrid)
            .await
            .unwrap();
        println!("{}", lyric);
    }
}
