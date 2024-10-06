use serde::{Deserialize, Serialize};

use crate::{
    interface::{
        playlist::Playlist,
        playlist_tag::{
            PlaylistTag, PlaylistTagCollection, ServerPlaylistTagCollection, TagPlaylistOrder,
        },
        server::MusicServer,
    },
    CLIENT,
};

pub async fn get_playlist_tags() -> anyhow::Result<ServerPlaylistTagCollection> {
    Ok(CLIENT.get( "http://wapi.kuwo.cn/api/pc/classify/playlist/getTagList?cmd=rcm_keyword_playlist&user=0&prod=kwplayer_pc_9.1.1.2&vipver=9.1.1.2&source=kwplayer_pc_9.1.1.2&loginUid=0&loginSid=0&appUid=38668888").send().await?.json::<KuwoPlaylistTagResult>().await?.into())
}

pub async fn get_playlists_from_tag(
    tag_id: &str,
    order: TagPlaylistOrder,
    page: u16,
    size: u16,
) -> anyhow::Result<Vec<Playlist>> {
    if page == 0 {
        return Err(anyhow::anyhow!("page must be greater than 0"));
    }
    let order = match order {
        TagPlaylistOrder::Hot => "hot",
        TagPlaylistOrder::New => "new",
    };

    let result: TagPlaylistResult = CLIENT.get(format!("https://wapi.kuwo.cn/api/pc/classify/playlist/getTagPlayList?loginUid=0&loginSid=0&appUid=38668888&id={}&pn={}&rn={}&order={}", tag_id, page - 1, size,order)).send().await?.json().await?;
    Ok(result.data.data.into_iter().map(|p| p.into()).collect())
}

#[cfg(test)]
mod test {
    use crate::server::kuwo::web_api::playlist_tag::{get_playlist_tags, get_playlists_from_tag};

    #[tokio::test]
    async fn test_get_playlist_tags() {
        let result = get_playlist_tags().await.unwrap();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_get_playlists_from_tag() {
        let result = get_playlists_from_tag(
            "37",
            crate::interface::playlist_tag::TagPlaylistOrder::Hot,
            1,
            10,
        )
        .await
        .unwrap();
        println!("{:?}", result);
        let result = get_playlists_from_tag(
            "37",
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
pub struct KuwoPlaylistTagResult {
    // pub code: i64,
    // pub cur_time: i64,
    pub data: Vec<KuwoPlaylistTagCollection>,
    // pub msg: String,
    // pub profile_id: String,
    // pub req_id: String,
    // pub t_id: String,
}

impl Into<ServerPlaylistTagCollection> for KuwoPlaylistTagResult {
    fn into(self) -> ServerPlaylistTagCollection {
        ServerPlaylistTagCollection {
            server: crate::interface::server::MusicServer::Kuwo,
            collections: self
                .data
                .into_iter()
                .map(|d| d.into())
                .filter(|d: &PlaylistTagCollection| !d.tags.is_empty())
                .collect(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuwoPlaylistTagCollection {
    // pub img: String,
    // pub mdigest: String,
    pub data: Vec<KuwoPlaylistTag>,
    pub name: String,
    // pub id: String,
    // #[serde(rename = "type")]
    // pub type_field: String,
    // pub img1: String,
}

impl Into<PlaylistTagCollection> for KuwoPlaylistTagCollection {
    fn into(self) -> PlaylistTagCollection {
        PlaylistTagCollection {
            name: self.name,
            tags: self
                .data
                .into_iter()
                .filter(|d| d.digest == "10000")
                .map(|d| d.into())
                .collect(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuwoPlaylistTag {
    // pub extend: String,
    // pub img: String,
    pub digest: String,
    pub name: String,
    // pub isnew: String,
    pub id: String,
}

impl Into<PlaylistTag> for KuwoPlaylistTag {
    fn into(self) -> PlaylistTag {
        PlaylistTag {
            name: self.name,
            id: self.id,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagPlaylistResult {
    // pub code: i64,
    // pub cur_time: i64,
    pub data: TagPlaylistInner,
    // pub msg: String,
    // pub profile_id: String,
    // pub req_id: String,
    // pub t_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagPlaylistInner {
    // pub total: i64,
    pub data: Vec<TagPlaylist>,
    // pub rn: i64,
    // pub pn: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagPlaylist {
    pub img: String,
    pub uname: String,
    // #[serde(rename = "lossless_mark")]
    // pub lossless_mark: String,
    // pub favorcnt: String,
    // pub isnew: String,
    // pub extend: String,
    pub uid: String,
    pub total: String,
    // pub commentcnt: String,
    // pub imgscript: String,
    // pub digest: String,
    pub name: String,
    pub listencnt: String,
    pub id: String,
    // pub attribute: String,
    // #[serde(rename = "radio_id")]
    // pub radio_id: String,
    pub desc: String,
    // pub info: String,
}

impl Into<Playlist> for TagPlaylist {
    fn into(self) -> Playlist {
        Playlist {
            from_db: false,
            server: Some(MusicServer::Kuwo),
            type_field: crate::interface::playlist::PlaylistType::UserPlaylist,
            identity: self.id,
            order: None,
            name: self.name,
            summary: Some(self.desc),
            cover: Some(self.img),
            creator: Some(self.uname),
            creator_id: Some(self.uid),
            play_time: self.listencnt.parse().ok(),
            music_num: self.total.parse().ok(),
            subscription: None,
        }
    }
}
