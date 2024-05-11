use serde::Deserialize;

use crate::{search_factory::CLIENT, util::format_seconds_to_timestamp};

#[derive(Deserialize, Debug)]
pub struct GerLrcResult {
    pub(crate) data: GetLrcData,
}

#[derive(Deserialize, Debug)]
pub struct GetLrcData {
    pub(crate) lrclist: Vec<GetLrc>,
}

#[derive(Deserialize, Debug)]
pub struct GetLrc {
    #[serde(rename = "lineLyric")]
    pub(crate) line_lyric: String,
    pub(crate) time: String,
}

pub(crate) fn gen_get_lrc_url(song_id: &str) -> String {
    format!(
        "https://m.kuwo.cn/newh5/singles/songinfoandlrc?musicId={}",
        song_id.replace("MUSIC_", "")
    )
}

pub(crate) async fn get_lrc(song_id: &str) -> Result<String, anyhow::Error> {
    let result = CLIENT
        .get(gen_get_lrc_url(song_id))
        .send()
        .await?
        .json::<GerLrcResult>()
        .await?;
    let lyrics = result
        .data
        .lrclist
        .into_iter()
        .filter_map(|lrc| {
            let time: f64 = match lrc.time.parse() {
                Ok(t) => t,
                Err(_) => return None,
            };
            Some(format!(
                "[{}]{}",
                format_seconds_to_timestamp(time),
                lrc.line_lyric
            ))
        })
        .collect::<Vec<String>>()
        .join("\n");
    Ok(lyrics)
}
