use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    data::interface::{playlist::Playlist, server::MusicServer},
    server::netease::model::Model,
    CLIENT,
};

use super::{encrypt::weapi, music::NeteaseMusic};

pub async fn get_musics_from_album(
    album_id: &str,
) -> Result<(Playlist, Vec<Model>), anyhow::Error> {
    let data = json!({}).to_string();
    let resp = CLIENT
        .post(format!("http://music.163.com/weapi/v1/album/{}",album_id))
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36")
        // .header("Referer", format!("https://music.163.com/song?id={music_id}"))
        .header("origin", "https://music.163.com")
        .form(&weapi(&data)?)
        .send()
        .await?;

    let resp = resp.text().await?;

    // std::fs::write("sample_data/netease/get_musics_from_album.json", resp).unwrap();

    let result: GetAlbumResult = serde_json::from_str(&resp)?;
    let playlist = result.album.into();
    let musics = result
        .songs
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<Model>>();
    Ok((playlist, musics))
}

#[tokio::test]
async fn test_get_musics_from_album() {
    let album_id = "78691451";
    let (playlist, musics) = get_musics_from_album(album_id).await.unwrap();
    println!("{:?}", playlist);
    println!("{:?}", musics);
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAlbumResult {
    // pub resource_state: bool,
    #[serde(default)]
    pub songs: Vec<NeteaseMusic>,
    // pub code: i64,
    pub album: Album,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    // pub songs: Vec<Value>,
    // pub paid: bool,
    // pub on_sale: bool,
    // pub mark: i64,
    // pub award_tags: Value,
    pub blur_pic_url: Option<String>,
    // pub company_id: i64,
    // pub pic: i64,
    // pub artists: Vec<Artist>,
    // pub copyright_id: i64,
    // pub pic_id: i64,
    pub artist: Artist2,
    // pub publish_time: i64,
    // pub company: String,
    // pub brief_desc: String,
    pub pic_url: Option<String>,
    // pub comment_thread_id: String,
    // pub status: i64,
    // pub sub_type: String,
    // pub alias: Vec<Value>,
    pub description: String,
    // pub tags: String,
    pub name: String,
    pub id: i64,
    // #[serde(rename = "type")]
    // pub type_field: String,
    pub size: i64,
    // #[serde(rename = "picId_str")]
    // pub pic_id_str: String,
    // pub info: Info,
}

impl Into<Playlist> for Album {
    fn into(self) -> Playlist {
        Playlist {
            from_db: false,
            server: Some(MusicServer::Netease),
            type_field: crate::data::interface::playlist::PlaylistType::Album,
            identity: self.id.to_string(),
            name: self.name,
            summary: Some(self.description),
            cover: self.pic_url.or(self.blur_pic_url),
            creator: Some(self.artist.name),
            creator_id: Some(self.artist.id.to_string()),
            play_time: None,
            music_num: Some(self.size),
            subscription: None,
            order: None,
        }
    }
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Artist {
//     #[serde(rename = "img1v1Id")]
//     pub img1v1id: i64,
//     pub topic_person: i64,
//     pub trans: String,
//     pub followed: bool,
//     pub pic_id: i64,
//     pub brief_desc: String,
//     pub music_size: i64,
//     pub album_size: i64,
//     pub pic_url: String,
//     #[serde(rename = "img1v1Url")]
//     pub img1v1url: String,
//     pub alias: Vec<Value>,
//     pub name: String,
//     pub id: i64,
//     #[serde(rename = "img1v1Id_str")]
//     pub img1v1id_str: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist2 {
    // #[serde(rename = "img1v1Id")]
    // pub img1v1id: i64,
    // pub topic_person: i64,
    // pub trans: String,
    // pub followed: bool,
    // pub pic_id: i64,
    // pub brief_desc: String,
    // pub music_size: i64,
    // pub album_size: i64,
    // pub pic_url: String,
    // #[serde(rename = "img1v1Url")]
    // pub img1v1url: String,
    // pub alias: Vec<String>,
    pub name: String,
    pub id: i64,
    // #[serde(rename = "picId_str")]
    // pub pic_id_str: String,
    // #[serde(rename = "img1v1Id_str")]
    // pub img1v1id_str: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Info {
//     pub comment_thread: CommentThread,
//     pub latest_liked_users: Value,
//     pub liked: bool,
//     pub comments: Value,
//     pub resource_type: i64,
//     pub resource_id: i64,
//     pub comment_count: i64,
//     pub liked_count: i64,
//     pub share_count: i64,
//     pub thread_id: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct CommentThread {
//     pub id: String,
//     pub resource_info: ResourceInfo,
//     pub resource_type: i64,
//     pub comment_count: i64,
//     pub liked_count: i64,
//     pub share_count: i64,
//     pub hot_count: i64,
//     pub latest_liked_users: Value,
//     pub resource_owner_id: i64,
//     pub resource_title: String,
//     pub resource_id: i64,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ResourceInfo {
//     pub id: i64,
//     pub user_id: i64,
//     pub name: String,
//     pub img_url: String,
//     pub creator: Value,
//     pub encoded_id: Value,
//     pub sub_title: Value,
//     pub web_url: Value,
// }
