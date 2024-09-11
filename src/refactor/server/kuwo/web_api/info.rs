use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::refactor::server::CLIENT;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicInfoResult {
    pub status: String,
    pub msg: Vec<Msg>,
    #[serde(rename = "video_show")]
    pub video_show: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Msg {
    pub disname: String,
    pub creator: Creator,
    pub can_full_screen: i64,
    #[serde(rename = "like_count")]
    pub like_count: i64,
    #[serde(rename = "overseas_pay")]
    pub overseas_pay: String,
    pub duration: i64,
    pub id: i64,
    pub listencnt: i64,
    pub album: String,
    pub img: String,
    pub title: String,
    #[serde(rename = "comment_count")]
    pub comment_count: i64,
    #[serde(rename = "dislike_count")]
    pub dislike_count: i64,
    pub source: String,
    #[serde(rename = "fav_status")]
    pub fav_status: i64,
    pub videosrc: i64,
    #[serde(rename = "fav_count")]
    pub fav_count: i64,
    #[serde(rename = "dislike_status")]
    pub dislike_status: i64,
    pub mvquality: String,
    #[serde(rename = "overseas_copyright")]
    pub overseas_copyright: String,
    pub traceid: String,
    pub mvpayinfo: Mvpayinfo,
    pub extend: String,
    #[serde(rename = "like_status")]
    pub like_status: i64,
    pub disable: i64,
    pub desc: String,
    pub name: String,
    #[serde(rename = "statistics_id")]
    pub statistics_id: String,
    pub artist: String,
    pub hot: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub uid: String,
    pub img: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mvpayinfo {
    pub down: String,
    pub download: String,
    pub play: String,
    pub vid: String,
}

pub async fn get_kuwo_music_info(music_rid: &str) -> Result<MusicInfoResult> {
    let url = format!("https://fvedio.kuwo.cn/rec.s?rid={}&cmd=rcm_switch&idfa=&prod=kwplayersimple_ip_1.0.2.0&source=kwplayersimple_ip_1.0.2.0_TJ.ipa&corp=kuwo&plat=ip&tmeapp=1&prod_from=kwplayersimple",music_rid.replace("MUSIC_", ""));
    let info: MusicInfoResult = CLIENT.get(url).send().await?.json().await?;

    Ok(info)
}
