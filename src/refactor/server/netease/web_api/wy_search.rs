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

// page starts with 1.
pub async fn wy_search(
    search_target: SearchTarget,
    content: &str,
    page: u32,
    limit: u32,
) -> Result<String, anyhow::Error> {
    assert!(page >= 1);
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
    page: u32,
    limit: u32,
) -> Result<(), anyhow::Error> {
    assert!(page >= 1, "Page must be greater than 0");
    let resp = wy_search(SearchTarget::SingleMusic, content, page, limit).await?;
    todo!()
}
