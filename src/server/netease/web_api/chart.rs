use crate::{
    interface::music_chart::{MusicChart, MusicChartCollection, ServerMusicChartCollection},
    server::netease::{self, web_api::encrypt::weapi},
    CLIENT,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::music::NeteaseMusic;

pub async fn get_music_chart_collection() -> anyhow::Result<ServerMusicChartCollection> {
    Ok(CLIENT
        .post("https://music.163.com/api/toplist")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36")
        .header("origin", "https://music.163.com")
        // .form(&weapi(&json!({}).to_string())?)
        .send()
        .await?.json::<NeteaseMusicChartCollectionResult>().await?.into())
}

pub async fn get_musics_from_chart(
    id: &str,
    page: u16,
    _size: u16,
) -> anyhow::Result<Vec<netease::model::Model>> {
    if page == 0 {
        return Err(anyhow::anyhow!("page must be greater than 0"));
    }

    if page > 1 {
        return Ok(vec![]);
    }

    // in test, the 'p' has no effect, the result is always the same
    let result:NeteaseMusicChartMusicResult = CLIENT.post("https://music.163.com/weapi/v3/playlist/detail")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36")
            .header("origin", "https://music.163.com")
            .form(&weapi(&json!({
                "id": id,
                "n": 10000,
                "p": page,
            }).to_string())?).send().await?.json().await?;

    result
        .playlist
        .tracks
        .into_iter()
        .map(|m| Ok(m.into()))
        .collect()
}

#[cfg(test)]
mod test {
    use crate::server::netease::web_api::chart::get_music_chart_collection;

    #[tokio::test]
    async fn test_get_music_chart_collection() {
        let result = get_music_chart_collection().await.unwrap();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_get_musics_from_chart() {
        let result = super::get_musics_from_chart("19723756", 1, 100)
            .await
            .unwrap();
        println!("{:?}", result);
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseMusicChartCollectionResult {
    // pub code: i64,
    pub list: Vec<NeteaseMusicChart>,
    // pub artist_toplist: ArtistToplist,
}

impl Into<MusicChartCollection> for NeteaseMusicChartCollectionResult {
    fn into(self) -> MusicChartCollection {
        MusicChartCollection {
            name: "网易云榜单".to_string(),
            summary: None,
            charts: self.list.into_iter().map(|chart| chart.into()).collect(),
        }
    }
}

impl Into<ServerMusicChartCollection> for NeteaseMusicChartCollectionResult {
    fn into(self) -> ServerMusicChartCollection {
        ServerMusicChartCollection {
            server: crate::interface::server::MusicServer::Netease,
            collections: vec![self.into()],
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseMusicChart {
    // pub subscribers: Vec<Value>,
    // pub subscribed: Value,
    // pub creator: Value,
    // pub artists: Value,
    // pub tracks: Value,
    // pub update_frequency: String,
    // pub background_cover_id: i64,
    // pub background_cover_url: Value,
    // pub title_image: i64,
    // pub cover_text: Value,
    // pub title_image_url: Value,
    // pub cover_image_url: Value,
    // pub icon_image_url: Value,
    // pub english_title: Value,
    // pub op_recommend: bool,
    // pub recommend_info: Value,
    // pub social_playlist_cover: Value,
    // pub ts_song_count: i64,
    // pub alg_type: Value,
    // pub track_number_update_time: i64,
    // pub track_update_time: i64,
    // pub privacy: i64,
    // pub high_quality: bool,
    // pub special_type: i64,
    // pub cover_img_id: i64,
    // pub update_time: i64,
    // pub new_imported: bool,
    // pub anonimous: bool,
    pub cover_img_url: String,
    // pub track_count: i64,
    // pub comment_thread_id: String,
    // pub total_duration: i64,
    // pub play_count: i64,
    // pub ad_type: i64,
    // pub subscribed_count: i64,
    // pub cloud_track_count: i64,
    // pub create_time: i64,
    // pub ordered: bool,
    pub description: Option<String>,
    // pub status: i64,
    // pub tags: Vec<String>,
    // pub user_id: i64,
    pub name: String,
    pub id: i64,
    // #[serde(rename = "coverImgId_str")]
    // pub cover_img_id_str: String,
    // #[serde(rename = "ToplistType")]
    // pub toplist_type: Option<String>,
}

impl Into<MusicChart> for NeteaseMusicChart {
    fn into(self) -> MusicChart {
        MusicChart {
            name: self.name,
            summary: self.description,
            cover: Some(self.cover_img_url),
            id: self.id.to_string(),
        }
    }
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ArtistToplist {
//     pub cover_url: String,
//     pub name: String,
//     pub upate_frequency: String,
//     pub position: i64,
//     pub update_frequency: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseMusicChartMusicResult {
    // pub code: i64,
    // pub related_videos: Value,
    pub playlist: Playlist,
    // pub urls: Vec<Url>,
    // pub privileges: Vec<Privilege>,
    // pub shared_privilege: Value,
    // pub res_entrance: Value,
    // pub from_users: Value,
    // pub from_user_count: i64,
    // pub song_from_users: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    // pub id: i64,
    // pub name: String,
    // pub cover_img_id: i64,
    // pub cover_img_url: String,
    // #[serde(rename = "coverImgId_str")]
    // pub cover_img_id_str: String,
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
    // pub tags: Vec<Value>,
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
    // pub subscribers: Vec<Value>,
    // pub subscribed: Value,
    // pub creator: Creator,
    pub tracks: Vec<NeteaseMusic>,
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
    // pub alg_tags: Value,
    // pub distribute_tags: Vec<Value>,
    // pub trial_mode: i64,
    // pub display_tags: Value,
    // pub display_user_info_as_tag_only: bool,
    // pub playlist_type: String,
    // pub biz_ext_info: BizExtInfo,
    // #[serde(rename = "ToplistType")]
    // pub toplist_type: String,
}
