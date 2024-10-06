use serde::{Deserialize, Serialize};
use serde_json::json;

use serde_json::Value;

use crate::interface::playlist::Playlist;
use crate::interface::server::MusicServer;
use crate::server::netease::model::Model;
use crate::server::netease::web_api::music_info::get_musics_info;
use crate::server::netease::web_api::{
    encrypt::linux_api,
    request::{search, SearchTarget},
};
use crate::CLIENT;
use anyhow::Result;

// 搜索歌单
pub async fn search_netease_music_list(
    content: &str,
    page: u16,
    limit: u16,
) -> Result<Vec<Playlist>> {
    if page == 0 {
        return Err(anyhow::anyhow!("Page must be greater than 0"));
    }
    let resp = search(SearchTarget::MusicList, content, page, limit).await?;
    // std::fs::write("sample_data/netease/search_music_list.json", &resp)
    //     .expect("Failed to write result to file");
    let result: SearchNeteaseMusiclistResult = serde_json::from_str(&resp)?;
    Ok(result
        .result
        .playlists
        .into_iter()
        .map(|p| p.into())
        .collect())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchNeteaseMusiclistResult {
    pub result: InnerResult,
    // pub code: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerResult {
    // pub search_qc_reminder: Value,
    pub playlists: Vec<NeteasePlaylist>,
    // pub playlist_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteasePlaylist {
    pub id: i64,
    pub name: String,
    pub cover_img_url: Option<String>,
    pub creator: Creator,
    // pub subscribed: bool,
    pub track_count: i64,
    pub user_id: i64,
    pub play_count: i64,
    // pub book_count: i64,
    // pub special_type: i64,
    // pub official_tags: Value,
    // pub action: Value,
    // pub action_type: Value,
    // pub recommend_text: Value,
    // pub score: Value,
    pub description: Option<String>,
    // pub high_quality: bool,
}

impl Into<Playlist> for NeteasePlaylist {
    fn into(self) -> Playlist {
        Playlist {
            from_db: false,
            server: Some(MusicServer::Netease),
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
pub struct Creator {
    pub nickname: String,
    pub user_id: i64,
    pub user_type: i64,
    pub avatar_url: Value,
    pub auth_status: i64,
    pub expert_tags: Option<Vec<String>>,
    pub experts: Value,
}

pub async fn get_musics_from_music_list(
    musiclist_id: &str,
    page: u16,
    limit: u16,
) -> Result<Vec<Model>> {
    assert!(page >= 1, "Page must be greater than 0");
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
    // std::fs::write("sample_data/netease/get_musics_from_music_list.json", &resp).unwrap();
    let result: GetMusicFromMusiclistResult = serde_json::from_str(&resp)?;
    let start = ((page - 1) * limit) as usize;

    let ids: Vec<i64> = result
        .playlist
        .track_ids
        .into_iter()
        .skip(start)
        .map(|t| t.id)
        .take(limit as usize)
        .collect();

    let musics = get_musics_info(&ids).await?;
    Ok(musics)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMusicFromMusiclistResult {
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
    // pub id: i64,
    // pub name: String,
    // pub cover_img_id: i64,
    // pub cover_img_url: String,
    // #[serde(rename = "coverImgId_str")]
    // pub cover_img_id_str: Value,
    // pub ad_type: i64,
    // pub user_id: i64,
    // pub create_time: i64,
    // pub status: i64,
    // pub op_recommend: bool,
    // pub high_quality: bool,
    // pub new_imported: bool,
    // pub update_time: i64,
    // pub track_count: i64,
    // pub special_type: i64,
    // pub privacy: i64,
    // pub track_update_time: i64,
    // pub comment_thread_id: String,
    // pub play_count: i64,
    // pub track_number_update_time: i64,
    // pub subscribed_count: i64,
    // pub cloud_track_count: i64,
    // pub ordered: bool,
    // pub description: String,
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
    // pub creator: Creator,
    // pub tracks: Vec<Value>,
    // pub video_ids: Value,
    // pub videos: Value,
    pub track_ids: Vec<TrackId>,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscriber {
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
    pub avatar_img_id_str2: Option<String>,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Creator {
//     pub default_avatar: bool,
//     pub province: i64,
//     pub auth_status: i64,
//     pub followed: bool,
//     pub avatar_url: String,
//     pub account_status: i64,
//     pub gender: i64,
//     pub city: i64,
//     pub birthday: i64,
//     pub user_id: i64,
//     pub user_type: i64,
//     pub nickname: String,
//     pub signature: String,
//     pub description: String,
//     pub detail_description: String,
//     pub avatar_img_id: i64,
//     pub background_img_id: i64,
//     pub background_url: String,
//     pub authority: i64,
//     pub mutual: bool,
//     pub expert_tags: Value,
//     pub experts: Value,
//     pub dj_status: i64,
//     pub vip_type: i64,
//     pub remark_name: Value,
//     pub authentication_types: i64,
//     pub avatar_detail: Value,
//     pub avatar_img_id_str: String,
//     pub background_img_id_str: String,
//     pub anchor: bool,
//     #[serde(rename = "avatarImgId_str")]
//     pub avatar_img_id_str2: String,
// }

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

#[tokio::test]
async fn search_music_list_test() {
    let content = "张惠妹";
    let page = 1;
    let limit = 30;
    let playlists = search_netease_music_list(content, page, limit)
        .await
        .unwrap();
    println!("{:?}", playlists)
}

#[tokio::test]
async fn get_musics_from_music_list_test() {
    let mut musics = get_musics_from_music_list("12560308462", 1, 10)
        .await
        .unwrap();
    musics.append(
        &mut get_musics_from_music_list("12560308462", 2, 10)
            .await
            .unwrap(),
    );
    musics.append(
        &mut get_musics_from_music_list("12560308462", 3, 10)
            .await
            .unwrap(),
    );
    musics.append(
        &mut get_musics_from_music_list("12560308462", 4, 10)
            .await
            .unwrap(),
    );
    musics.append(
        &mut get_musics_from_music_list("12560308462", 5, 10)
            .await
            .unwrap(),
    );
    println!("{}", musics.len());
}
