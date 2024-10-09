use serde::{Deserialize, Serialize};
use serde_json::json;

use serde_json::Value;

use crate::interface::playlist::Playlist;
use crate::interface::server::MusicServer;
use crate::server::netease::web_api::encrypt::linux_api;
use crate::CLIENT;
use anyhow::Result;

use super::utils::find_netease_playlist_id_from_share;

pub async fn get_netease_music_list_from_share(share: &str) -> Result<Playlist> {
    let musiclist_id =
        find_netease_playlist_id_from_share(share).ok_or(anyhow::anyhow!("No id found"))?;

    let data = json!({
      "method": "POST",
      "url": "https://music.163.com/api/v3/playlist/detail",
      "params": {
        "id": musiclist_id,
        "n": 0,
        "s": 8,
      },
    })
    .to_string();

    let resp = CLIENT
        .post("https://music.163.com/api/linux/forward")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36")
        .header("Cookie", "MUSIC_U=")
        .form(&linux_api(&data)).send().await;

    let resp = resp?.text().await?;
    // std::fs::write("sample_data/netease/share.json", &resp).unwrap();
    let result: GetPlaylistFromShareResult = serde_json::from_str(&resp)?;

    Ok(result.playlist.into())
}

#[tokio::test]
async fn test_get_netease_music_list_from_share() {
    let share = "https://music.163.com/playlist?id=12497815913&uct2=U2FsdGVkX19tzJpiufgwqfBqjgNRIDask6O0auKK8SQ=";
    let playlist = get_netease_music_list_from_share(share).await.unwrap();
    println!("{:?}", playlist);
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPlaylistFromShareResult {
    // pub code: i64,
    // pub related_videos: Value,
    pub playlist: GetMusicInnerPlaylist,
    // pub urls: Value,
    // pub privileges: Vec<Value>,
    // pub shared_privilege: Value,
    // pub res_entrance: Value,
    // pub from_users: Value,
    // pub from_user_count: i64,
    // pub song_from_users: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMusicInnerPlaylist {
    pub id: i64,
    pub name: String,
    // pub cover_img_id: i64,
    pub cover_img_url: Option<String>,
    // #[serde(rename = "coverImgId_str")]
    // pub cover_img_id_str: Value,
    // pub ad_type: i64,
    pub user_id: i64,
    // pub create_time: i64,
    // pub status: i64,
    // pub op_recommend: bool,
    // pub high_quality: bool,
    // pub new_imported: bool,
    // pub update_time: i64,
    pub track_count: i64,
    // pub special_type: i64,
    // pub privacy: i64,
    // pub track_update_time: i64,
    // pub comment_thread_id: String,
    pub play_count: i64,
    // pub track_number_update_time: i64,
    // pub subscribed_count: i64,
    // pub cloud_track_count: i64,
    // pub ordered: bool,
    pub description: Option<String>,
    // pub tags: Vec<String>,
    // pub update_frequency: Value,
    // pub background_cover_id: i64,
    // pub background_cover_url: Value,
    // pub title_image: i64,
    // pub title_image_url: Value,
    // pub detail_page_title: Value,
    // pub english_title: Value,
    // pub official_playlist_type: Value,
    // pub copied: bool,
    // pub relate_res_type: Value,
    // pub cover_status: i64,
    // pub subscribers: Vec<Subscriber>,
    // pub subscribed: Value,
    pub creator: Creator,
    // pub tracks: Vec<Value>,
    // pub video_ids: Value,
    // pub videos: Value,
    // pub track_ids: Vec<TrackId>,
    // pub banned_track_ids: Value,
    // pub mv_resource_infos: Value,
    // pub share_count: i64,
    // pub comment_count: i64,
    // pub remix_video: Value,
    // pub new_detail_page_remix_video: Value,
    // pub shared_users: Value,
    // pub history_shared_users: Value,
    // pub grade_status: String,
    // pub score: Value,
    // pub alg_tags: Vec<Value>,
    // pub distribute_tags: Vec<Value>,
    // pub trial_mode: i64,
    // pub display_tags: Value,
    // pub display_user_info_as_tag_only: bool,
    // pub playlist_type: String,
}

impl Into<Playlist> for GetMusicInnerPlaylist {
    fn into(self) -> Playlist {
        Playlist {
            from_db: false,
            server: Some(MusicServer::Netease),
            collection_id: None,
            type_field: crate::interface::playlist::PlaylistType::UserPlaylist,
            identity: self.id.to_string(),
            name: self.name,
            summary: self.description,
            cover: self.cover_img_url,
            creator: Some(self.creator.nickname),
            creator_id: Some(self.creator.user_id.to_string()),
            play_time: Some(self.play_count),
            music_num: Some(self.track_count),
            subscription: None,
            order: None,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscriber {
    // pub default_avatar: bool,
    // pub province: i64,
    // pub auth_status: i64,
    // pub followed: bool,
    // pub avatar_url: String,
    // pub account_status: i64,
    // pub gender: i64,
    // pub city: i64,
    // pub birthday: i64,
    pub user_id: i64,
    // pub user_type: i64,
    pub nickname: String,
    // pub signature: String,
    // pub description: String,
    // pub detail_description: String,
    // pub avatar_img_id: i64,
    // pub background_img_id: i64,
    // pub background_url: String,
    // pub authority: i64,
    // pub mutual: bool,
    // pub expert_tags: Value,
    // pub experts: Value,
    // pub dj_status: i64,
    // pub vip_type: i64,
    // pub remark_name: Value,
    // pub authentication_types: i64,
    // pub avatar_detail: Value,
    // pub avatar_img_id_str: String,
    // pub background_img_id_str: String,
    // pub anchor: bool,
    // #[serde(rename = "avatarImgId_str")]
    // pub avatar_img_id_str2: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub default_avatar: bool,
    pub province: i64,
    pub auth_status: i64,
    pub followed: bool,
    pub avatar_url: String,
    pub account_status: i64,
    pub gender: i64,
    pub city: i64,
    pub birthday: i64,
    pub user_id: i64,
    pub user_type: i64,
    pub nickname: String,
    pub signature: String,
    pub description: String,
    pub detail_description: String,
    pub avatar_img_id: i64,
    pub background_img_id: i64,
    pub background_url: String,
    pub authority: i64,
    pub mutual: bool,
    pub expert_tags: Value,
    pub experts: Value,
    pub dj_status: i64,
    pub vip_type: i64,
    pub remark_name: Value,
    pub authentication_types: i64,
    pub avatar_detail: Value,
    pub avatar_img_id_str: String,
    pub background_img_id_str: String,
    pub anchor: bool,
    #[serde(rename = "avatarImgId_str")]
    pub avatar_img_id_str2: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackId {
    pub id: i64,
    // pub v: i64,
    // pub t: i64,
    // pub at: i64,
    // pub alg: Value,
    // pub uid: i64,
    // pub rcmd_reason: String,
    // pub sc: Value,
    // pub f: Value,
    // pub sr: Value,
    // pub dpr: Value,
}
