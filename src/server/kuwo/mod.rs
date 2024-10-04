pub mod create_kuwo_music_table_migration;
pub mod model;
pub mod web_api;

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_quality() {
        let musics = super::web_api::music::search_kuwo_musics("周杰伦", 1, 30)
            .await
            .unwrap();
        let first = musics.first().unwrap();
        println!("{:?}", first);
    }
}
