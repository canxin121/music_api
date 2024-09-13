use serde::Deserialize;
use serde_json::json;

use crate::refactor::server::CLIENT;

use super::encrypt::weapi;

pub async fn get_musics_from_album(album_id: u64) -> Result<(), anyhow::Error> {
    let data = json!({}).to_string();
    let resp = CLIENT
        .post(format!("http://music.163.com/weapi/v1/album/{}",album_id))
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36")
        // .header("Referer", format!("https://music.163.com/song?id={music_id}"))
        .header("origin", "https://music.163.com")
        .form(&weapi(&data)?)
        .send()
        .await?;

    let mut resp = resp.text().await?;
    println!("{}", resp);
    todo!()
}

#[tokio::test]
async fn test_get_musics_from_album() {
    let album_id = 78691451;
    let result = get_musics_from_album(album_id).await.unwrap();
}
