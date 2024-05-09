use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::json;
use urlencoding::encode;

use crate::{kuwo::KUWO, Music, MusicList};

use super::kuwo_music::KuwoMusic;

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
                    name: m.name,
                    art_pic: m.pic,
                    desc: m.intro,
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
    Ok(result
        .musiclist
        .into_iter()
        .map(|m| Box::new(m) as Music)
        .collect())
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
    let musics = get_musics_of_music_list("3452422908", 1).await.unwrap();
}