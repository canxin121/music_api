use anyhow::Result;
use kuwo::web_api::share_playlist::get_kuwo_music_list_from_share;
use netease::web_api::share_playlist::get_netease_music_list_from_share;
use std::collections::HashMap;
use std::sync::Arc;

pub mod kuwo;
pub mod netease;

use crate::interface::music_chart::ServerMusicChartCollection;
use crate::interface::playlist_tag::ServerPlaylistTagCollection;
use crate::interface::playlist_tag::TagPlaylistOrder;
use crate::interface::server::MusicServer;

use super::interface::music_aggregator::Music;
use super::interface::music_aggregator::MusicAggregator;
use super::interface::playlist::Playlist;

impl Music {
    /// Search music online
    pub async fn search_online(
        servers: Vec<MusicServer>,
        content: String,
        page: u16,
        size: u16,
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
                        page,
                        size,
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

impl MusicAggregator {
    /// takes ownership
    pub async fn search_online(
        aggs: Vec<MusicAggregator>,
        servers: Vec<MusicServer>,
        content: String,
        page: u16,
        size: u16,
    ) -> anyhow::Result<Vec<Self>> {
        if servers.is_empty() {
            return Err(anyhow::anyhow!("No servers provided".to_string()));
        }

        let mut map = {
            if !aggs.is_empty() {
                let pair = aggs
                    .into_iter()
                    .enumerate()
                    .collect::<Vec<(usize, MusicAggregator)>>();
                let map: HashMap<String, (usize, MusicAggregator)> = pair
                    .into_iter()
                    .map(|pair| (pair.1.identity(), pair))
                    .collect();
                map
            } else {
                HashMap::new()
            }
        };

        let mut success = false;
        if let Some(musics) = Music::search_online(servers, content, page, size)
            .await
            .ok()
        {
            for music in musics {
                let identity = format!("{}#+#{}", music.name, {
                    let mut artists = music
                        .artists
                        .iter()
                        .map(|x| x.name.clone())
                        .collect::<Vec<String>>();
                    artists.sort();
                    artists.join("&")
                })
                .to_lowercase();
                if let Some(pair) = map.get_mut(&identity) {
                    if !pair.1.musics.iter().any(|x| x.server == music.server) {
                        pair.1.musics.push(music);
                    }
                } else {
                    let index = map.len();
                    map.insert(
                        identity.clone(),
                        (
                            index,
                            MusicAggregator {
                                name: music.name.clone(),
                                artist: music
                                    .artists
                                    .iter()
                                    .map(|x| x.name.clone())
                                    .collect::<Vec<String>>()
                                    .join("&"),
                                from_db: false,
                                default_server: music.server.clone(),
                                musics: vec![music],
                                order: None,
                            },
                        ),
                    );
                }
            }
            success = true;
        }

        let mut pairs: Vec<(usize, MusicAggregator)> =
            map.into_iter().map(|(_, pair)| pair).collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let aggs = pairs.into_iter().map(|pair| pair.1).collect();

        if success {
            Ok(aggs)
        } else {
            Err(anyhow::anyhow!("Music search failed".to_string()))
        }
    }

    /// takes ownership
    pub async fn fetch_server_online(
        mut self,
        mut servers: Vec<MusicServer>,
    ) -> anyhow::Result<Self> {
        servers.retain(|x| !self.musics.iter().any(|y| y.server == *x));

        if servers.is_empty() {
            return Err(anyhow::anyhow!("No more servers to fetch".to_string()));
        }
        match Music::search_online(servers.clone(), self.identity(), 1, 10).await {
            Ok(musics) => {
                if musics.is_empty() {
                    return Err(anyhow::anyhow!("No musics found from servers".to_string()));
                }
                for server in servers {
                    if let Some(music) = musics.iter().find(|x| {
                        x.server == server && x.name == self.name && {
                            let mut artists = x
                                .artists
                                .iter()
                                .map(|artist| artist.name.as_str())
                                .collect::<Vec<&str>>();
                            artists.sort();
                            artists.join("&") == self.artist
                        }
                    }) {
                        self.musics.push(music.clone());
                    }
                }
                Ok(self)
            }
            Err(e) => Err(anyhow::anyhow!(format!("Failed to fetch servers: {}", e))),
        }
    }
    pub async fn fetch_artist_music_aggregators(
        server: MusicServer,
        artist_id: &str,
        page: u16,
        limit: u16,
    ) -> anyhow::Result<Vec<Self>> {
        match server {
            MusicServer::Kuwo => kuwo::web_api::artist::get_artist_musics(artist_id, page, limit)
                .await
                .and_then(|musics| {
                    Ok(musics
                        .into_iter()
                        .map(|music| {
                            MusicAggregator::from_music(Music::from(music.into_music(false)))
                        })
                        .collect())
                }),
            MusicServer::Netease => {
                netease::web_api::artist::get_artist_musics(artist_id, page, limit)
                    .await
                    .and_then(|musics| {
                        Ok(musics
                            .into_iter()
                            .map(|music| {
                                MusicAggregator::from_music(Music::from(music.into_music(false)))
                            })
                            .collect())
                    })
            }
        }
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
                        .replace("/240/", &format!("/{}/", size))
                        .replace("/120/", &format!("/{}/", size)),
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
        page: u16,
        size: u16,
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
                            page,
                            size,
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
                super::interface::playlist::PlaylistType::UserPlaylist => {
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
                super::interface::playlist::PlaylistType::Album => {
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
                super::interface::playlist::PlaylistType::UserPlaylist => {
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
                super::interface::playlist::PlaylistType::Album => {
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

    pub async fn fetch_artist_albums(
        server: MusicServer,
        artist_id: &str,
        page: u16,
        limit: u16,
    ) -> Result<Vec<Playlist>> {
        match server {
            MusicServer::Kuwo => {
                kuwo::web_api::artist::get_artist_albums(artist_id, page, limit).await
            }
            MusicServer::Netease => {
                netease::web_api::artist::get_artist_albums(artist_id, page, limit).await
            }
        }
    }
}

impl ServerMusicChartCollection {
    pub async fn get_music_chart_collection() -> Result<Vec<ServerMusicChartCollection>> {
        let mut handles = Vec::with_capacity(MusicServer::length());
        // todo: add more servers
        handles.push(tokio::spawn(async move {
            kuwo::web_api::chart::get_music_chart_collection().await
        }));
        handles.push(tokio::spawn(async move {
            netease::web_api::chart::get_music_chart_collection().await
        }));

        let mut collections = Vec::new();
        for handle in handles {
            if let Ok(Ok(collection)) = handle.await {
                collections.push(collection);
            }
        }
        Ok(collections)
    }

    pub async fn get_musics_from_chart(
        server: MusicServer,
        id: &str,
        page: u16,
        limit: u16,
    ) -> Result<Vec<MusicAggregator>> {
        match server {
            MusicServer::Kuwo => kuwo::web_api::chart::get_musics_from_chart(id, page, limit)
                .await
                .and_then(|musics| {
                    Ok(musics
                        .into_iter()
                        .map(|music| MusicAggregator::from_music(music.into_music(false)))
                        .collect())
                }),
            MusicServer::Netease => netease::web_api::chart::get_musics_from_chart(id, page, limit)
                .await
                .and_then(|musics| {
                    Ok(musics
                        .into_iter()
                        .map(|music| MusicAggregator::from_music(music.into_music(false)))
                        .collect())
                }),
        }
    }
}

impl ServerPlaylistTagCollection {
    pub async fn get_playlist_tags() -> Result<Vec<ServerPlaylistTagCollection>> {
        let mut handles = Vec::with_capacity(MusicServer::length());
        handles.push(tokio::spawn(async move {
            kuwo::web_api::playlist_tag::get_playlist_tags().await
        }));
        handles.push(tokio::spawn(async move {
            netease::web_api::playlist_tag::get_playlist_tags().await
        }));

        let mut collections = Vec::new();
        for handle in handles {
            if let Ok(Ok(collection)) = handle.await {
                collections.push(collection);
            }
        }
        Ok(collections)
    }

    pub async fn get_playlists_from_tag(
        server: MusicServer,
        tag_id: &str,
        order: TagPlaylistOrder,
        page: u16,
        limit: u16,
    ) -> Result<Vec<Playlist>> {
        match server {
            MusicServer::Kuwo => {
                kuwo::web_api::playlist_tag::get_playlists_from_tag(tag_id, order, page, limit)
                    .await
            }
            MusicServer::Netease => {
                netease::web_api::playlist_tag::get_playlists_from_tag(tag_id, order, page, limit)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::interface::{
        music_aggregator::Music, playlist::Playlist, playlist_tag::TagPlaylistOrder,
        server::MusicServer,
    };

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

    #[tokio::test]
    async fn test_chart() {
        let collections = super::ServerMusicChartCollection::get_music_chart_collection()
            .await
            .unwrap();
        println!("{:?}", collections);

        let first_collection = collections.first().unwrap();
        let first_chart = first_collection
            .collections
            .first()
            .unwrap()
            .charts
            .first()
            .unwrap();

        let musics = super::ServerMusicChartCollection::get_musics_from_chart(
            first_collection.server.clone(),
            &first_chart.id,
            1,
            10,
        )
        .await
        .unwrap();

        println!("{:?}", musics);
    }

    #[tokio::test]
    async fn test_playlist_tag() {
        let collections = super::ServerPlaylistTagCollection::get_playlist_tags()
            .await
            .unwrap();
        println!("{:?}", collections);

        let first_collection = collections.first().unwrap();
        let first_tag = first_collection
            .collections
            .first()
            .unwrap()
            .tags
            .first()
            .unwrap();

        let playlists = super::ServerPlaylistTagCollection::get_playlists_from_tag(
            first_collection.server.clone(),
            &first_tag.id,
            TagPlaylistOrder::Hot,
            1,
            10,
        )
        .await
        .unwrap();

        println!("{:?}", playlists);
    }
}
