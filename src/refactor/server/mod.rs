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

use super::data::interface::music::Music;
use super::data::interface::playlist::Playlist;
use super::data::interface::MusicServer;

impl Music {
    pub async fn search(
        servers: Vec<MusicServer>,
        content: String,
        page: u32,
        size: u32,
    ) -> Result<Vec<Music>> {
        let mut handles:Vec<tokio::task::JoinHandle<Vec<Music>>> = Vec::with_capacity(MusicServer::length());
        let content = Arc::new(content);
        for server in servers {
            let content = Arc::clone(&content);
            match server {
                MusicServer::Kuwo => {
                    handles.push(tokio::spawn(async move {
                        match kuwo::web_api::music::search_kuwo_musics(content.as_ref(), page, size).await {
                            Ok(musics) => musics.into_iter().map(|music| music.into()).collect(),
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
}

impl Playlist {
    pub async fn search(
        target: Option<MusicServer>,
        content: &str,
        page: u32,
        size: u32,
    ) -> Result<Vec<Playlist>> {
        let kuwo_playlists =
            kuwo::web_api::music_list::search_kuwo_music_list(content, page, size).await?;
        Ok(kuwo_playlists)
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
        }
    }
}

#[cfg(test)]
mod server_test {}
