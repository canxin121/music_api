use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Music, MusicList};

use super::{
    kuwo_lyric::get_lrc,
    kuwo_music::KuwoMusic,
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
    format!("http://search.kuwo.cn/r.s?pn={}&rn={}&stype=albuminfo&albumid={}&show_copyright_off=0&encoding=utf&vipver=MUSIC_9.1.0",page,limit,album_id)
}

use futures::stream::{FuturesUnordered, StreamExt};

pub async fn get_all_album_music(payload: Value) -> Result<(MusicList, Vec<Music>), anyhow::Error> {
    let album = payload
        .get("album")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid Payload"))?;
    let album_id = payload
        .get("album_id")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid Payload"))?;
    let url = gen_album_url(album_id, 0, 999);
    let text = reqwest::get(&url).await?.text().await?.replace("'", "\"");
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
            let pic = result.img.to_string();
            async move {
                let lrc = get_lrc(&m.id).await.map_err(|e| anyhow::anyhow!(e))?;
                let raw_quality = gen_minfo_from_formats(&m.formats);
                let mut qualities: Vec<KuWoQuality> = KuWoQuality::parse_quality(&raw_quality);
                qualities = process_qualities(qualities);
                if qualities.is_empty() {
                    return Err(anyhow::anyhow!("No qualities"));
                }
                let default_quality = qualities.first().unwrap().clone();
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
                    duration: String::with_capacity(0),
                    quality: qualities,
                    default_quality: default_quality,
                    pic,
                    lyric: lrc,
                    id: 0,
                };
                Ok::<_, anyhow::Error>(Box::new(music) as Music)
            }
        })
        .collect();

    let mut musics = Vec::new();
    while let Some(music) = music_futures.next().await {
        musics.push(music?);
    }

    Ok((music_list, musics))
}
