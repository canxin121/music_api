use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    interface::{self, playlist::Playlist, quality::Quality, server::MusicServer},
    server::netease,
};

use super::request::eapi_request;

pub async fn get_artist_musics(
    artist_id: &str,
    page: u16,
    limit: u16,
) -> anyhow::Result<Vec<netease::model::Model>> {
    if page < 1 {
        return Err(anyhow::anyhow!("page must be greater than 0"));
    }
    let text = eapi_request(
        "/api/v2/artist/songs",
        &json!({
            "id": artist_id,
            "limit": limit,
            "offset": limit * (page - 1),
        })
        .to_string(),
    )
    .await?;
    let result = serde_json::from_str::<ArtistMusicResult>(&text)?;
    // tokio::fs::write("sample_data/netease/artist_musics.json", text).await?;
    result.songs.into_iter().map(|m| Ok(m.into())).collect()
}

pub async fn get_artist_albums(
    artist_id: &str,
    page: u16,
    limit: u16,
) -> anyhow::Result<Vec<Playlist>> {
    if page < 1 {
        return Err(anyhow::anyhow!("page must be greater than 0"));
    }
    let text = eapi_request(
        &format!("/api/artist/albums/{artist_id}"),
        &json!({
            "limit": limit,
            "offset": limit * (page - 1),
        })
        .to_string(),
    )
    .await?;

    // tokio::fs::write("sample_data/netease/artist_albums.json", text).await?;
    let result = serde_json::from_str::<ArtistAlbumResult>(&text)?;
    result
        .hot_albums
        .into_iter()
        .map(|p| Ok(p.into()))
        .collect()
}

#[cfg(test)]
mod test {
    use crate::server::netease::web_api::artist::{get_artist_albums, get_artist_musics};

    #[tokio::test]
    async fn test_get_artist_musics() {
        let result = get_artist_musics("159300", 1, 9999).await.unwrap();
        println!("{:?}", result);
    }
    #[tokio::test]
    async fn test_get_artist_albums() {
        let result = get_artist_albums("159300", 1, 999).await.unwrap();
        println!("{:?}", result);
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistMusicResult {
    pub songs: Vec<Song>,
    // pub more: bool,
    // pub total: i64,
    // pub code: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    // pub starred: bool,
    // pub popularity: f64,
    // pub starred_num: i64,
    // pub played_num: i64,
    // pub day_plays: i64,
    // pub hear_time: i64,
    // #[serde(rename = "mp3Url")]
    // pub mp3url: String,
    // pub rt_urls: Value,
    // pub mark: i64,
    // pub no_copyright_rcmd: Value,
    // pub origin_cover_type: i64,
    // pub origin_song_simple_data: Value,
    // pub song_jump_info: Value,
    pub artists: Vec<Artist>,
    // pub copyright_id: i64,
    pub album: Album,
    // pub score: i64,
    pub m_music: Option<MMusic>,
    pub l_music: Option<LMusic>,
    // pub audition: Value,
    // pub copy_from: String,
    // pub ringtone: Option<String>,
    // pub disc: String,
    // pub no: i64,
    // pub fee: i64,
    pub h_music: Option<HMusic>,
    // pub comment_thread_id: String,
    // pub mvid: i64,
    // pub b_music: BMusic,
    pub sq_music: Option<SqMusic>,
    pub hr_music: Option<HrMusic>,
    // pub crbt: Value,
    // pub rt_url: Value,
    // pub ftype: i64,
    // pub rtype: i64,
    // pub rurl: Value,
    // pub position: i64,
    pub duration: i64,
    // pub alias: Vec<String>,
    // pub status: i64,
    pub name: String,
    pub id: i64,
    // pub privilege: Privilege,
    // #[serde(default)]
    // pub trans_names: Vec<String>,
}

impl Song {
    pub fn get_qualities(&self) -> Vec<Quality> {
        let mut qualities = Vec::new();
        if let Some(hr) = &self.hr_music {
            qualities.push(Quality {
                summary: "hires".to_string(),
                bitrate: Some(hr.bitrate.to_string()),
                format: Some(hr.extension.to_string()),
                size: Some(hr.size.to_string()),
            });
        }
        if let Some(sq) = &self.sq_music {
            qualities.push(Quality {
                summary: "lossless".to_string(),
                bitrate: Some(sq.bitrate.to_string()),
                format: Some(sq.extension.to_string()),
                size: Some(sq.size.to_string()),
            });
        }
        if let Some(h) = &self.h_music {
            qualities.push(Quality {
                summary: "exhigh".to_string(),
                bitrate: Some(h.bitrate.to_string()),
                format: Some(h.extension.to_string()),
                size: Some(h.size.to_string()),
            });
        }
        if let Some(m) = &self.m_music {
            qualities.push(Quality {
                summary: "higher".to_string(),
                bitrate: Some(m.bitrate.to_string()),
                format: Some(m.extension.to_string()),
                size: Some(m.size.to_string()),
            });
        }
        if let Some(l) = &self.l_music {
            qualities.push(Quality {
                summary: "standard".to_string(),
                bitrate: Some(l.bitrate.to_string()),
                format: Some(l.extension.to_string()),
                size: Some(l.size.to_string()),
            });
        }
        qualities
    }
}

impl Into<netease::model::Model> for Song {
    fn into(self) -> netease::model::Model {
        netease::model::Model {
            qualities: self.get_qualities().into(),
            name: self.name,
            music_id: self.id.to_string(),
            duration: Some(self.duration),
            artists: self
                .artists
                .into_iter()
                .map(|artist| artist.into())
                .collect::<Vec<interface::artist::Artist>>()
                .into(),
            album: Some(self.album.name),
            album_id: Some(self.album.id.to_string()),
            cover: Some(self.album.pic_url),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    // #[serde(rename = "img1v1Id")]
    // pub img1v1id: i64,
    // pub topic_person: i64,
    // pub pic_id: i64,
    // pub brief_desc: String,
    // pub music_size: i64,
    // pub album_size: i64,
    // pub pic_url: String,
    // #[serde(rename = "img1v1Url")]
    // pub img1v1url: String,
    // pub followed: bool,
    // pub trans: String,
    // pub alias: Vec<Value>,
    pub name: String,
    pub id: i64,
    // #[serde(rename = "img1v1Id_str")]
    // pub img1v1id_str: String,
}

impl Into<interface::artist::Artist> for Artist {
    fn into(self) -> interface::artist::Artist {
        interface::artist::Artist {
            name: self.name,
            id: Some(self.id.to_string()),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    // pub songs: Vec<Value>,
    // pub paid: bool,
    // pub on_sale: bool,
    // pub mark: i64,
    // pub award_tags: Value,
    // pub artists: Vec<Artist2>,
    // pub copyright_id: i64,
    // pub pic_id: i64,
    // pub artist: Artist3,
    // pub publish_time: i64,
    // pub company: String,
    // pub brief_desc: String,
    pub pic_url: String,
    // pub comment_thread_id: String,
    // pub blur_pic_url: String,
    // pub company_id: i64,
    // pub pic: i64,
    // pub status: i64,
    // pub sub_type: String,
    // pub alias: Vec<String>,
    // pub description: String,
    // pub tags: String,
    pub name: String,
    pub id: i64,
    // #[serde(rename = "type")]
    // pub type_field: String,
    // pub size: i64,
    // #[serde(rename = "picId_str")]
    // pub pic_id_str: String,
    // #[serde(default)]
    // pub trans_names: Vec<String>,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Artist2 {
//     #[serde(rename = "img1v1Id")]
//     pub img1v1id: i64,
//     pub topic_person: i64,
//     pub pic_id: i64,
//     pub brief_desc: String,
//     pub music_size: i64,
//     pub album_size: i64,
//     pub pic_url: String,
//     #[serde(rename = "img1v1Url")]
//     pub img1v1url: String,
//     pub followed: bool,
//     pub trans: String,
//     pub alias: Vec<Value>,
//     pub name: String,
//     pub id: i64,
//     #[serde(rename = "img1v1Id_str")]
//     pub img1v1id_str: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Artist3 {
//     #[serde(rename = "img1v1Id")]
//     pub img1v1id: i64,
//     pub topic_person: i64,
//     pub pic_id: i64,
//     pub brief_desc: String,
//     pub music_size: i64,
//     pub album_size: i64,
//     pub pic_url: String,
//     #[serde(rename = "img1v1Url")]
//     pub img1v1url: String,
//     pub followed: bool,
//     pub trans: String,
//     pub alias: Vec<Value>,
//     pub name: String,
//     pub id: i64,
//     #[serde(rename = "img1v1Id_str")]
//     pub img1v1id_str: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MMusic {
    // pub volume_delta: f64,
    // pub play_time: i64,
    pub bitrate: i64,
    // pub dfs_id: i64,
    // pub sr: i64,
    pub name: String,
    // pub id: i64,
    pub size: i64,
    pub extension: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LMusic {
    // pub volume_delta: f64,
    // pub play_time: i64,
    pub bitrate: i64,
    // pub dfs_id: i64,
    // pub sr: i64,
    pub name: String,
    // pub id: i64,
    pub size: i64,
    pub extension: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HMusic {
    // pub volume_delta: f64,
    // pub play_time: i64,
    pub bitrate: i64,
    // pub dfs_id: i64,
    // pub sr: i64,
    pub name: String,
    // pub id: i64,
    pub size: i64,
    pub extension: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BMusic {
    // pub volume_delta: f64,
    // pub play_time: i64,
    pub bitrate: i64,
    // pub dfs_id: i64,
    // pub sr: i64,
    pub name: String,
    // pub id: i64,
    pub size: i64,
    pub extension: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqMusic {
    // pub volume_delta: f64,
    // pub play_time: i64,
    pub bitrate: i64,
    // pub dfs_id: i64,
    // pub sr: i64,
    pub name: String,
    // pub id: i64,
    pub size: i64,
    pub extension: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrMusic {
    // pub volume_delta: f64,
    // pub play_time: i64,
    pub bitrate: i64,
    // pub dfs_id: i64,
    // pub sr: i64,
    pub name: String,
    // pub id: i64,
    pub size: i64,
    pub extension: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Privilege {
//     pub id: i64,
//     pub fee: i64,
//     pub payed: i64,
//     pub st: i64,
//     pub pl: i64,
//     pub dl: i64,
//     pub sp: i64,
//     pub cp: i64,
//     pub subp: i64,
//     pub cs: bool,
//     pub maxbr: i64,
//     pub fl: i64,
//     pub toast: bool,
//     pub flag: i64,
//     pub pre_sell: bool,
//     pub play_maxbr: i64,
//     pub download_maxbr: i64,
//     pub max_br_level: String,
//     pub play_max_br_level: String,
//     pub download_max_br_level: String,
//     pub pl_level: String,
//     pub dl_level: String,
//     pub fl_level: String,
//     pub rscl: i64,
//     pub free_trial_privilege: FreeTrialPrivilege,
//     pub right_source: i64,
//     pub charge_info_list: Vec<ChargeInfoList>,
//     pub code: i64,
//     pub message: Value,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct FreeTrialPrivilege {
//     pub res_consumable: bool,
//     pub user_consumable: bool,
//     pub listen_type: Value,
//     pub cannot_listen_reason: i64,
//     pub play_reason: Value,
//     pub free_limit_tag_type: Value,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ChargeInfoList {
//     pub rate: i64,
//     pub charge_url: Value,
//     pub charge_message: Value,
//     pub charge_type: i64,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistAlbumResult {
    // pub artist: Artist,
    pub hot_albums: Vec<HotAlbum>,
    // pub more: bool,
    // pub code: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotAlbum {
    // pub songs: Vec<Value>,
    // pub paid: bool,
    // pub on_sale: bool,
    // pub mark: i64,
    // pub award_tags: Value,
    // pub artists: Vec<Artist2>,
    // pub copyright_id: i64,
    // pub pic_id: i64,
    pub artist: Artist,
    // pub publish_time: i64,
    // pub company: String,
    // pub brief_desc: String,
    pub pic_url: String,
    // pub comment_thread_id: String,
    // pub blur_pic_url: String,
    // pub company_id: i64,
    // pub pic: i64,
    // pub status: i64,
    // pub sub_type: String,
    // pub alias: Vec<String>,
    pub description: String,
    // pub tags: String,
    pub name: String,
    pub id: i64,
    // #[serde(rename = "type")]
    // pub type_field: String,
    // pub size: i64,
    // #[serde(rename = "picId_str")]
    // pub pic_id_str: Option<String>,
    // #[serde(default)]
    // pub trans_names: Vec<String>,
}

impl Into<interface::playlist::Playlist> for HotAlbum {
    fn into(self) -> interface::playlist::Playlist {
        Playlist {
            from_db: false,
            server: Some(MusicServer::Netease),
            type_field: interface::playlist::PlaylistType::Album,
            identity: self.id.to_string(),
            order: None,
            name: self.name,
            summary: Some(self.description),
            cover: Some(self.pic_url),
            creator: Some(self.artist.name),
            creator_id: Some(self.artist.id.to_string()),
            play_time: None,
            music_num: None,
            subscription: None,
        }
    }
}
