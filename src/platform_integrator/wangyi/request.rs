use crate::util::CLIENT;

use super::encrypt::eapi;

pub(crate) async fn eapi_request(url: &str, data: &str) -> Result<String, anyhow::Error> {
    Ok(CLIENT.post("http://interface.music.163.com/eapi/batch")
        .header("User-Agent","Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36" )
        .header("origin", "https://music.163.com")
        .form(&eapi(url, data))
        .send()
        .await?
        .text()
        .await?)
}
