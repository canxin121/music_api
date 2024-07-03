use std::mem;

use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};

use futures::future::join;

use crate::{
    music_aggregator::music_aggregator_online::SearchMusicAggregator,
    music_list::{MusicList, MusicListTrait},
    platform_integrator::kuwo::util::decode_html_entities,
    util::CLIENT,
    Music, MusicAggregator, MusicListInfo,
};

use super::{
    kuwo_lyric::get_lrc,
    kuwo_music::KuwoMusic,
    kuwo_music_info::get_music_info,
    kuwo_quality::{gen_minfo_from_formats, process_qualities, KuWoQuality},
    KUWO,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct AlbumResult {
    name: String,
    artist: String,
    info: String,
    albumid: String,
    img: String,
    musiclist: Vec<AlbumMusic>,
}

impl MusicListTrait for AlbumResult {
    fn source(&self) -> String {
        KUWO.to_string()
    }

    fn get_musiclist_info(&self) -> MusicListInfo {
        MusicListInfo {
            id: 0,
            name: self.name.clone(),
            art_pic: self.img.clone(),
            desc: self.info.clone(),
            extra: None,
        }
    }

    fn get_music_aggregators(
        &self,
        page: u32,
        limit: u32,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                    Output = Result<Vec<crate::music_aggregator::MusicAggregator>, anyhow::Error>,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let (_, aggregators) = get_music_album(&self.albumid, &self.name, page, limit).await?;
            Ok(aggregators)
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlbumMusic {
    name: String,
    id: String,
    artist: String,
    artistid: String,
    formats: String,
}

pub fn gen_album_url(album_id: &str, page: u32, limit: u32) -> String {
    format!("http://search.kuwo.cn/r.s?pn={}&rn={}&stype=albuminfo&albumid={}&show_copyright_off=0&encoding=utf&vipver=MUSIC_9.1.0",page-1,limit,album_id)
}

pub async fn get_music_album(
    album_id: &str,
    album: &str,
    page: u32,
    limit: u32,
) -> Result<(MusicList, Vec<MusicAggregator>), anyhow::Error> {
    assert!(page >= 1, "Page must be greater than 0");
    let url = gen_album_url(album_id, page, limit);
    let mut text = CLIENT
        .get(&url)
        .send()
        .await?
        .text()
        .await?
        .replace("'", "\"");
    text = decode_html_entities(text);

    let mut album_result = serde_json::from_str::<AlbumResult>(&text)?;

    let mut musiclist = Vec::new();
    mem::swap(&mut musiclist, &mut album_result.musiclist);

    let mut music_futures: FuturesUnordered<_> = musiclist
        .into_iter()
        .map(|m| {
            let album = album.to_string();
            let album_id = album_id.to_string();
            let artist = album_result.artist.clone();
            async move {
                let (lrc_result, music_info_result) =
                    join(get_lrc(&m.id), get_music_info(&m.id)).await;

                let raw_quality = gen_minfo_from_formats(&m.formats);
                let mut qualities = KuWoQuality::parse_quality(&raw_quality);
                qualities = process_qualities(qualities);
                let default_quality = qualities.first().cloned().unwrap_or_default();

                let music = KuwoMusic {
                    album,
                    album_id,
                    artist,
                    artist_id: m.artistid,
                    format: "wma".to_string(),
                    song_name: decode_html_entities(m.name),
                    music_rid: m.id,
                    minfo: raw_quality,
                    n_minfo: String::with_capacity(0),
                    duration: music_info_result
                        .as_ref()
                        .map_or("unknown".to_string(), |info| info.duration.to_string()),
                    quality: qualities,
                    default_quality,
                    pic: music_info_result
                        .as_ref()
                        .map_or(String::new(), |info| info.img.clone()),
                    lyric: lrc_result.unwrap_or_default(),
                    id: 0,
                };
                Ok::<Option<Music>, anyhow::Error>(Some(Box::new(music) as Music))
            }
        })
        .collect();

    let mut musics = Vec::new();
    while let Some(music_result) = music_futures.next().await {
        if let Some(music) = music_result? {
            musics.push(Box::new(SearchMusicAggregator::from_music(music)) as MusicAggregator);
        }
    }

    Ok((Box::new(album_result) as MusicList, musics))
}
