use crate::refactor::{
    data::interface::{playlist::Playlist, MusicServer},
    server::{kuwo::web_api::utils::find_id_from_share_url, CLIENT},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub async fn get_kuwo_music_list_from_share(share: &str) -> Result<Playlist> {
    let playlist_id = find_id_from_share_url(share).ok_or(anyhow::anyhow!(
        "Failed to find playlist id in share content"
    ))?;
    let url = format!("http://nplserver.kuwo.cn/pl.svc?op=getlistinfo&pid={}&pn=0&rn=0&encode=utf8&keyset=pl2012&vipver=MUSIC_9.1.1.2_BCS2&newver=1",playlist_id);
    let share_music: ShareMusicList = CLIENT.get(&url).send().await?.json().await?;
    Ok(share_music.into())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareMusicList {
    // pub abstime: i64,
    // pub ctime: i64,
    pub id: i64,
    pub info: String,
    // pub ispub: bool,
    // pub musiclist: Vec<Value>,
    pub pic: String,
    pub playnum: i64,
    // pub pn: i64,
    // pub result: String,
    // pub rn: i64,
    // pub sharenum: i64,
    // pub songtime: i64,
    // pub state: i64,
    // pub tag: String,
    // pub tagid: String,
    pub title: String,
    pub total: i64,
    // #[serde(rename = "type")]
    // pub type_field: String,
    pub uid: i64,
    pub uname: String,
    // pub validtotal: i64,
}

impl Into<Playlist> for ShareMusicList {
    fn into(self) -> Playlist {
        Playlist {
            from_db: false,
            server: Some(MusicServer::Kuwo),
            type_field: crate::refactor::data::interface::playlist::PlaylistType::UserPlaylist,
            identity: self.id.to_string(),
            name: self.title,
            summary: Some(self.info),
            cover: Some(self.pic),
            creator: Some(self.uname),
            creator_id: Some(self.uid.to_string()),
            play_time: Some(self.playnum),
            music_num: Some(self.total),
            subscription: None,
        }
    }
}

#[tokio::test]
async fn test_share() {
    let share = "https://m.kuwo.cn/newh5app/playlist_detail/1312045587?from=ip&t=qqfriend";
    let playlist = get_kuwo_music_list_from_share(share).await.unwrap();
    let musics = playlist.fetch_musics(1, 9999).await.unwrap();
    println!("{:?}", musics);

    println!("Length: {}", musics.len());
}
