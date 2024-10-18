use serde::{Deserialize, Serialize};

use crate::{
    interface::{
        artist::Artist,
        music_chart::{MusicChart, MusicChartCollection, ServerMusicChartCollection},
        server::MusicServer,
    },
    server::kuwo::{self, web_api::utils::get_music_rid_pic},
    CLIENT,
};

use super::utils::parse_qualities_formats;

pub async fn get_music_chart_collection() -> anyhow::Result<ServerMusicChartCollection> {
    Ok(CLIENT
        .get("http://wapi.kuwo.cn/api/pc/bang/list")
        .send()
        .await?
        .json::<KuwoMusicChartsCollectionResult>()
        .await?
        .into())
}

pub async fn get_musics_from_chart(
    id: &str,
    page: u16,
    size: u16,
) -> anyhow::Result<Vec<kuwo::model::Model>> {
    let url = format!("http://kbangserver.kuwo.cn/ksong.s?from=pc&fmt=json&pn={}&rn={}&type=bang&data=content&id={}&show_copyright_off=0&pcmp4=1&isbang=1&userid=0",page-1,size,id);

    let result: ChartMusicResult = CLIENT.get(url).send().await?.json().await?;

    let mut musics = result
        .musiclist
        .into_iter()
        .map(|m| {
            let model: kuwo::model::Model = m.into();
            model
        })
        .collect::<Vec<kuwo::model::Model>>();

    let mut handles = Vec::with_capacity(musics.len());

    for music in &musics {
        let id = music.music_id.clone();
        handles.push(tokio::spawn(async move { get_music_rid_pic(&id).await }))
    }

    for (music, handle) in musics.iter_mut().zip(handles) {
        music.cover = handle.await?.ok().ok_or(anyhow::anyhow!("No cover"))?;
    }

    Ok(musics)
}

#[cfg(test)]
mod test {
    use crate::server::kuwo::web_api::chart::{get_music_chart_collection, get_musics_from_chart};

    #[tokio::test]
    async fn test_chart_collection() {
        let result = get_music_chart_collection().await.unwrap();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_chart_musics() {
        let chart_collections = get_music_chart_collection().await.unwrap();
        let first_chart = chart_collections
            .collections
            .first()
            .unwrap()
            .charts
            .first()
            .unwrap();
        println!("{:?}", first_chart);
        let musics = get_musics_from_chart(&first_chart.id, 1, 30).await.unwrap();
        println!("{:?}", musics)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuwoMusicChartsCollectionResult {
    // pub sourceid: String,
    // pub like: String,
    // pub isnew: String,
    // pub pic: String,
    // pub source: String,
    // pub tips: String,
    // pub listen: String,
    // pub extend: String,
    // pub newcnt: String,
    // pub intro: String,
    // pub name: String,
    // #[serde(rename = "pc_extend")]
    // pub pc_extend: String,
    // pub id: String,
    // pub pic2: String,
    // pub disname: String,
    // pub info: String,
    pub child: Vec<KuwoMusicCollection>,
    // pub pic5: String,
}

impl Into<ServerMusicChartCollection> for KuwoMusicChartsCollectionResult {
    fn into(self) -> ServerMusicChartCollection {
        ServerMusicChartCollection {
            server: MusicServer::Kuwo,
            collections: self.child.into_iter().map(|c| c.into()).collect(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuwoMusicCollection {
    // pub sourceid: String,
    // pub like: String,
    // pub isnew: String,
    pub pic: String,
    // pub source: String,
    // pub tips: String,
    // pub listen: String,
    // pub extend: String,
    // pub newcnt: String,
    // pub intro: String,
    pub name: String,
    // #[serde(rename = "pc_extend")]
    // pub pc_extend: String,
    // pub id: String,
    // pub pic2: String,
    pub disname: String,
    // pub info: String,
    pub child: Vec<KuwoMusicChart>,
    // pub pic5: String,
}

impl Into<MusicChartCollection> for KuwoMusicCollection {
    fn into(self) -> MusicChartCollection {
        MusicChartCollection {
            name: self.name,
            charts: self.child.into_iter().map(|c| c.into()).collect(),
            summary: Some(self.disname),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuwoMusicChart {
    pub sourceid: String,
    // pub like: String,
    // pub pub_time: String,
    // pub isnew: String,
    // pub pic: String,
    // pub source: String,
    // pub tips: String,
    // pub listen: String,
    // pub extend: String,
    // pub newcnt: String,
    pub intro: String,
    pub name: String,
    // #[serde(rename = "pc_extend")]
    // pub pc_extend: String,
    // pub id: String,
    pub pic2: String,
    // pub disname: String,
    // pub info: String,
    // pub child: Vec<Value>,
    // pub pic5: String,
}

impl Into<MusicChart> for KuwoMusicChart {
    fn into(self) -> MusicChart {
        MusicChart {
            name: self.name,
            id: self.sourceid,
            summary: Some(self.intro),
            cover: Some(self.pic2),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartMusicResult {
    // pub name: String,
    // pub leader: String,
    // pub term: String,
    // pub info: String,
    // pub pic: String,
    // #[serde(rename = "pub")]
    // pub pub_field: String,
    // pub timestamp: String,
    // pub num: String,
    // #[serde(rename = "v9_pic2")]
    // pub v9_pic2: String,
    // #[serde(rename = "type")]
    // pub type_field: String,
    pub musiclist: Vec<KuwoChartMusic>,
    // pub volume: Volume,
    // pub current_volume: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuwoChartMusic {
    pub cover: Option<String>,
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artistid: String,
    pub album: String,
    pub albumid: String,
    // pub score100: String,
    pub formats: String,
    // pub mp4sig1: i64,
    // pub mp4sig2: i64,
    // pub param: String,
    // pub ispoint: String,
    // pub mutiver: String,
    // pub pay: String,
    // pub online: String,
    // pub copyright: String,
    // #[serde(rename = "rank_change")]
    // pub rank_change: String,
    // pub isnew: String,
    // pub duration: String,
    // #[serde(rename = "highest_rank")]
    // pub highest_rank: String,
    // pub score: String,
    // pub transsongname: String,
    // pub aartist: String,
    // pub opay: String,
    // pub tpay: String,
    // #[serde(rename = "overseas_pay")]
    // pub overseas_pay: String,
    // #[serde(rename = "overseas_copyright")]
    // pub overseas_copyright: String,
    #[serde(rename = "song_duration")]
    pub song_duration: String,
    // pub pay_info: PayInfo,
    // pub mvpayinfo: Mvpayinfo,
    // pub audiobookpayinfo: Audiobookpayinfo,
    // pub nationid: String,
    // pub fpay: String,
    // pub isdownload: String,
    // pub trend: String,
}

impl Into<kuwo::model::Model> for KuwoChartMusic {
    fn into(self) -> kuwo::model::Model {
        let artist_names = self
            .artist
            .split("&")
            .filter(|a| !a.is_empty())
            .collect::<Vec<&str>>();
        let artist_ids = self
            .artistid
            .split("&")
            .filter(|a| !a.is_empty())
            .collect::<Vec<&str>>();
        let artists: Vec<Artist> = artist_names
            .into_iter()
            .zip(artist_ids.into_iter().chain(std::iter::repeat("")))
            .map(|(name, id)| crate::interface::artist::Artist {
                name: name.to_string(),
                id: id.parse().ok(),
            })
            .collect();
        let artists = crate::interface::artist::ArtistVec::from(artists);
        kuwo::model::Model {
            name: self.name,
            music_id: self.id,
            duration: self.song_duration.parse().ok(),
            artists,
            album: Some(self.album),
            album_id: Some(self.albumid),
            qualities: parse_qualities_formats(&self.formats).into(),
            cover: self.cover,
        }
    }
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PayInfo {
//     pub cannot_online_play: String,
//     pub cannot_download: String,
//     pub download: String,
//     pub fee_type: FeeType,
//     #[serde(rename = "listen_fragment")]
//     pub listen_fragment: String,
//     #[serde(rename = "local_encrypt")]
//     pub local_encrypt: String,
//     pub play: String,
//     #[serde(rename = "tips_intercept")]
//     pub tips_intercept: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct FeeType {
//     pub album: String,
//     pub bookvip: String,
//     pub song: String,
//     pub vip: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Mvpayinfo {
//     pub download: String,
//     pub play: String,
//     pub vid: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Audiobookpayinfo {
//     pub download: String,
//     pub play: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Volume {
//     #[serde(rename = "2024")]
//     pub n2024: Vec<n2024>,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct n2024 {
//     pub id: String,
//     #[serde(rename = "second_id")]
//     pub second_id: String,
//     #[serde(rename = "third_id")]
//     pub third_id: String,
//     pub name: String,
// }
