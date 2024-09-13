use anyhow::Ok;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::refactor::server::{netease::web_api::encrypt::weapi, CLIENT};

#[derive(Serialize)]
struct CItem {
    id: u64,
}

pub async fn get_musics_detail(music_ids: &[u64]) -> Result<(), anyhow::Error> {
    let c_map_str = serde_json::to_string(
        &music_ids
            .iter()
            .map(|id| CItem { id: *id })
            .collect::<Vec<CItem>>(),
    )?;
    let data = json!({"c":c_map_str,"ids":json!(music_ids).to_string()}).to_string();
    let resp = CLIENT
        .post(r#"https://music.163.com/weapi/v3/song/detail"#)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36")
        .header("origin", "https://music.163.com")
        .form(&weapi(&data)?)
        .send()
        .await?;
    todo!()
}

#[tokio::test]
async fn test_get_song_detail() {
    let music_ids = [
        430685732, 22707008, 16846091, 26127164, 5308028, 698479, 493478198, 36897723, 5307982,
        27514120, 16846088, 22676167, 5101648, 578090, 434902428, 5254129, 1638654, 28283137,
        857896, 451319227, 31066449, 22844535, 26237342, 116493, 22822506, 139718, 4341314,
        5307932, 28406526, 1993749, 22712173, 5271071, 27707270, 1091873, 22802176, 139774,
        1091088, 443521, 103301, 103035, 5267808, 22743825, 21725725, 406232,
    ];
}
