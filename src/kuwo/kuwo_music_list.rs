use futures::stream::{self, StreamExt};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::json;
use urlencoding::encode;

use crate::{Music, MusicList};

use super::{
    kuwo_lyric::get_lrc,
    kuwo_music::KuwoMusic,
    kuwo_quality::{process_qualities, KuWoQuality},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    abslist: Vec<KuwoMusicList>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KuwoMusicList {
    pic: String,
    name: String,
    intro: String,
    playlistid: String,
}

pub fn gen_music_list_url(content: &str, page: u32, limit: u32) -> String {
    format!("http://search.kuwo.cn/r.s?all={}&pn={}&rn={limit}&rformat=json&encoding=utf8&ver=mbox&vipver=MUSIC_8.7.7.0_BCS37&plat=pc&devid=28156413&ft=playlist&pay=0&needliveshow=0",encode(content),page-1)
}

pub fn gen_get_musics_url(playlist_id: &str, page: u32, limit: u32) -> String {
    format!("http://nplserver.kuwo.cn/pl.svc?op=getlistinfo&pid={playlist_id}&pn={}&rn={limit}&encode=utf8&keyset=pl2012&identity=kuwo&pcmp4=1&vipver=MUSIC_9.0.5.0_W1&newver=1",page-1)
}

pub async fn search_music_list(
    content: &str,
    page: u32,
) -> Result<Vec<(String, MusicList)>, anyhow::Error> {
    let url = gen_music_list_url(content, page, 30);
    let text = reqwest::get(&url).await?.text().await?.replace("'", "\"");
    let search_result: SearchResult = serde_json::from_str(&text)?;
    Ok(search_result
        .abslist
        .into_iter()
        .map(|m| {
            (
                json!({"playlist_id":m.playlistid}).to_string(),
                MusicList {
                    name: m.name.replace("&nbsp;", " "),
                    art_pic: m.pic,
                    desc: m.intro.replace("&nbsp;", " "),
                },
            )
        })
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct GetMusicsResult {
    musiclist: Vec<KuwoMusic>,
}

pub async fn get_musics_of_music_list(
    payload: &str,
    page: u32,
) -> Result<Vec<Music>, anyhow::Error> {
    let value = serde_json::Value::from_str(payload)?;
    let playlist_id = value
        .get("playlist_id")
        .and_then(|r| r.as_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid payload"))?;
    let url = gen_get_musics_url(playlist_id, page, 30);
    let result: GetMusicsResult = reqwest::get(url).await?.json().await?;

    let music_futures = result.musiclist.into_iter().map(|mut music| async move {
        let lyric = get_lrc(&music.music_rid).await.unwrap_or_default();
        let qualities = process_qualities(KuWoQuality::parse_quality(&music.minfo));
        let default_quality = qualities.first().cloned().unwrap_or_default();
        music.quality = qualities;
        music.default_quality = default_quality;
        music.lyric = lyric;
        Ok(Box::new(music) as Music)
    });

    let musics: Result<Vec<Music>, _> = stream::iter(music_futures)
        .buffer_unordered(30)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect();

    musics
}

#[test]
fn test_url() {
    println!("{}", gen_music_list_url("张惠妹", 1, 30));
}

#[tokio::test]
async fn test_get() {
    let result = search_music_list("张惠妹", 1).await.unwrap();
    let first = result.first().unwrap();
    let url = gen_get_musics_url(&first.0, 1, 30);
    println!("{:#?}", result);
    println!("{}", url);
}

#[tokio::test]
async fn test_get_musics() {
    let musics = get_musics_of_music_list(&json!({"playlist_id":"3452422908"}).to_string(), 1)
        .await
        .unwrap();
    musics.iter().for_each(|m| {
        println!("{}", m.get_music_info());
    })
}
