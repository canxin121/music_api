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

impl Into<ServerMusicChartCollection> for NeteaseMusicChartCollectionResult {
    fn into(self) -> ServerMusicChartCollection {
        let mut charts: Vec<Option<MusicChart>> = self
            .list
            .into_iter()
            .map(|chart| Some(chart.into()))
            .collect();
        let official_charts = charts
            .iter_mut()
            .filter(|c| {
                c.is_some()
                    && ["飙升榜", "新歌榜", "热歌榜", "原创榜"]
                        .contains(&c.as_ref().unwrap().name.as_str())
            })
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let official_charts = MusicChartCollection {
            name: "官方榜".to_string(),
            summary: None,
            charts: official_charts,
        };
        let selected_charts = charts
            .iter_mut()
            .filter(|c| {
                c.is_some()
                    && (["实时热度榜", "赏音榜", "网络热歌榜", "蛋仔派对听歌榜"]
                        .contains(&c.as_ref().unwrap().name.as_str())
                        || ["星云", "黑胶"]
                            .iter()
                            .any(|k| c.as_ref().unwrap().name.starts_with(k)))
            })
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let selected_charts = MusicChartCollection {
            name: "精选榜".to_string(),
            summary: None,
            charts: selected_charts,
        };
        let genre_charts = charts
            .iter_mut()
            .filter(|c| {
                c.is_some()
                    && ([
                        "云音乐电音榜",
                        "欧美R&B榜",
                        "云音乐说唱榜",
                        "云音乐ACG榜",
                        "云音乐摇滚榜",
                        "云音乐民谣榜",
                        "云音乐古典榜",
                        "云音乐国风榜",
                        "中文DJ榜",
                    ]
                    .contains(&c.as_ref().unwrap().name.as_str()))
            })
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let genre_charts = MusicChartCollection {
            name: "曲风榜".to_string(),
            summary: None,
            charts: genre_charts,
        };
        let global_charts = charts
            .iter_mut()
            .filter(|c| {
                c.is_some()
                    && ([
                        "美国Billboard榜",
                        "UK排行榜周榜",
                        "日本Oricon榜",
                        "法国 NRJ Vos Hits 周榜",
                        "俄罗斯top hit流行音乐榜",
                        "Beatport全球电子舞曲榜",
                    ]
                    .contains(&c.as_ref().unwrap().name.as_str()))
            })
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let global_charts = MusicChartCollection {
            name: "全球榜".to_string(),
            summary: None,
            charts: global_charts,
        };
        let language_charts = charts
            .iter_mut()
            .filter(|c| {
                c.is_some()
                    && (["云音乐欧美热歌榜", "云音乐欧美新歌榜"]
                        .contains(&c.as_ref().unwrap().name.as_str())
                        || ["语榜"]
                            .iter()
                            .any(|k| c.as_ref().unwrap().name.ends_with(k)))
            })
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let language_charts = MusicChartCollection {
            name: "语言榜".to_string(),
            summary: None,
            charts: language_charts,
        };
        let special_charts = charts
            .iter_mut()
            .filter(|c| {
                c.is_some()
                    && ([
                        "LOOK直播歌曲榜",
                        "BEAT排行榜",
                        "听歌识曲榜",
                        "潜力爆款榜",
                        "KTV唛榜",
                        "Suno AI新歌榜",
                    ]
                    .contains(&c.as_ref().unwrap().name.as_str()))
            })
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let special_charts = MusicChartCollection {
            name: "特色榜".to_string(),
            summary: None,
            charts: special_charts,
        };
        let car_charts = charts
            .iter_mut()
            .filter(|c| c.is_some() && c.as_ref().unwrap().name.ends_with("车友爱听榜"))
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let car_charts = MusicChartCollection {
            name: "车主榜".to_string(),
            summary: None,
            charts: car_charts,
        };
        let acg_charts = charts
            .iter_mut()
            .filter(|c| c.is_some() && c.as_ref().unwrap().name.contains("ACG"))
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let acg_charts = MusicChartCollection {
            name: "ACG榜".to_string(),
            summary: None,
            charts: acg_charts,
        };
        let other_charts = charts
            .iter_mut()
            .filter(|c| c.is_some())
            .map(|c| c.take().unwrap())
            .collect::<Vec<MusicChart>>();
        let other_charts = MusicChartCollection {
            name: "其他榜".to_string(),
            summary: None,
            charts: other_charts,
        };
        let collections = vec![
            official_charts,
            selected_charts,
            genre_charts,
            global_charts,
            language_charts,
            acg_charts,
            special_charts,
            car_charts,
            other_charts,
        ];

        ServerMusicChartCollection {
            collections,
            server: crate::interface::server::MusicServer::Netease,
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
