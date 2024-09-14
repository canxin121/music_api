#![allow(non_snake_case, unused)]
use std::{fs, io::Write};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{fs::File, io::AsyncWriteExt as _};

use super::{encrypt::linux_api, request::eapi_request};

pub enum SearchTarget {
    Singer,
    Album,
    SingleMusic,
    MusicList,
}

impl SearchTarget {
    fn to_type(&self) -> u16 {
        match self {
            SearchTarget::SingleMusic => 1,
            SearchTarget::Album => 10,
            SearchTarget::MusicList => 1000,
            SearchTarget::Singer => 100,
        }
    }
}

pub async fn wy_search(
    search_target: SearchTarget,
    content: &str,
    page: u16,
    limit: u16,
) -> Result<String, anyhow::Error> {
    if page == 0 {
        return Err(anyhow::anyhow!("Page must be greater than 0"));
    }
    let offset = limit * (page - 1);
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

// 搜索单曲
pub async fn search_single_music(
    content: &str,
    page: u16,
    limit: u16,
) -> Result<(), anyhow::Error> {
    let resp = wy_search(SearchTarget::SingleMusic, content, page, limit).await?;
    std::fs::write("sample_data/netease/search_music.json", resp).expect("Failed to write result to file");
    todo!()
}

#[tokio::test]
async fn test_search_netease_single_music() {
    let content = "张惠妹";
    let page = 1;
    let limit = 30;
    let result = search_single_music(content, page, limit).await.unwrap();
}
