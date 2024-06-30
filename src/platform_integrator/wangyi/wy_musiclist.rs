#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    music_aggregator::MusicAggregator,
    music_list::{ExtraInfo, MusicList, MusicListTrait},
    platform_integrator::wangyi::{
        encrypt::linux_api,
        wy_music_detail::get_musics_detail,
        wy_search::{self, SearchTarget},
    },
    util::CLIENT,
    MusicListInfo,
};

use super::WANGYI;

#[derive(Serialize, Deserialize, Debug)]
pub struct WyPlaylist {
    pub id: u64,
    pub name: String,
    pub coverImgUrl: String,
    pub creator: Creator,
    pub subscribed: bool,
    pub trackCount: u32,
    pub userId: u64,
    pub playCount: u32,
    pub bookCount: u64,
    pub specialType: u32,
    pub description: Option<String>,
    pub highQuality: bool,
    // pub officialTags: Option<serde_json::Value>, // Assuming this can be any JSON value
    // pub action: Option<serde_json::Value>,       // Assuming this can be any JSON value
    // pub actionType: Option<serde_json::Value>,   // Assuming this can be any JSON value
    // pub recommendText: Option<serde_json::Value>, // Assuming this can be any JSON value
    // pub score: Option<serde_json::Value>,        // Assuming this can be any JSON value
}
impl MusicListTrait for WyPlaylist {
    fn get_musiclist_info(&self) -> MusicListInfo {
        MusicListInfo {
            name: self.name.clone(),
            art_pic: self.coverImgUrl.clone(),
            desc: self.description.clone().unwrap_or_default(),
            extra: Some(ExtraInfo {
                play_count: Some(self.playCount),
                music_count: Some(self.trackCount),
            }),
        }
    }

    fn get_music_aggregators(
        &self,
        page: u32,
        limit: u32,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                Output = Result<Vec<crate::music_aggregator::MusicAggregator>, anyhow::Error>,
            >,
        >,
    > {
        let musiclist_id = self.id.clone();
        Box::pin(async move {
            Ok(get_musics_from_music_list(musiclist_id, page, limit)
                .await?
                .1)
        })
    }

    fn source(&self) -> String {
        WANGYI.to_string()
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ResultData {
    pub searchQcReminder: Option<serde_json::Value>, // Assuming this can be any JSON value
    pub playlists: Vec<WyPlaylist>,
    pub playlistCount: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Creator {
    #[serde(default)]
    pub nickname: String,
    pub userId: u64,
    pub userType: u32,
    pub authStatus: u32,
    // pub avatarUrl: Option<serde_json::Value>, // Assuming this can be any JSON value
    // pub expertTags: Option<serde_json::Value>, // Assuming this can be any JSON value
    // pub experts: Option<serde_json::Value>,    // Assuming this can be any JSON value
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MusicListSearchResponse {
    pub result: ResultData,
    pub code: u32,
}

// 搜索歌单
pub async fn search_music_list(
    content: &str,
    page: u32,
    limit: u32,
) -> Result<Vec<MusicList>, anyhow::Error> {
    assert!(page >= 1, "Page must be greater than 0");
    let resp = wy_search::wy_search(SearchTarget::MusicList, content, page, limit).await?;
    let resp = serde_json::from_str::<MusicListSearchResponse>(&resp)?;
    Ok(resp
        .result
        .playlists
        .into_iter()
        .map(|p| Box::new(p) as MusicList)
        .collect())
}

#[tokio::test]
async fn test_search_music_list() {
    let res = search_music_list("周杰伦", 1, 10).await.unwrap();
    res.iter().for_each(|x| {
        println!("{}", x.get_musiclist_info());
    });
    let first = res.first().unwrap();
    let musics = first.get_music_aggregators(1, 10).await.unwrap();
    musics.iter().for_each(|x| {
        println!("{}", x.get_default_music().get_music_info());
    });
}

#[derive(Debug, Serialize, Deserialize)]
struct MusicListDetailResponse {
    pub playlist: MusicListDetailResponseInnerPlaylist,
}

#[derive(Debug, Serialize, Deserialize)]
struct MusicListDetailResponseInnerPlaylist {
    #[serde(default)]
    pub id: u64,
    pub trackIds: Vec<TrackId>,
    pub name: Option<String>,
    pub coverImgUrl: Option<String>,
    pub playCount: u32,
    pub description: Option<String>,
}

impl MusicListTrait for MusicListDetailResponseInnerPlaylist {
    fn get_musiclist_info(&self) -> MusicListInfo {
        MusicListInfo {
            name: self.name.clone().unwrap_or_default(),
            art_pic: self.coverImgUrl.clone().unwrap_or_default(),
            desc: self.description.clone().unwrap_or_default(),
            extra: Some(ExtraInfo {
                play_count: Some(self.playCount),
                music_count: Some(self.trackIds.len() as u32),
            }),
        }
    }

    fn get_music_aggregators<'a>(
        &'a self,
        page: u32,
        limit: u32,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<Vec<MusicAggregator>, anyhow::Error>> + 'a>,
    > {
        let musiclist_id = self.id.clone();
        Box::pin(async move {
            Ok(get_musics_from_music_list(musiclist_id, page, limit)
                .await?
                .1)
        })
    }

    fn source(&self) -> String {
        WANGYI.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackId {
    id: u64,
}

// page starts with 1.
pub async fn get_musics_from_music_list(
    musiclist_id: u64,
    page: u32,
    limit: u32,
) -> Result<(MusicList, Vec<MusicAggregator>), anyhow::Error> {
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

    let resp = resp?.json::<MusicListDetailResponse>().await?;
    let mut musiclist = resp.playlist;
    musiclist.id = musiclist_id;

    let ids = musiclist
        .trackIds
        .iter()
        .map(|id| id.id)
        .collect::<Vec<u64>>();

    // 计算分页的开始和结束位置
    let start = ((page - 1) * limit) as usize;

    // 获取分页后的ID
    let paged_ids = ids
        .iter()
        .skip(start)
        .take(limit as usize)
        .cloned()
        .collect::<Vec<u64>>();

    let mut musics = Vec::new();
    for chunk in paged_ids.chunks(1000) {
        let musics_detail = get_musics_detail(chunk).await?;
        musics.extend(musics_detail);
    }
    Ok((Box::new(musiclist) as MusicList, musics))
}
