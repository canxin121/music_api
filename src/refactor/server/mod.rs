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
                            Ok(musics) => musics.into_iter().map(|music| music.into()).collect(),
                            Err(e) => {
                                log::error!("Failed to search kuwo musics: {}", e);
                                Vec::new()
                            }
                        }
                    }));
                }
                MusicServer::Netease => todo!(),
                MusicServer::Database => todo!(),
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
                let musics = musics.into_iter().map(|music| music.into()).collect();
                Ok((album, musics))
            }
            MusicServer::Netease => todo!(),
            MusicServer::Database => todo!(),
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
                MusicServer::Database => todo!(),
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

    pub async fn get_musics(&self, page: u32, size: u32) -> Result<Vec<Music>> {
        // 返回的一定是单个server的音乐
        // 因此可以直接构建新的MusicAggregator
        match self.server {
            MusicServer::Kuwo => {
                let kuwo_musics = kuwo::web_api::music_list::get_kuwo_musics_of_music_list(
                    &self.identity,
                    page,
                    size,
                )
                .await?;
                let kuwo_musics = kuwo_musics.into_iter().map(|music| music.into()).collect();
                Ok(kuwo_musics)
            }
            MusicServer::Netease => todo!(),
            MusicServer::Database => todo!(),
        }
    }
}

#[cfg(test)]
mod server_test {
    #[tokio::test]
    async fn test_search() {
        let playlists =
            super::Playlist::search(vec![super::MusicServer::Kuwo], "周杰伦".to_string(), 1, 10)
                .await
                .unwrap();
        println!("{:?}", playlists);
    }
    #[tokio::test]
    async fn test_get_musics() {
        let playlist =
            super::Playlist::search(vec![super::MusicServer::Kuwo], "周杰伦".to_string(), 1, 10)
                .await
                .unwrap()
                .pop()
                .unwrap();
        let musics = playlist.get_musics(1, 10).await.unwrap();
        println!("{:?}", musics);
    }
}
