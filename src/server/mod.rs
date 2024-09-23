use anyhow::Result;
use kuwo::web_api::share_playlist::get_kuwo_music_list_from_share;
use netease::web_api::share_playlist::get_netease_music_list_from_share;
use std::sync::Arc;

pub(crate) mod kuwo;
pub(crate) mod netease;

use crate::data::interface::server::MusicServer;

use super::data::interface::music_aggregator::Music;
use super::data::interface::music_aggregator::MusicAggregator;
use super::data::interface::playlist::Playlist;

impl Music {
    /// Search music online
    pub async fn search_online(
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
                                Vec::with_capacity(0)
                            }
                        }
                    }));
                }
                MusicServer::Netease => handles.push(tokio::spawn(async move {
                    match netease::web_api::music::search_netease_music(
                        content.as_ref(),
                        page as u16,
                        size as u16,
                    )
                    .await
                    {
                        Ok(musics) => musics
                            .into_iter()
                            .map(|music| music.into_music(false))
                            .collect(),
                        Err(e) => {
                            log::error!("Failed to search netease musics: {}", e);
                            Vec::with_capacity(0)
                        }
                    }
                })),
            }
        }
        let mut musics = Vec::with_capacity(MusicServer::length());
        for handle in handles {
            musics.extend(handle.await?);
        }
        Ok(musics)
    }

    /// return the album playlist on first page, and musics on each page
    /// on some music server, the page and limit has no effect, they just return the all musics.
    pub async fn get_album(
        &self,
        page: u16,
        limit: u16,
    ) -> Result<(Option<Playlist>, Vec<MusicAggregator>)> {
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
                    .map(|m| MusicAggregator::from_music(m))
                    .collect();
                Ok((album, musics))
            }
            MusicServer::Netease => {
                let (playlist, musics) = netease::web_api::album::get_musics_from_album(
                    self.album_id
                        .as_ref()
                        .ok_or(anyhow::anyhow!("No album id"))?,
                )
                .await?;
                let musics = musics
                    .into_iter()
                    .map(|music| music.into_music(false))
                    .map(|m| MusicAggregator::from_music(m))
                    .collect();
                Ok((Some(playlist), musics))
            }
        }
    }
}

#[cfg(test)]
mod server_music_test {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let musics =
            Music::search_online(vec![MusicServer::Kuwo], "Lemon 米津玄师".to_string(), 1, 10)
                .await
                .unwrap();
        println!("{:?}", musics);
    }
}

impl Playlist {
    pub fn get_cover(&self, size: u16) -> Option<String> {
        match self.server {
            Some(MusicServer::Kuwo) => self.cover.clone().and_then(|cover| {
                Some(
                    cover
                        .replace("_700.", &format!("_{}.", size))
                        .replace("_150.", &format!("_{}.", size))
                        .replace("/240/", &format!("/{}/", size)),
                )
            }),
            Some(MusicServer::Netease) => self
                .cover
                .clone()
                .and_then(|c| Some(format!("{}?param={}y{}", c, size, size))),
            None => self.cover.clone(),
        }
    }

    /// Search playlist online
    pub async fn search_online(
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
                        kuwo::web_api::playlist::search_kuwo_music_list(
                            content.as_str(),
                            page,
                            size,
                        )
                        .await
                    }));
                }
                MusicServer::Netease => {
                    handles.push(tokio::spawn(async move {
                        netease::web_api::playlist::search_netease_music_list(
                            content.as_str(),
                            page as u16,
                            size as u16,
                        )
                        .await
                    }));
                }
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

    /// get a playlist from share link
    pub async fn get_from_share(share: &str) -> Result<Self> {
        if share.contains("kuwo") {
            get_kuwo_music_list_from_share(share).await
        } else if share.contains("music.163.com") {
            get_netease_music_list_from_share(share).await
        } else {
            Err(anyhow::anyhow!("Unsupport share content."))
        }
    }

    /// Fetch musics from playlist
    pub async fn fetch_musics_online(&self, page: u16, limit: u16) -> Result<Vec<MusicAggregator>> {
        if self.from_db || self.server.is_none() {
            return Err(anyhow::anyhow!("Cant't get music from db playlist"));
        }

        let server = self.server.as_ref().ok_or(anyhow::anyhow!(
            "This music is not from db, but has no server."
        ))?;
        match server {
            MusicServer::Kuwo => match self.type_field {
                super::data::interface::playlist::PlaylistType::UserPlaylist => {
                    let kuwo_musics = kuwo::web_api::playlist::get_kuwo_musics_of_music_list(
                        &self.identity,
                        page,
                        limit,
                    )
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
                super::data::interface::playlist::PlaylistType::Album => {
                    let (_playlist, kuwo_musics) = kuwo::web_api::album::get_kuwo_music_album(
                        &self.identity,
                        &self.name,
                        page,
                        limit,
                    )
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
            },
            MusicServer::Netease => match self.type_field {
                super::data::interface::playlist::PlaylistType::UserPlaylist => {
                    let models = netease::web_api::playlist::get_musics_from_music_list(
                        &self.identity,
                        page,
                        limit,
                    )
                    .await?;
                    let musics: Vec<Music> = models
                        .into_iter()
                        .map(|music| music.into_music(false))
                        .collect();

                    Ok(musics
                        .into_iter()
                        .map(|music| MusicAggregator::from_music(music))
                        .collect())
                }
                super::data::interface::playlist::PlaylistType::Album => {
                    let (_album, models) =
                        netease::web_api::album::get_musics_from_album(&self.identity).await?;
                    let musics: Vec<Music> = models
                        .into_iter()
                        .map(|music| music.into_music(false))
                        .collect();

                    Ok(musics
                        .into_iter()
                        .map(|music| MusicAggregator::from_music(music))
                        .collect())
                }
            },
        }
    }
}

impl Music {
    pub async fn get_lyric(&self) -> Result<String> {
        match self.server {
            MusicServer::Kuwo => kuwo::web_api::lyric::get_kuwo_lyric(&self.identity).await,
            MusicServer::Netease => {
                netease::web_api::lyric::get_netease_lyric(&self.identity).await
            }
        }
    }

    pub fn get_cover(&self, size: u16) -> Option<String> {
        match self.server {
            MusicServer::Kuwo => self.cover.clone().and_then(|cover| {
                Some(
                    cover
                        .replace("_700.", &format!("_{}.", size))
                        .replace("/500/", &format!("/{}/", size)),
                )
            }),
            MusicServer::Netease => self
                .cover
                .clone()
                .and_then(|c| Some(format!("{c}?param={size}y{size}"))),
        }
    }
}

#[cfg(test)]
mod server_test {
    use crate::data::interface::{
        music_aggregator::Music, playlist::Playlist, server::MusicServer,
    };

    use super::kuwo::web_api::playlist;

    #[tokio::test]
    async fn test_search() {
        let playlists = super::Playlist::search_online(
            vec![super::MusicServer::Kuwo, super::MusicServer::Netease],
            "周杰伦".to_string(),
            8,
            10,
        )
        .await
        .unwrap();

        println!("{:?}", playlists);
        let playlists = super::Playlist::search_online(
            vec![super::MusicServer::Kuwo, super::MusicServer::Netease],
            "周杰伦".to_string(),
            9999,
            10,
        )
        .await
        .unwrap();

        println!("{:?}", playlists);
    }

    #[tokio::test]
    async fn test_fetch_musics() {
        let playlists = super::Playlist::search_online(
            vec![super::MusicServer::Kuwo, super::MusicServer::Netease],
            "周杰伦".to_string(),
            1,
            10,
        )
        .await
        .unwrap();
        let playlist = playlists.first().unwrap();
        let musics = playlist.fetch_musics_online(1, 10).await.unwrap();
        assert!(musics.len() > 0 && musics.len() <= 10);
        let musics = playlist.fetch_musics_online(999, 10).await.unwrap();
        assert!(musics.len() == 0)
    }

    #[tokio::test]
    async fn test_fetch_all_musics() {
        let playlists = super::Playlist::search_online(
            vec![super::MusicServer::Kuwo, super::MusicServer::Netease],
            "周杰伦".to_string(),
            1,
            10,
        )
        .await
        .unwrap();
        let playlist = playlists.first().unwrap();
        let start = std::time::Instant::now();
        let musics = playlist.fetch_musics_online(1, 999).await.unwrap();

        println!("{:?}", musics);
        println!("Time: {:?}", start.elapsed());
        println!("Length: {}", musics.len());
        assert!(musics.len() > 0);
    }

    #[tokio::test]
    async fn test_from_share() {
        let share = Playlist::get_from_share(
            "https://m.kuwo.cn/newh5app/playlist_detail/1312045587?from=ip&t=qqfriend",
        )
        .await
        .unwrap();
        println!("{:#?}", share);
        let share = Playlist::get_from_share("分享Z殘心的歌单《米津玄师》https://y.music.163.com/m/playlist?app_version=8.9.20&id=6614178314&userid=317416193&dlt=0846&creatorId=317416193 (@网易云音乐)").await.unwrap();
        println!("{:#?}", share);
    }

    #[tokio::test]
    async fn test_from_share2() {
        let share = Playlist::get_from_share(
            "https://music.163.com/playlist?id=12497815913&uct2=U2FsdGVkX19tzJpiufgwqfBqjgNRIDask6O0auKK8SQ=",
        )
        .await
        .unwrap();
        println!("{:#?}", share);
        let music_aggs = share.fetch_musics_online(1, 2333).await.unwrap();
        println!("{:#?}", music_aggs)
    }

    #[tokio::test]
    async fn test_get_cover() {
        let musics = Music::search_online(
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "米津玄师".to_string(),
            1,
            10,
        )
        .await
        .unwrap();
        for music in &musics {
            if let Some(cover) = music.get_cover(100) {
                println!("{}", cover);
                assert!(cover.contains("100"));
            }
        }

        let first_kuwo = musics
            .iter()
            .find(|m| m.server == MusicServer::Kuwo && m.album_id.is_some())
            .unwrap();

        let (kuwo_album, music_aggs) = first_kuwo.get_album(1, 10).await.unwrap();

        if let Some(cover) = kuwo_album.unwrap().get_cover(100) {
            println!("{}", cover);
            assert!(cover.contains("100"));
        }

        for music_agg in &music_aggs {
            if let Some(cover) = music_agg.musics.first().unwrap().get_cover(100) {
                assert!(cover.contains("100"));
            }
        }

        let first_netease = musics
            .iter()
            .find(|m| m.server == MusicServer::Netease && m.album_id.is_some())
            .unwrap();

        let (netease_album, music_aggs) = first_netease.get_album(1, 10).await.unwrap();

        if let Some(cover) = netease_album.unwrap().get_cover(100) {
            assert!(cover.contains("100"));
        }

        for music_agg in &music_aggs {
            if let Some(cover) = music_agg.musics.first().unwrap().get_cover(100) {
                assert!(cover.contains("100"));
            }
        }

        let playlists = Playlist::search_online(
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "米津玄师".to_string(),
            1,
            30,
        )
        .await
        .unwrap();

        for playlist in &playlists {
            if let Some(cover) = playlist.get_cover(100) {
                println!("{}", cover);
                assert!(cover.contains("100"));
            }
        }
    }
}
