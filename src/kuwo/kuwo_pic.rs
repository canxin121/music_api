fn gen_get_pic_url(music_id: &str) -> String {
    format!(
        "http://artistpicserver.kuwo.cn/pic.web?corp=kuwo&type=rid_pic&pictype=500&size=500&rid={}",
        music_id.replace("MUSIC_", "")
    )
}

pub(crate) async fn get_pic_url(music_id: &str) -> Result<String, anyhow::Error> {
    let url = reqwest::get(gen_get_pic_url(music_id))
        .await?
        .text()
        .await?;
    if !url.contains("http") {
        Err(anyhow::anyhow!("No 'http' in return, image not found."))
    } else {
        Ok(url)
    }
}
