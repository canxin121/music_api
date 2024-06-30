use serde::{Deserialize, Serialize};

use crate::util::CLIENT;

#[derive(Serialize, Deserialize)]
pub struct MusicInfoResult {
    msg: Vec<Msg>,
}

#[derive(Serialize, Deserialize)]
pub struct Msg {
    duration: u32,
    creator: Creator,
}
#[derive(Serialize, Deserialize)]
pub struct Creator {
    img: String,
}

pub fn gen_get_music_info_url(music_rid: &str) -> String {
    format!("https://fvedio.kuwo.cn/rec.s?rid={}&cmd=rcm_switch&idfa=&prod=kwplayersimple_ip_1.0.2.0&source=kwplayersimple_ip_1.0.2.0_TJ.ipa&corp=kuwo&plat=ip&tmeapp=1&prod_from=kwplayersimple",music_rid.replace("MUSIC_", ""))
}

pub struct DetailMusicInfo {
    pub img: String,
    pub duration: String,
}

pub async fn get_music_info(music_rid: &str) -> Result<DetailMusicInfo, anyhow::Error> {
    let url = gen_get_music_info_url(music_rid);
    let info: MusicInfoResult = CLIENT.get(url).send().await?.json().await?;
    let msg = match info.msg.into_iter().next() {
        Some(msg) => msg,
        None => {
            return Err(anyhow::anyhow!("No Msg Found"));
        }
    };
    Ok(DetailMusicInfo {
        img: msg.creator.img,
        duration: msg.duration.to_string(),
    })
}
