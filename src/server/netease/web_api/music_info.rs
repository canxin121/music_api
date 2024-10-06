use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{server::netease::model::Model, CLIENT};

use super::{encrypt::weapi, music::NeteaseMusic};
use anyhow::Result;

#[derive(Serialize)]
struct CItem {
    id: i64,
}

#[derive(Deserialize)]
struct GetMusicResponse {
    #[serde(default)]
    songs: Vec<NeteaseMusic>,
}

pub async fn get_musics_info(music_ids: &[i64]) -> Result<Vec<Model>> {
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
    let resp = resp.json::<GetMusicResponse>().await?;

    let musics = resp.songs.into_iter().map(|s| s.into()).collect();
    Ok(musics)
}
