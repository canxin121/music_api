use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    interface::{
        artist::Artist,
        playlist::{Playlist, PlaylistType},
        server::MusicServer,
    },
    server::kuwo,
    CLIENT,
};

use super::utils::{decode_html_entities, get_music_rid_pic, parse_qualities_formats};

pub(crate) async fn get_kuwo_music_album(
    album_id: &str,
    album_name: &str,
    page: u16,
    limit: u16,
) -> Result<(Option<Playlist>, Vec<kuwo::model::Model>)> {
    if page == 0 {
        return Err(anyhow::anyhow!("Page must be more than or equal 1."));
    }

    let url = format!("http://search.kuwo.cn/r.s?pn={}&rn={}&stype=albuminfo&albumid={}&show_copyright_off=0&encoding=utf&vipver=MUSIC_9.1.0",page-1,limit,album_id);

    let text = CLIENT
        .get(&url)
        .send()
        .await?
        .text()
        .await?
        .replace("'", "\"");
    // std::fs::write("sample_data/kuwo/album.json", &text).unwrap();
    let mut result: Album = serde_json::from_str(&text)?;
    let mut musics = Vec::new();
    std::mem::swap(&mut musics, &mut result.musiclist);

    let mut handles = Vec::with_capacity(musics.len());

    for music in musics.iter_mut() {
        music.album = album_name.to_string();
        music.album_id = album_id.to_string();
        let music_id = music.id.clone();
        handles.push(async move {
            let music_pic = get_music_rid_pic(&music_id).await?;
            Ok::<String, anyhow::Error>(music_pic.ok_or(anyhow!("No pic found."))?)
        })
    }

    for (music, handle) in musics.iter_mut().zip(handles.into_iter()) {
        music.cover = handle.await.ok();
    }

    let musics = musics
        .into_iter()
        .map(|m| {
            let model: crate::server::kuwo::model::Model = m.into();
            model
        })
        .collect();

    if page == 1 {
        Ok((Some(result.into()), musics))
    } else {
        Ok((None, musics))
    }
}

#[tokio::test]
async fn test_get_kuwo_music_album() {
    let (playlist, musics) = get_kuwo_music_album("56865692", "LOST CORNER", 1, 10)
        .await
        .unwrap();

    println!("{:?}", playlist);
    println!("{:?}", musics);
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    // pub aartist: String,
    // #[serde(rename = "ad_subtype")]
    // pub ad_subtype: String,
    // #[serde(rename = "ad_type")]
    // pub ad_type: String,
    // pub albumid: String,
    pub artist: String,
    pub artistid: String,
    // pub artistpic: String,
    // pub company: String,
    // #[serde(rename = "content_type")]
    // pub content_type: String,
    // pub falbum: String,
    // pub fartist: String,
    // pub finished: String,
    // #[serde(rename = "hts_img")]
    // pub hts_img: String,
    pub id: String,
    pub img: String,
    pub info: String,
    // pub lang: String,
    pub musiclist: Vec<AlbumMusic>,
    pub name: String,
    // pub pay: String,
    // pub pic: String,
    // #[serde(rename = "pub")]
    // pub pub_field: String,
    pub songnum: String,
    // #[serde(rename = "sort_policy")]
    // pub sort_policy: String,
    // pub sp_privilege: String,
    // pub tag: Vec<Tag>,
    // pub title: String,
    // pub vip: String,
}

impl Into<Playlist> for Album {
    fn into(self) -> Playlist {
        Playlist {
            server: Some(MusicServer::Kuwo),
            type_field: PlaylistType::Album,
            identity: self.id,
            name: decode_html_entities(self.name),
            summary: Some(self.info),
            cover: Some(self.img),
            creator: Some(self.artist),
            creator_id: Some(self.artistid),
            play_time: None,
            music_num: if let Ok(n) = self.songnum.parse() {
                Some(n)
            } else {
                None
            },
            subscription: None,
            from_db: false,
            order: None,
            collection_id: None,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMusic {
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub album_id: String,
    // #[serde(rename = "CanSetRing")]
    // pub can_set_ring: String,
    // #[serde(rename = "CanSetRingback")]
    // pub can_set_ringback: String,
    // #[serde(rename = "MVFLAG")]
    // pub mvflag: Option<String>,
    // pub aartist: String,
    // #[serde(rename = "ad_subtype")]
    // pub ad_subtype: String,
    // #[serde(rename = "ad_type")]
    // pub ad_type: String,
    pub allartistid: String,
    pub artist: String,
    // pub artistid: String,
    // pub audiobookpayinfo: Audiobookpayinfo,
    // pub barrage: String,
    // #[serde(rename = "cache_status")]
    // pub cache_status: String,
    // #[serde(rename = "content_type")]
    // pub content_type: String,
    // pub copyright: String,
    // pub fartist: String,
    pub formats: String,
    // pub fpay: String,
    // pub fsongname: String,
    pub id: String,
    // #[serde(rename = "iot_info")]
    // pub iot_info: String,
    // #[serde(rename = "is_point")]
    // pub is_point: String,
    // pub isdownload: String,
    // pub isshowtype: String,
    // pub mp4sig1: String,
    // pub mp4sig2: String,
    // #[serde(rename = "muti_ver")]
    // pub muti_ver: String,
    // pub mvpayinfo: Mvpayinfo,
    pub name: String,
    // pub nationid: String,
    // pub online: String,
    // pub opay: String,
    // pub originalsongtype: String,
    // #[serde(rename = "overseas_copyright")]
    // pub overseas_copyright: String,
    // #[serde(rename = "overseas_pay")]
    // pub overseas_pay: String,
    // pub param: String,
    // pub pay: String,
    // pub pay_info: PayInfo,
    // pub playcnt: String,
    // pub rdts: String,
    // pub releasedate: String,
    // pub score: String,
    // pub score100: String,
    // pub sp_privilege: String,
    // pub subs_strategy: String,
    // pub subs_text: String,
    // pub subtitle: String,
    // pub terminal: String,
    // #[serde(rename = "tme_musician_adtype")]
    // pub tme_musician_adtype: String,
    // pub tpay: String,
    // pub track: String,
    // pub uploader: String,
    // pub uptime: String,
    // #[serde(rename = "web_albumpic_short")]
    // pub web_albumpic_short: String,
    // #[serde(rename = "web_artistpic_short")]
    // pub web_artistpic_short: String,
    // #[serde(rename = "web_timingonline")]
    // pub web_timingonline: String,
}

impl Into<crate::server::kuwo::model::Model> for AlbumMusic {
    fn into(self) -> crate::server::kuwo::model::Model {
        let artist_names = self
            .artist
            .split("&")
            .filter(|a| !a.is_empty())
            .collect::<Vec<&str>>();
        let artist_ids = self
            .allartistid
            .split("&")
            .filter(|a| !a.is_empty())
            .collect::<Vec<&str>>();
        let artists: Vec<Artist> = artist_names
            .into_iter()
            .zip(artist_ids.into_iter().chain(std::iter::repeat("")))
            .map(|(name, id)| crate::interface::artist::Artist {
                name: name.to_string(),
                id: id.parse().ok(),
            })
            .collect();
        let artists = crate::interface::artist::ArtistVec::from(artists);
        crate::server::kuwo::model::Model {
            name: decode_html_entities(self.name),
            music_id: self
                .id
                .strip_prefix("MUSIC_")
                .unwrap_or(&self.id)
                .to_string(),
            artists,
            album: Some(self.album),
            album_id: Some(self.album_id),
            qualities: parse_qualities_formats(&self.formats).into(),
            cover: self.cover,
            // artist_pic: build_web_artistpic_short(&self.web_artistpic_short),
            // album_pic: build_web_albumpic_short(&self.web_albumpic_short),
            duration: None,
        }
    }
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Audiobookpayinfo {
//     pub download: String,
//     pub play: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Mvpayinfo {
//     pub download: String,
//     pub play: String,
//     pub vid: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PayInfo {
//     pub cannot_download: String,
//     pub cannot_online_play: String,
//     pub download: String,
//     pub fee_type: FeeType,
//     pub limitfree: String,
//     #[serde(rename = "listen_fragment")]
//     pub listen_fragment: String,
//     #[serde(rename = "local_encrypt")]
//     pub local_encrypt: String,
//     pub ndown: String,
//     pub nplay: String,
//     #[serde(rename = "overseas_ndown")]
//     pub overseas_ndown: String,
//     #[serde(rename = "overseas_nplay")]
//     pub overseas_nplay: String,
//     pub paytagindex: Paytagindex,
//     pub play: String,
//     #[serde(rename = "refrain_end")]
//     pub refrain_end: String,
//     #[serde(rename = "refrain_start")]
//     pub refrain_start: String,
//     #[serde(rename = "tips_intercept")]
//     pub tips_intercept: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct FeeType {
//     pub album: String,
//     pub bookvip: String,
//     pub song: String,
//     pub vip: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Paytagindex {
//     #[serde(rename = "AR501")]
//     pub ar501: i64,
//     #[serde(rename = "DB")]
//     pub db: i64,
//     #[serde(rename = "F")]
//     pub f: i64,
//     #[serde(rename = "H")]
//     pub h: i64,
//     #[serde(rename = "HR")]
//     pub hr: i64,
//     #[serde(rename = "L")]
//     pub l: i64,
//     #[serde(rename = "S")]
//     pub s: i64,
//     #[serde(rename = "ZP")]
//     pub zp: i64,
//     #[serde(rename = "ZPGA201")]
//     pub zpga201: i64,
//     #[serde(rename = "ZPGA501")]
//     pub zpga501: i64,
//     #[serde(rename = "ZPLY")]
//     pub zply: i64,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Tag {
//     pub cat1: String,
//     pub cat2: String,
//     #[serde(rename = "type")]
//     pub type_field: String,
// }
