use std::mem;

use serde::Deserialize;
use serde_json::json;

use crate::{
    music_aggregator::music_aggregator_online::SearchMusicAggregator,
    music_list::{ExtraInfo, MusicList, MusicListTrait},
    util::CLIENT,
    Music, MusicAggregator, MusicListInfo,
};

use super::{weapi, wy_music::WyMusic, WANGYI};

#[derive(Deserialize)]
struct AlbumResponse {
    album: Album,
    songs: Vec<WyMusic>,
}
#[derive(Deserialize)]
struct Album {
    #[allow(unused)]
    id: u64,
    name: String,
    #[serde(rename = "picUrl")]
    pic_url: String,
    description: String,
    #[serde(default)]
    song_num: u32,
}

impl MusicListTrait for Album {
    fn source(&self) -> String {
        WANGYI.to_string()
    }

    fn get_musiclist_info(&self) -> MusicListInfo {
        MusicListInfo {
            name: self.name.clone(),
            art_pic: self.pic_url.clone(),
            desc: self.description.clone(),
            extra: Some(ExtraInfo {
                play_count: None,
                music_count: Some(self.song_num),
            }),
        }
    }

    fn get_music_aggregators(
        &self,
        _page: u32,
        _limit: u32,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                    Output = Result<Vec<crate::music_aggregator::MusicAggregator>, anyhow::Error>,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async { Ok(Vec::with_capacity(0)) })
    }
}

pub async fn get_musics_from_album(
    album_id: u64,
) -> Result<(MusicList, Vec<MusicAggregator>), anyhow::Error> {
    let data = json!({}).to_string();
    let resp = CLIENT
        .post(format!("http://music.163.com/weapi/v1/album/{}",album_id))
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.90 Safari/537.36")
        // .header("Referer", format!("https://music.163.com/song?id={music_id}"))
        .header("origin", "https://music.163.com")
        .form(&weapi(&data)?)
        .send()
        .await?;
    let mut resp = resp.json::<AlbumResponse>().await?;
    resp.songs.iter_mut().for_each(|s| {
        s.default_quality = s.get_highest_quality();
    });
    let mut musics = Vec::new();
    mem::swap(&mut musics, &mut resp.songs);
    let musics = resp
        .songs
        .into_iter()
        .map(|m| {
            Box::new(SearchMusicAggregator::from_music(Box::new(m) as Music)) as MusicAggregator
        })
        .collect();

    Ok((Box::new(resp.album), musics))
}

#[tokio::test]
async fn test_get_musics_from_album() {
    let album_id = 78691451;
    let result = get_musics_from_album(album_id).await.unwrap();
    println!("{}", result.0.get_musiclist_info());
    result.1.iter().for_each(|m| {
        println!("{}", m);
    });
}
