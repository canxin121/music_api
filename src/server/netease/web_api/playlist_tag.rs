use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    interface::{
        playlist::Playlist,
        playlist_tag::{
            PlaylistTag, PlaylistTagCollection, ServerPlaylistTagCollection, TagPlaylistOrder,
        },
    },
    server::netease::web_api::encrypt::weapi,
    CLIENT,
};

use super::playlist::NeteasePlaylist;

pub async fn get_playlist_tags() -> anyhow::Result<ServerPlaylistTagCollection> {
    Ok(CLIENT
        .post("https://music.163.com/weapi/playlist/catalogue")
        .form(&weapi(&json!({}).to_string())?)
        .send()
        .await?
        .json::<NeteasePlaylistTagCollectionResult>()
        .await?
        .into())
}

pub async fn get_playlists_from_tag(
    tag_id: &str,
    order: TagPlaylistOrder,
    page: u16,
    size: u16,
) -> anyhow::Result<Vec<Playlist>> {
    let order = match order {
        TagPlaylistOrder::Hot => "hot",
        TagPlaylistOrder::New => "new",
    };

    let data = json!({
        "cat": tag_id,
        "order": order,
        "limit": size,
        "offset": size * (page - 1),
        "total": true,
    });

    CLIENT
        .post("https://music.163.com/weapi/playlist/list")
        .form(&weapi(&data.to_string())?)
        .send()
        .await?
        .json::<NeteaseTagPlaylistResult>()
        // .text()
        .await?
        .playlists
        .into_iter()
        .map(|p| Ok(p.into()))
        .collect()
}

#[cfg(test)]
mod test {
    use crate::server::netease::web_api::playlist_tag::{
        get_playlist_tags, get_playlists_from_tag,
    };

    #[tokio::test]
    async fn test_get_playlist_tags() {
        let result = get_playlist_tags().await.unwrap();
        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_get_playlists_from_tag() {
        let result = get_playlists_from_tag(
            "全部歌单",
            crate::interface::playlist_tag::TagPlaylistOrder::Hot,
            1,
            10,
        )
        .await
        .unwrap();
        println!("{:?}", result);
        let result = get_playlists_from_tag(
            "全部歌单",
            crate::interface::playlist_tag::TagPlaylistOrder::New,
            1,
            10,
        )
        .await
        .unwrap();
        println!("{:?}", result);
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteasePlaylistTagCollectionResult {
    // pub code: i64,
    pub all: NeteaseAllPlaylistTags,
    pub sub: Vec<NeteasePlaylistTag>,
    pub categories: NeteasePlaylistTagCategories,
}

impl Into<ServerPlaylistTagCollection> for NeteasePlaylistTagCollectionResult {
    fn into(self) -> ServerPlaylistTagCollection {
        let mut collections: HashMap<String, Vec<PlaylistTag>> = HashMap::new();

        let all_categeory = self.categories.get(self.all.category);
        let all_collection = collections.entry(all_categeory).or_insert_with(Vec::new);
        all_collection.push(self.all.into());

        for sub in self.sub {
            let category = self.categories.get(sub.category);
            let collection = collections.entry(category).or_insert_with(Vec::new);
            collection.push(sub.into());
        }

        ServerPlaylistTagCollection {
            server: crate::interface::server::MusicServer::Netease,
            collections: collections
                .into_iter()
                .map(|(name, tags)| PlaylistTagCollection { name, tags })
                .collect(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseAllPlaylistTags {
    pub name: String,
    // pub resource_count: i64,
    // pub img_id: i64,
    // pub img_url: Value,
    // #[serde(rename = "type")]
    // pub type_field: i64,
    pub category: i64,
    // pub resource_type: i64,
    // pub hot: bool,
    // pub activity: bool,
}

impl Into<PlaylistTag> for NeteaseAllPlaylistTags {
    fn into(self) -> PlaylistTag {
        PlaylistTag {
            name: self.name.clone(),
            id: self.name,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteasePlaylistTag {
    pub name: String,
    // pub resource_count: i64,
    // pub img_id: i64,
    // pub img_url: Value,
    // #[serde(rename = "type")]
    // pub type_field: i64,
    pub category: i64,
    // pub resource_type: i64,
    // pub hot: bool,
    // pub activity: bool,
}

impl Into<PlaylistTag> for NeteasePlaylistTag {
    fn into(self) -> PlaylistTag {
        PlaylistTag {
            name: self.name.clone(),
            id: self.name,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteasePlaylistTagCategories {
    #[serde(rename = "0")]
    pub n0: String,
    #[serde(rename = "1")]
    pub n1: String,
    #[serde(rename = "2")]
    pub n2: String,
    #[serde(rename = "3")]
    pub n3: String,
    #[serde(rename = "4")]
    pub n4: String,
}

impl NeteasePlaylistTagCategories {
    pub fn get(&self, index: i64) -> String {
        match index {
            0 => self.n0.clone(),
            1 => self.n1.clone(),
            2 => self.n2.clone(),
            3 => self.n3.clone(),
            4 => self.n4.clone(),
            _ => "未知".to_string(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseTagPlaylistResult {
    pub playlists: Vec<NeteasePlaylist>,
    // pub total: i64,
    // pub code: i64,
    // pub more: bool,
    // pub cat: String,
}
