use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};

use futures::future::join;
use serde_json::Value;

use crate::{search_factory::CLIENT, Music, MusicList};

use super::{
    kuwo_lyric::get_lrc,
    kuwo_music::KuwoMusic,
    kuwo_music_info::get_music_info,
    kuwo_quality::{gen_minfo_from_formats, process_qualities, KuWoQuality},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchResult {
    name: String,
    artist: String,
    info: String,
    albumid: String,
    img: String,
    musiclist: Vec<AlbumMusic>,
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
    payload: Value,
    page: u32,
) -> Result<(MusicList, Vec<Music>), anyhow::Error> {
    let album = payload
        .get("album")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid Payload"))?;
    let album_id = payload
        .get("album_id")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid Payload"))?;
    let url = gen_album_url(album_id, page, 30);
    let text = CLIENT
        .get(&url)
        .send()
        .await?
        .text()
        .await?
        .replace("'", "\"");
    let mut result = serde_json::from_str::<SearchResult>(&text)?;
    result.name = result.name.replace("&nbsp;", " ");
    result.artist = result.artist.replace("&nbsp;", " ");
    result.info = result.info.replace("&nbsp;", " ");

    let music_list = MusicList {
        name: result.name,
        art_pic: result.img.to_string(),
        desc: result.info,
    };

    let mut music_futures: FuturesUnordered<_> = result
        .musiclist
        .into_iter()
        .map(|m| {
            let album = album.to_string();
            let album_id = album_id.to_string();
            let artist = result.artist.to_string().replace("&nbsp;", " ");
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
                    song_name: m.name.replace("&nbsp;", " "),
                    music_rid: m.id,
                    minfo: raw_quality,
                    n_minfo: String::with_capacity(0),
                    duration: music_info_result
                        .as_ref()
                        .map_or("unknown".to_string(), |info| info.duration.to_string()),
                    quality: qualities,
                    default_quality: default_quality,
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
            musics.push(music);
        }
    }

    Ok((music_list, musics))
}
