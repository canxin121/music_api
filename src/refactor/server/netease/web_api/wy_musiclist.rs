#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::refactor::server::{
    netease::web_api::{
        encrypt::linux_api,
        wy_search::{wy_search, SearchTarget},
    },
    CLIENT,
};

// 搜索歌单
pub async fn search_music_list(content: &str, page: u32, limit: u32) -> Result<(), anyhow::Error> {
    assert!(page >= 1, "Page must be greater than 0");
    let resp = wy_search(SearchTarget::MusicList, content, page, limit).await?;
    todo!()
}

// page starts with 1.
pub async fn get_musics_from_music_list(
    musiclist_id: u64,
    page: u32,
    limit: u32,
) -> Result<(), anyhow::Error> {
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
    todo!()
}
