use anyhow::Result;
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

use super::data::common::playlist::Playlist;
use super::data::common::MusicServer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Music {
    Kuwo(KuwoMusicModel),
}

impl Music {
    pub async fn search(
        target: Option<MusicServer>,
        content: &str,
        page: u32,
        size: u32,
    ) -> Result<Vec<Music>> {
        let kuwo_musics = kuwo::web_api::music::search_kuwo_musics(content, page, size).await?;

        let kuwo_musics = kuwo_musics
            .into_iter()
            .map(|music| Music::Kuwo(music))
            .collect();
        Ok(kuwo_musics)
    }

    pub async fn get_lyric(&self) -> Result<String> {
        match self {
            Music::Kuwo(music) => kuwo::web_api::lyric::get_kuwo_lyric(&music.music_id).await,
        }
    }

    pub async fn get_album(
        &self,
        page: u32,
        limit: u32,
    ) -> Result<(Option<Playlist>, Vec<Music>)> {
        match self {
            Music::Kuwo(music) => {
                if music.album_id.is_none() {
                    return Err(anyhow::anyhow!("Album id is empty."));
                }
                if music.album.is_none() {
                    return Err(anyhow::anyhow!("Album name is empty."));
                }
                let (playlist, musics) = kuwo::web_api::album::get_kuwo_music_album(
                    music.album_id.as_ref().unwrap(),
                    music.album.as_ref().unwrap(),
                    page,
                    limit,
                )
                .await?;
                let musics = musics
                    .into_iter()
                    .map(|music| music.into())
                    .collect();
                Ok((playlist, musics))
            }
        }
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
        match self.server {
            MusicServer::Kuwo => {
                let kuwo_musics = kuwo::web_api::music_list::get_kuwo_musics_of_music_list(
                    &self.identity,
                    page,
                    size,
                )
                .await?;
                let kuwo_musics = kuwo_musics
                    .into_iter()
                    .map(|music|music.into())
                    .collect();
                Ok(kuwo_musics)
            }
            MusicServer::Netease => todo!(),
            MusicServer::Database => todo!(),
        }
    }
}

#[cfg(test)]
mod server_test{
    #[tokio::test]
    async fn test_search_musics(){
        let musics = super::Music::search(Some(super::MusicServer::Kuwo), "周杰伦", 1, 10).await.unwrap();
        assert!(musics.len() > 0);
        for music in musics{
            println!("{:?}", music);
        }
    }
}