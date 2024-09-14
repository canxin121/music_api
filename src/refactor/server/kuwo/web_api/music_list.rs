use anyhow::Result;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::refactor::{
    data::interface::{artist::Artist, playlist::Playlist},
    server::{KuwoMusicModel, CLIENT},
};

use super::utils::{find_id_from_share_url, get_music_rid_pic, parse_qualities_minfo};

pub async fn search_kuwo_music_list(content: &str, page: u32, limit: u32) -> Result<Vec<Playlist>> {
    let url = format!("http://search.kuwo.cn/r.s?all={}&pn={}&rn={limit}&rformat=json&encoding=utf8&ver=mbox&vipver=MUSIC_8.7.7.0_BCS37&plat=pc&devid=28156413&ft=playlist&pay=0&needliveshow=0",encode(content),page-1);
    let text = CLIENT
        .get(&url)
        .send()
        .await?
        .text()
        .await?
        .replace('"', "")
        .replace("'", "\"");
    // std::fs::write("sample_data/kuwo/search_music_list.json", &text).unwrap();
    let result: SearchMusiclistResult = serde_json::from_str(&text)?;
    let playlists = result.abslist.into_iter().map(|p| p.into()).collect();
    Ok(playlists)
}

pub async fn get_kuwo_musics_of_music_list(
    playlist_id: &str,
    page: u16,
    limit: u16,
) -> Result<Vec<KuwoMusicModel>> {
    let url = format!("http://nplserver.kuwo.cn/pl.svc?op=getlistinfo&pid={playlist_id}&pn={}&rn={limit}&encode=utf8&keyset=pl2012&identity=kuwo&pcmp4=1&vipver=MUSIC_9.0.5.0_W1&newver=1",page-1);

    let mut musiclist: GetMusicListResult = CLIENT.get(url).send().await?.json().await?;
    let mut handles = Vec::with_capacity(musiclist.musiclist.len());
    for music in &musiclist.musiclist {
        let id = music.id.clone();
        handles.push(tokio::spawn(async move { get_music_rid_pic(&id).await }))
    }

    for (music, handle) in musiclist.musiclist.iter_mut().zip(handles) {
        music.cover = handle.await?.ok();
    }

    let musiclist = musiclist.musiclist.into_iter().map(|m| m.into()).collect();
    Ok(musiclist)
}

/// SearchMusiclistResult
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMusiclistResult {
    // #[serde(rename = "ARTISTPIC")]
    // pub artistpic: String,
    // #[serde(rename = "HIT")]
    // pub hit: String,
    // #[serde(rename = "HITMODE")]
    // pub hitmode: String,
    // #[serde(rename = "HIT_BUT_OFFLINE")]
    // pub hit_but_offline: String,
    // #[serde(rename = "MSHOW")]
    // pub mshow: String,
    // #[serde(rename = "NEW")]
    // pub new: String,
    // #[serde(rename = "PN")]
    // pub pn: String,
    // #[serde(rename = "RN")]
    // pub rn: String,
    // #[serde(rename = "SHOW")]
    // pub show: String,
    // #[serde(rename = "TOTAL")]
    // pub total: String,
    // #[serde(rename = "UK")]
    // pub uk: String,
    pub abslist: Vec<SearchMusicList>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMusicList {
    // #[serde(rename = "DC_TARGETID")]
    // pub dc_targetid: String,
    // #[serde(rename = "DC_TARGETTYPE")]
    // pub dc_targettype: String,
    // pub hasdeal: String,
    // pub hitcontent: String,
    // #[serde(rename = "hts_pic")]
    // pub hts_pic: String,
    pub intro: String,
    // pub isshow: String,
    pub name: String,
    pub nickname: String,
    pub pic: String,
    pub playcnt: String,
    pub playlistid: String,
    // #[serde(rename = "react_type")]
    // pub react_type: String,
    pub songnum: String,
    // pub tags: String,
}

impl Into<Playlist> for SearchMusicList {
    fn into(self) -> Playlist {
        Playlist {
            server: Some(crate::refactor::data::interface::MusicServer::Kuwo),
            type_field: crate::refactor::data::interface::playlist::PlaylistType::UserPlaylist,
            identity: self.playlistid,
            name: self.name,
            summary: Some(self.intro),
            cover: Some(self.pic),
            creator: Some(self.nickname),
            creator_id: None,
            play_time: self.playcnt.parse().ok(),
            music_num: self.songnum.parse().ok(),
            subscription: None,
            from_db: false,
        }
    }
}

/// GetMusicListResult
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMusicListResult {
    // pub abstime: i64,
    // pub ctime: i64,
    // pub id: i64,
    // pub info: String,
    // pub ispub: bool,
    pub musiclist: Vec<MusiclistMusic>,
    // pub pic: String,
    // pub playnum: i64,
    // pub pn: i64,
    // pub result: String,
    // pub rn: i64,
    // pub sharenum: i64,
    // pub songtime: i64,
    // pub state: i64,
    // pub tag: String,
    // pub tagid: String,
    // pub title: String,
    // pub total: i64,
    // #[serde(rename = "type")]
    // pub type_field: String,
    // pub uid: i64,
    // pub uname: String,
    // pub validtotal: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusiclistMusic {
    #[serde(default)]
    pub cover: Option<String>,
    // #[serde(default)]
    // pub musiclist_pic: String,
    // #[serde(rename = "AARTIST")]
    // pub aartist: String,
    // #[serde(rename = "FALBUM")]
    // pub falbum: String,
    // #[serde(rename = "FARTIST")]
    // pub fartist: String,
    // #[serde(rename = "FSONGNAME")]
    // pub fsongname: String,
    #[serde(rename = "MINFO")]
    pub minfo: String,
    // #[serde(rename = "N_MINFO")]
    // pub n_minfo: String,
    // #[serde(rename = "ad_subtype")]
    // pub ad_subtype: String,
    // #[serde(rename = "ad_type")]
    // pub ad_type: String,
    pub album: String,
    pub albumid: String,
    pub artist: String,
    pub artistid: String,
    // pub audiobookpayinfo: Audiobookpayinfo,
    // pub barrage: String,
    // #[serde(rename = "cache_status")]
    // pub cache_status: String,
    // #[serde(rename = "collect_num")]
    // pub collect_num: String,
    // #[serde(rename = "content_type")]
    // pub content_type: String,
    // pub copyright: String,
    // pub displayalbumname: String,
    // pub displayartistname: String,
    // pub displaysongname: String,
    pub duration: String,
    // pub firstrecordtime: String,
    // pub formats: String,
    // pub hasmv: String,
    pub id: String,
    // #[serde(rename = "is_point")]
    // pub is_point: String,
    // pub isbatch: String,
    // pub isdownload: String,
    // pub isshow: String,
    // pub isshowtype: String,
    // pub isstar: String,
    // pub mp3sig1: String,
    // pub mp3sig2: String,
    // pub mp4sig1: Option<String>,
    // pub mp4sig2: Option<String>,
    // pub musicattachinfoid: String,
    // #[serde(rename = "muti_ver")]
    // pub muti_ver: String,
    // pub mvpayinfo: Mvpayinfo,
    pub name: String,
    // pub nationid: String,
    // pub nsig1: String,
    // pub nsig2: String,
    // pub online: String,
    // pub opay: String,
    // #[serde(rename = "overseas_copyright")]
    // pub overseas_copyright: String,
    // #[serde(rename = "overseas_pay")]
    // pub overseas_pay: String,
    // pub params: String,
    // pub pay: String,
    // pub pay_info: PayInfo,
    // pub score100: String,
    // pub sp_privilege: String,
    // pub subs_strategy: String,
    // pub subs_text: String,
    // #[serde(rename = "tme_musician_adtype")]
    // pub tme_musician_adtype: String,
    // pub tpay: String,
}

impl Into<crate::refactor::server::kuwo::model::Model> for MusiclistMusic {
    fn into(self) -> crate::refactor::server::kuwo::model::Model {
        let artist_names = self.artist.split("&").collect::<Vec<&str>>();
        let artist_ids = self.artistid.split("&").collect::<Vec<&str>>();

        let artists: Vec<Artist> = artist_names
            .into_iter()
            .zip(artist_ids.into_iter().chain(std::iter::repeat("")))
            .map(
                |(name, id)| crate::refactor::data::interface::artist::Artist {
                    name: name.to_string(),
                    id: id.parse().ok(),
                },
            )
            .collect();
        let artists = crate::refactor::data::interface::artist::ArtistVec::from(artists);

        crate::refactor::server::kuwo::model::Model {
            name: self.name,
            artists,
            music_id: self.id,
            album: Some(self.album),
            album_id: Some(self.albumid),
            qualities: parse_qualities_minfo(&self.minfo).into(),
            cover: self.cover,
            duration: self.duration.parse().ok(),
        }
    }
}
