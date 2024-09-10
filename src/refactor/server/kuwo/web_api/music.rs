use crate::refactor::server::CLIENT;

use serde::{Deserialize, Serialize};

use super::utils::{
    build_music_rid_pic, build_web_albumpic_short, build_web_artistpic_short, parse_qualities_minfo,
};

pub async fn search_kuwo_musics(
    content: &str,
    page: u32,
    limit: u32,
) -> Result<KuwoMusics, anyhow::Error> {
    assert!(page >= 1, "Page must be greater than 0");

    let url = format!("http://search.kuwo.cn/r.s?client=kt&all={}&pn={}&rn={}&uid=794762570&ver=kwplayer_ar_9.2.2.1&vipver=1&show_copyright_off=1&newver=1&ft=music&cluster=0&strategy=2012&encoding=utf8&rformat=json&vermerge=1&mobi=1&issubtitle=1",urlencoding::encode(content),page-1,limit);

    let result: KuwoMusics = CLIENT.get(&url).send().await?.json().await?;
    Ok(result)
}

#[tokio::test]
async fn test_search_single_music() {
    let musics = search_kuwo_musics("张惠妹", 1, 30).await.unwrap().abslist;

    musics.iter().for_each(|m| println!("{:?}", m));
    println!("length:{}", musics.len())
}

// generated with https://transform.tools/json-to-rust-serde
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuwoMusics {
    #[serde(rename = "ARTISTPIC")]
    pub artistpic: String,
    #[serde(rename = "HIT")]
    pub hit: String,
    #[serde(rename = "HITMODE")]
    pub hitmode: String,
    #[serde(rename = "HIT_BUT_OFFLINE")]
    pub hit_but_offline: String,
    #[serde(rename = "MSHOW")]
    pub mshow: String,
    #[serde(rename = "NEW")]
    pub new: String,
    #[serde(rename = "PN")]
    pub pn: String,
    #[serde(rename = "RN")]
    pub rn: String,
    #[serde(rename = "SHOW")]
    pub show: String,
    #[serde(rename = "TOTAL")]
    pub total: String,
    #[serde(rename = "UK")]
    pub uk: String,
    pub abslist: Vec<SearchMusic>,
    pub searchgroup: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMusic {
    #[serde(rename = "AARTIST")]
    pub aartist: String,
    #[serde(rename = "ALBUM")]
    pub album: String,
    #[serde(rename = "ALBUMID")]
    pub albumid: String,
    #[serde(rename = "ALIAS")]
    pub alias: String,
    #[serde(rename = "ARTIST")]
    pub artist: String,
    #[serde(rename = "ARTISTID")]
    pub artistid: String,
    #[serde(rename = "CanSetRing")]
    pub can_set_ring: String,
    #[serde(rename = "CanSetRingback")]
    pub can_set_ringback: String,
    #[serde(rename = "DC_TARGETID")]
    pub dc_targetid: String,
    #[serde(rename = "DC_TARGETTYPE")]
    pub dc_targettype: String,
    #[serde(rename = "DURATION")]
    pub duration: String,
    #[serde(rename = "FARTIST")]
    pub fartist: String,
    #[serde(rename = "FORMAT")]
    pub format: String,
    #[serde(rename = "FSONGNAME")]
    pub fsongname: String,
    #[serde(rename = "KMARK")]
    pub kmark: String,
    #[serde(rename = "MINFO")]
    pub minfo: String,
    #[serde(rename = "MUSICRID")]
    pub musicrid: String,
    #[serde(rename = "MVFLAG")]
    pub mvflag: String,
    #[serde(rename = "MVPIC")]
    pub mvpic: String,
    #[serde(rename = "MVQUALITY")]
    pub mvquality: String,
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "NEW")]
    pub new: String,
    #[serde(rename = "N_MINFO")]
    pub n_minfo: String,
    #[serde(rename = "ONLINE")]
    pub online: String,
    #[serde(rename = "PAY")]
    pub pay: String,
    #[serde(rename = "PROVIDER")]
    pub provider: String,
    #[serde(rename = "SONGNAME")]
    pub songname: String,
    #[serde(rename = "SUBLIST")]
    pub sublist: SublistVec,
    #[serde(rename = "SUBTITLE")]
    pub subtitle: String,
    #[serde(rename = "TAG")]
    pub tag: String,
    #[serde(rename = "ad_subtype")]
    pub ad_subtype: String,
    #[serde(rename = "ad_type")]
    pub ad_type: String,
    pub allartistid: String,
    pub audiobookpayinfo: Audiobookpayinfo2,
    pub barrage: String,
    #[serde(rename = "cache_status")]
    pub cache_status: String,
    #[serde(rename = "content_type")]
    pub content_type: String,
    pub fpay: String,
    #[serde(rename = "hts_MVPIC")]
    pub hts_mvpic: Option<String>,
    pub info: String,
    #[serde(rename = "iot_info")]
    pub iot_info: String,
    pub isdownload: String,
    pub isshowtype: String,
    pub isstar: String,
    pub mvpayinfo: Mvpayinfo2,
    pub nationid: String,
    pub opay: String,
    pub originalsongtype: String,
    #[serde(rename = "overseas_copyright")]
    pub overseas_copyright: String,
    #[serde(rename = "overseas_pay")]
    pub overseas_pay: String,
    pub pay_info: PayInfo2,
    #[serde(rename = "react_type")]
    pub react_type: String,
    pub sp_privilege: String,
    pub subs_strategy: String,
    pub subs_text: String,
    pub terminal: String,
    #[serde(rename = "tme_musician_adtype")]
    pub tme_musician_adtype: String,
    pub tpay: String,
    #[serde(rename = "web_albumpic_short")]
    pub web_albumpic_short: String,
    #[serde(rename = "web_artistpic_short")]
    pub web_artistpic_short: String,
    #[serde(rename = "web_timingonline")]
    pub web_timingonline: String,
}

impl Into<crate::refactor::server::kuwo::model::Model> for SearchMusic {
    fn into(self) -> crate::refactor::server::kuwo::model::Model {
        let musicrid_pic = build_music_rid_pic(&self.musicrid);
        crate::refactor::server::kuwo::model::Model {
            name: self.name,
            music_id: self.musicrid,
            artist: self.artist,
            artist_id: self.artistid,
            album: Some(self.album),
            album_id: Some(self.albumid),
            qualities: parse_qualities_minfo(&self.minfo).into(),
            music_pic: musicrid_pic,
            artist_pic: build_web_artistpic_short(&self.web_artistpic_short),
            album_pic: build_web_albumpic_short(&self.web_albumpic_short),
            // mv_vid: if self.mvpayinfo.vid.is_empty() {
            //     None
            // } else {
            //     Some(self.mvpayinfo.vid)
            // },
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SublistVec(pub Vec<Sublist>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sublist {
    #[serde(rename = "AARTIST")]
    pub aartist: String,
    #[serde(rename = "ALBUM")]
    pub album: String,
    #[serde(rename = "ALBUMID")]
    pub albumid: String,
    #[serde(rename = "ALIAS")]
    pub alias: String,
    #[serde(rename = "ARTIST")]
    pub artist: String,
    #[serde(rename = "ARTISTID")]
    pub artistid: String,
    #[serde(rename = "CanSetRing")]
    pub can_set_ring: String,
    #[serde(rename = "CanSetRingback")]
    pub can_set_ringback: String,
    #[serde(rename = "DC_TARGETID")]
    pub dc_targetid: String,
    #[serde(rename = "DC_TARGETTYPE")]
    pub dc_targettype: String,
    #[serde(rename = "DURATION")]
    pub duration: String,
    #[serde(rename = "FARTIST")]
    pub fartist: String,
    #[serde(rename = "FORMAT")]
    pub format: String,
    #[serde(rename = "FSONGNAME")]
    pub fsongname: String,
    #[serde(rename = "KMARK")]
    pub kmark: String,
    #[serde(rename = "MINFO")]
    pub minfo: String,
    #[serde(rename = "MUSICRID")]
    pub musicrid: String,
    #[serde(rename = "MVFLAG")]
    pub mvflag: String,
    #[serde(rename = "MVPIC")]
    pub mvpic: String,
    #[serde(rename = "MVQUALITY")]
    pub mvquality: String,
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "NEW")]
    pub new: String,
    #[serde(rename = "N_MINFO")]
    pub n_minfo: String,
    #[serde(rename = "ONLINE")]
    pub online: String,
    #[serde(rename = "PAY")]
    pub pay: String,
    #[serde(rename = "PROVIDER")]
    pub provider: String,
    #[serde(rename = "SONGNAME")]
    pub songname: String,
    #[serde(rename = "SUBTITLE")]
    pub subtitle: String,
    #[serde(rename = "TAG")]
    pub tag: String,
    #[serde(rename = "ad_subtype")]
    pub ad_subtype: String,
    #[serde(rename = "ad_type")]
    pub ad_type: String,
    pub allartistid: String,
    pub audiobookpayinfo: Audiobookpayinfo,
    pub barrage: String,
    #[serde(rename = "cache_status")]
    pub cache_status: String,
    #[serde(rename = "content_type")]
    pub content_type: String,
    pub fpay: String,
    pub info: String,
    #[serde(rename = "iot_info")]
    pub iot_info: String,
    pub isdownload: String,
    pub isshowtype: String,
    pub isstar: String,
    pub mvpayinfo: Mvpayinfo,
    pub nationid: String,
    pub opay: String,
    pub originalsongtype: String,
    #[serde(rename = "overseas_copyright")]
    pub overseas_copyright: String,
    #[serde(rename = "overseas_pay")]
    pub overseas_pay: String,
    pub pay_info: PayInfo,
    #[serde(rename = "react_type")]
    pub react_type: String,
    pub sp_privilege: String,
    pub subs_strategy: String,
    pub subs_text: String,
    pub terminal: String,
    #[serde(rename = "tme_musician_adtype")]
    pub tme_musician_adtype: String,
    pub tpay: String,
    #[serde(rename = "web_albumpic_short")]
    pub web_albumpic_short: String,
    #[serde(rename = "web_artistpic_short")]
    pub web_artistpic_short: String,
    #[serde(rename = "web_timingonline")]
    pub web_timingonline: String,
    #[serde(rename = "hts_MVPIC")]
    pub hts_mvpic: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audiobookpayinfo {
    pub download: String,
    pub play: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mvpayinfo {
    pub download: String,
    pub play: String,
    pub vid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayInfo {
    pub cannot_download: String,
    pub cannot_online_play: String,
    pub download: String,
    pub fee_type: FeeType,
    pub limitfree: String,
    #[serde(rename = "listen_fragment")]
    pub listen_fragment: String,
    #[serde(rename = "local_encrypt")]
    pub local_encrypt: String,
    pub ndown: String,
    pub nplay: String,
    #[serde(rename = "overseas_ndown")]
    pub overseas_ndown: String,
    #[serde(rename = "overseas_nplay")]
    pub overseas_nplay: String,
    pub paytagindex: Paytagindex,
    pub play: String,
    #[serde(rename = "refrain_end")]
    pub refrain_end: String,
    #[serde(rename = "refrain_start")]
    pub refrain_start: String,
    #[serde(rename = "tips_intercept")]
    pub tips_intercept: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeType {
    pub album: String,
    pub bookvip: String,
    pub song: String,
    pub vip: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paytagindex {
    #[serde(rename = "AR501")]
    pub ar501: i64,
    #[serde(rename = "DB")]
    pub db: i64,
    #[serde(rename = "F")]
    pub f: i64,
    #[serde(rename = "H")]
    pub h: i64,
    #[serde(rename = "HR")]
    pub hr: i64,
    #[serde(rename = "L")]
    pub l: i64,
    #[serde(rename = "S")]
    pub s: i64,
    #[serde(rename = "ZP")]
    pub zp: i64,
    #[serde(rename = "ZPGA201")]
    pub zpga201: i64,
    #[serde(rename = "ZPGA501")]
    pub zpga501: i64,
    #[serde(rename = "ZPLY")]
    pub zply: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audiobookpayinfo2 {
    pub download: String,
    pub play: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mvpayinfo2 {
    pub download: String,
    pub play: String,
    pub vid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayInfo2 {
    pub cannot_download: String,
    pub cannot_online_play: String,
    pub download: String,
    pub fee_type: FeeType2,
    pub limitfree: String,
    #[serde(rename = "listen_fragment")]
    pub listen_fragment: String,
    #[serde(rename = "local_encrypt")]
    pub local_encrypt: String,
    pub ndown: String,
    pub nplay: String,
    #[serde(rename = "overseas_ndown")]
    pub overseas_ndown: String,
    #[serde(rename = "overseas_nplay")]
    pub overseas_nplay: String,
    pub paytagindex: Paytagindex2,
    pub play: String,
    #[serde(rename = "refrain_end")]
    pub refrain_end: String,
    #[serde(rename = "refrain_start")]
    pub refrain_start: String,
    #[serde(rename = "tips_intercept")]
    pub tips_intercept: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeType2 {
    pub album: String,
    pub bookvip: String,
    pub song: String,
    pub vip: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paytagindex2 {
    #[serde(rename = "AR501")]
    pub ar501: i64,
    #[serde(rename = "DB")]
    pub db: i64,
    #[serde(rename = "F")]
    pub f: i64,
    #[serde(rename = "H")]
    pub h: i64,
    #[serde(rename = "HR")]
    pub hr: i64,
    #[serde(rename = "L")]
    pub l: i64,
    #[serde(rename = "S")]
    pub s: i64,
    #[serde(rename = "ZP")]
    pub zp: i64,
    #[serde(rename = "ZPGA201")]
    pub zpga201: i64,
    #[serde(rename = "ZPGA501")]
    pub zpga501: i64,
    #[serde(rename = "ZPLY")]
    pub zply: i64,
}
