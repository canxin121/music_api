use anyhow::Result;
use std::sync::Arc;
use std::sync::LazyLock;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

pub mod kuwo;

pub static CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    ClientBuilder::new(
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap(),
    )
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(2),
    ))
    .build()
});

pub use kuwo::model::ActiveModel as KuwoMusicActiveModel;
pub use kuwo::model::Model as KuwoMusicModel;

use super::data::interface::music_aggregator::Music;
use super::data::interface::music_aggregator::MusicAggregator;
use super::data::interface::playlist::Playlist;
use super::data::interface::MusicServer;

impl Music {
    pub async fn search(
        servers: Vec<MusicServer>,
        content: String,
        page: u32,
        size: u32,
    ) -> Result<Vec<Music>> {
        let mut handles: Vec<tokio::task::JoinHandle<Vec<Music>>> =
            Vec::with_capacity(MusicServer::length());
        let content = Arc::new(content);
        for server in servers {
            let content = Arc::clone(&content);
            match server {
                MusicServer::Kuwo => {
                    handles.push(tokio::spawn(async move {
                        match kuwo::web_api::music::search_kuwo_musics(content.as_ref(), page, size)
                            .await
                        {
                            Ok(musics) => musics
                                .into_iter()
                                .map(|music| music.into_music(false))
                                .collect(),
                            Err(e) => {
                                log::error!("Failed to search kuwo musics: {}", e);
                                Vec::new()
                            }
                        }
                    }));
                }
                MusicServer::Netease => todo!(),
            }
        }
        let mut musics = Vec::with_capacity(MusicServer::length());
        for handle in handles {
            musics.extend(handle.await?);
        }
        Ok(musics)
    }

    // 由于专辑歌曲较少，有的平台不分页，因此第一次就返回 第一页 的歌曲(可能就是全部)
    pub async fn get_album(&self, page: u32, limit: u32) -> Result<(Option<Playlist>, Vec<Music>)> {
        match self.server {
            MusicServer::Kuwo => {
                let (album, musics) = kuwo::web_api::album::get_kuwo_music_album(
                    self.album_id
                        .as_ref()
                        .ok_or(anyhow::anyhow!("No album id"))?,
                    self.album
                        .as_ref()
                        .ok_or(anyhow::anyhow!("No album name"))?,
                    page,
                    limit,
                )
                .await?;
                let musics = musics
                    .into_iter()
                    .map(|music| music.into_music(false))
                    .collect();
                Ok((album, musics))
            }
            MusicServer::Netease => todo!(),
        }
    }
}

#[cfg(test)]
mod server_music_test {
    use super::super::data::interface::MusicServer;
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let musics = Music::search(vec![MusicServer::Kuwo], "Lemon 米津玄师".to_string(), 1, 10)
            .await
            .unwrap();
        println!("{:?}", musics);
    }
}

pub async fn fetch_musics(
    server: &MusicServer,
    identity: &str,
    page: u32,
    size: u32,
) -> Result<Vec<MusicAggregator>> {
    match server {
        MusicServer::Kuwo => {
            let kuwo_musics =
                kuwo::web_api::music_list::get_kuwo_musics_of_music_list(identity, page, size)
                    .await?;
            let kuwo_musics: Vec<Music> = kuwo_musics
                .into_iter()
                .map(|music| music.into_music(false))
                .collect();

            Ok(kuwo_musics
                .into_iter()
                .map(|music| MusicAggregator::from_music(music))
                .collect::<Vec<MusicAggregator>>())
        }
        MusicServer::Netease => todo!(),
    }
}

// async fn fetch_musics_retry(
//     server: &MusicServer,
//     identity: &str,
//     page: u32,
//     size: u32,
//     retries: usize,
// ) -> Result<Vec<MusicAggregator>> {
//     for attempt in 0..=retries {
//         match fetch_musics(server, identity, page, size).await {
//             Ok(musics) => return Ok(musics),
//             Err(e) if attempt < retries => {
//                 eprintln!("Retrying page {} due to error: {:?}", page, e);
//             }
//             Err(e) => return Err(e),
//         }
//     }
//     Err(anyhow::anyhow!("Failed after {} retries", retries))
// }

// async fn fetch_musics_concurrent(
//     server: Arc<MusicServer>,
//     identity: Arc<String>,
//     page: u32,
//     size: u32,
//     semaphore: Arc<Semaphore>,
//     retries: usize,
// ) -> Result<Vec<MusicAggregator>> {
//     // Acquire the semaphore to limit concurrent requests
//     let _permit = semaphore.acquire().await.unwrap();
//     fetch_musics_retry(&server, &identity, page, size, retries).await
// }

impl Playlist {
    pub async fn search(
        servers: Vec<MusicServer>,
        content: String,
        page: u32,
        size: u32,
    ) -> Result<Vec<Playlist>> {
        if servers.is_empty() {
            return Err(anyhow::anyhow!("No server specified"));
        }
        let mut handles: Vec<tokio::task::JoinHandle<Result<Vec<Playlist>>>> =
            Vec::with_capacity(MusicServer::length());
        let content = Arc::new(content);
        for server in servers {
            let content = Arc::clone(&content);
            match server {
                MusicServer::Kuwo => {
                    handles.push(tokio::spawn(async move {
                        kuwo::web_api::music_list::search_kuwo_music_list(
                            content.as_str(),
                            page,
                            size,
                        )
                        .await
                    }));
                }
                MusicServer::Netease => todo!(),
            }
        }
        let mut playlists = Vec::with_capacity(MusicServer::length());
        for handle in handles {
            match handle.await? {
                Ok(mut ps) => playlists.append(&mut ps),
                Err(e) => log::error!("Failed to search playlist: {}", e),
            }
        }
        Ok(playlists)
    }

    pub async fn fetch_musics(&self, page: u32, size: u32) -> Result<Vec<MusicAggregator>> {
        // 返回的一定是单个server的音乐
        // 因此可以直接构建新的MusicAggregator
        if self.from_db || self.server.is_none() {
            return Err(anyhow::anyhow!("Cant't get music from db playlist"));
        }

        let server = self.server.as_ref().ok_or(anyhow::anyhow!(
            "This music is not from db, but has no server."
        ))?;
        fetch_musics(server, self.identity.as_str(), page, size).await
    }

    // pub async fn fetch_all_musics(&self) -> Result<Vec<MusicAggregator>> {
    //     let server = Arc::new(self.server.clone().ok_or(anyhow::anyhow!(
    //         "This music is not from db, but has no server."
    //     ))?);
    //     let identity = Arc::new(self.identity.clone());

    //     let mut musics = Vec::new();
    //     let mut page = 1;
    //     let size = 100;
    //     let semaphore = Arc::new(Semaphore::new(10)); // Limit concurrency to 10 pages at a time
    //     let mut tasks = Vec::new();

    //     loop {
    //         let fetch_task = fetch_musics_concurrent(
    //             server.clone(),
    //             identity.clone(),
    //             page,
    //             size,
    //             semaphore.clone(),
    //             3,
    //         );
    //         tasks.push(fetch_task);

    //         if tasks.len() == 10 {
    //             let results = join_all(tasks).await;
    //             for result in results {
    //                 match result {
    //                     Ok(new_musics) if new_musics.is_empty() => return Ok(musics), // No more pages
    //                     Ok(new_musics) => musics.extend(new_musics),
    //                     Err(e) => eprintln!("Error fetching music: {:?}", e),
    //                 }
    //             }
    //             tasks = Vec::new();
    //         }

    //         page += 1;
    //     }
    // }
}

#[cfg(test)]
mod server_test {
    #[tokio::test]
    async fn test_search() {
        let playlists =
            super::Playlist::search(vec![super::MusicServer::Kuwo], "周杰伦".to_string(), 8, 10)
                .await
                .unwrap();
        assert!(playlists.len() <= 10 && playlists.len() > 0);

        let playlists = super::Playlist::search(
            vec![super::MusicServer::Kuwo],
            "周杰伦".to_string(),
            9999,
            10,
        )
        .await
        .unwrap();
        assert!(playlists.len() == 0)
    }

    #[tokio::test]
    async fn test_fetch_musics() {
        let playlists =
            super::Playlist::search(vec![super::MusicServer::Kuwo], "周杰伦".to_string(), 1, 10)
                .await
                .unwrap();
        let playlist = playlists.first().unwrap();
        let musics = playlist.fetch_musics(1, 10).await.unwrap();
        assert!(musics.len() > 0 && musics.len() <= 10);
        let musics = playlist.fetch_musics(999, 10).await.unwrap();
        assert!(musics.len() == 0)
    }

    #[tokio::test]
    async fn test_fetch_all_musics() {
        let playlists =
            super::Playlist::search(vec![super::MusicServer::Kuwo], "周杰伦".to_string(), 1, 10)
                .await
                .unwrap();
        let playlist = playlists.first().unwrap();
        let start = std::time::Instant::now();
        let musics = playlist.fetch_musics(1, 999).await.unwrap();

        println!("{:?}", musics);
        println!("Time: {:?}", start.elapsed());
        println!("Length: {}", musics.len());
        assert!(musics.len() > 0);
    }
}
