use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    interface::{
        artist::Artist,
        playlist::{Playlist, PlaylistType},
        server::MusicServer,
    },
    server::kuwo::{self, web_api::utils::get_music_rid_pic},
    CLIENT,
};

use super::utils::{decode_html_entities, parse_qualities_formats};

pub async fn get_artist_musics(
    artist_id: &str,
    page: u16,
    page_size: u16,
) -> anyhow::Result<Vec<kuwo::model::Model>> {
    if page < 1 {
        return Err(anyhow::anyhow!("page must be greater than 0"));
    }
    let url = format!("https://search.kuwo.cn/r.s?pn={}&rn={}&artistid={}&stype=artist2music&sortby=0&alflac=1&show_copyright_off=1&pcmp4=1&encoding=utf8&plat=pc&thost=search.kuwo.cn&vipver=MUSIC_9.1.1.2_BCS2&devid=38668888&newver=1&pcjson=1",page-1, page_size, artist_id);

    let result: ArtistMusicsResult = CLIENT.post(&url).send().await?.json().await?;
    let mut musics = result.musiclist;
    let mut handles = Vec::with_capacity(musics.len());

    for music in musics.iter_mut() {
        let music_id = music.musicrid.clone();
        handles.push(async move {
            let music_pic = get_music_rid_pic(&music_id).await?;
            Ok::<String, anyhow::Error>(music_pic.ok_or(anyhow!("no pic"))?)
        });
    }

    for (music, handle) in musics.iter_mut().zip(handles.into_iter()) {
        music.cover = handle.await.ok();
    }

    musics
        .into_iter()
        .map(|music| {
            let model: kuwo::model::Model = music.into();
            Ok(model)
        })
        .collect()
}

pub async fn get_artist_albums(
    artist_id: &str,
    page: u16,
    page_size: u16,
) -> anyhow::Result<Vec<Playlist>> {
    let url = format!("https://search.kuwo.cn/r.s?pn={}&rn={}&artistid={}&stype=albumlist&sortby=1&alflac=1&show_copyright_off=1&pcmp4=1&encoding=utf8&plat=pc&thost=search.kuwo.cn&vipver=MUSIC_9.1.1.2_BCS2&devid=38668888&pcjson=1",page-1, page_size, artist_id);

    let result: ArtistAlbumResult = CLIENT.get(&url).send().await?.json().await?;

    result
        .albumlist
        .into_iter()
        .map(|a| {
            let playlist: Playlist = a.into();
            Ok(playlist)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::server::kuwo::web_api::artist::{get_artist_albums, get_artist_musics};

    #[tokio::test]
    async fn test_get_artist_musics() {
        let result = get_artist_musics("74016", 1, 30).await.unwrap();
        result.iter().for_each(|m| println!("{:?}", m.cover));
    }

    #[tokio::test]
    async fn test_get_artist_albums() {
        let result = get_artist_albums("74016", 1, 30).await.unwrap();
        println!("{:?}", result);
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistMusicsResult {
    // pub artist: String,
    pub musiclist: Vec<ArtistMusic>,
    // pub pn: String,
    // #[serde(rename = "return")]
    // pub return_field: String,
    // pub total: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistMusic {
    #[serde(default)]
    pub cover: Option<String>,
    // #[serde(rename = "COPYRIGHT")]
    // pub copyright: String,
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
    pub album: String,
    pub albumid: String,
    pub allartistid: String,
    pub artist: String,
    // pub artistid: String,
    // pub audiobookpayinfo: Audiobookpayinfo,
    // pub barrage: String,
    // #[serde(rename = "cache_status")]
    // pub cache_status: String,
    // #[serde(rename = "content_type")]
    // pub content_type: String,
    // pub falbum: String,
    // pub fartist: String,
    pub formats: String,
    // pub fpay: String,
    // pub fsongname: String,
    // pub hasecho: String,
    // pub haskdatx: String,
    // #[serde(rename = "iot_info")]
    // pub iot_info: String,
    // #[serde(rename = "is_point")]
    // pub is_point: String,
    // pub isdownload: String,
    // pub isshowtype: String,
    // pub mkvnsig1: String,
    // pub mkvnsig2: String,
    // pub mkvrid: String,
    // pub mp3rid: String,
    // pub mp3sig1: String,
    // pub mp3sig2: String,
    // pub mp4sig1: String,
    // pub mp4sig2: String,
    pub musicrid: String,
    // #[serde(rename = "muti_ver")]
    // pub muti_ver: String,
    // pub mvpayinfo: Mvpayinfo,
    pub name: String,
    // pub nationid: String,
    // pub new: String,
    // pub nsig1: String,
    // pub nsig2: String,
    // pub online: String,
    // pub opay: String,
    // pub originalsongtype: String,
    // #[serde(rename = "overseas_copyright")]
    // pub overseas_copyright: String,
    // #[serde(rename = "overseas_pay")]
    // pub overseas_pay: String,
    // pub pay: String,
    // pub pay_info: PayInfo,
    // pub playcnt: String,
    // pub releasedate: String,
    // pub score100: String,
    // pub sp_privilege: String,
    // pub subs_strategy: String,
    // pub subs_text: String,
    // pub subtitle: String,
    // pub terminal: String,
    // #[serde(rename = "tme_musician_adtype")]
    // pub tme_musician_adtype: String,
    // pub tpay: String,
    // pub uploader: String,
    // pub uptime: String,
    // #[serde(rename = "web_albumpic_short")]
    // pub web_albumpic_short: String,
    // #[serde(rename = "web_artistpic_short")]
    // pub web_artistpic_short: String,
    // #[serde(rename = "web_timingonline")]
    // pub web_timingonline: String,
}

impl Into<crate::server::kuwo::model::Model> for ArtistMusic {
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
                .musicrid
                .strip_prefix("MUSIC_")
                .unwrap_or(&self.musicrid)
                .to_string(),
            artists,
            album: Some(self.album),
            album_id: Some(self.albumid),
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistAlbumResult {
    pub albumlist: Vec<Album>,
    // pub pn: String,
    // #[serde(rename = "return")]
    // pub return_field: String,
    // pub total: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    // #[serde(rename = "PAY")]
    // pub pay: String,
    #[serde(rename = "PLAYCNT")]
    pub playcnt: String,
    pub aartist: String,
    // #[serde(rename = "ad_subtype")]
    // pub ad_subtype: String,
    // #[serde(rename = "ad_type")]
    // pub ad_type: String,
    pub albumid: String,
    pub artist: String,
    pub artistid: String,
    pub artistpic: String,
    // pub color: String,
    // pub company: String,
    // #[serde(rename = "content_type")]
    // pub content_type: String,
    // pub falbum: String,
    // pub fartist: String,
    // pub finished: String,
    pub id: String,
    pub info: Option<String>,
    // pub isstar: String,
    // pub lang: String,
    pub musiccnt: String,
    pub name: String,
    // pub new: String,
    pub pic: String,
    // #[serde(rename = "pub")]
    // pub pub_field: String,
    // pub score: String,
    // pub sp_privilege: String,
    // pub startype: String,
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
            summary: self.info.and_then(|i| Some(decode_html_entities(i))),
            cover: Some(format!("https://img2.kuwo.cn/star/albumcover/{}", self.pic)),
            creator: Some(self.artist),
            creator_id: Some(self.artistid),
            play_time: self.playcnt.parse().ok(),
            music_num: self.musiccnt.parse().ok(),
            subscription: None,
            from_db: false,
            order: None,
            collection_id: None,
        }
    }
}
