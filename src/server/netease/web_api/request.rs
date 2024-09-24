use serde_json::json;

use crate::CLIENT;

use super::encrypt::eapi;

pub async fn eapi_request(url: &str, data: &str) -> Result<String, anyhow::Error> {
    Ok(CLIENT.post("http://interface.music.163.com/eapi/batch")
        .header("User-Agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36" )
        .header("origin", "https://music.163.com")
        .form(&eapi(url, data))
        .send()
        .await?
        .text()
        .await?)
}

pub enum SearchTarget {
    // Singer,
    // Album,
    Music,
    MusicList,
}

impl SearchTarget {
    fn to_type(&self) -> u16 {
        match self {
            SearchTarget::Music => 1,
            // SearchTarget::Album => 10,
            SearchTarget::MusicList => 1000,
            // SearchTarget::Singer => 100,
        }
    }
}

pub async fn search(
    search_target: SearchTarget,
    content: &str,
    page: i64,
    limit: i64,
) -> Result<String, anyhow::Error> {
    if page == 0 {
        return Err(anyhow::anyhow!("Page must be greater than 0"));
    }
    let offset: u64 = limit as u64 * (page as u64 - 1);
    let total = page == 1;

    let request_body = json!({
        "s": content,
        "type": search_target.to_type(),
        "limit": limit,
        "total": total,
        "offset": offset
    })
    .to_string();

    let result = eapi_request("/api/cloudsearch/pc", &request_body).await?;
    Ok(result)
}
