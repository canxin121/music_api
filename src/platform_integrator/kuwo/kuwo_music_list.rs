use std::mem;

use crate::music_aggregator::music_aggregator_online::SearchMusicAggregator;
use crate::music_aggregator::MusicAggregator;
use crate::music_list::{ExtraInfo, MusicList, MusicListTrait};
use crate::util::CLIENT;
use crate::{Music, MusicListInfo};
use serde::{Deserialize, Serialize};

use urlencoding::encode;

use super::util::decode_html_entities;
use super::KUWO;
use super::{
    kuwo_music::KuwoMusic,
    kuwo_quality::{process_qualities, KuWoQuality},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    abslist: Vec<KuwoMusicList>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KuwoMusicList {
    pic: String,
    name: String,
    intro: String,
    playlistid: String,
    playcnt: String,
    songnum: String,
}

impl MusicListTrait for KuwoMusicList {
    fn get_musiclist_info(&self) -> MusicListInfo {
        MusicListInfo {
            id: 0,
            name: self.name.clone(),
            art_pic: self.pic.clone(),
            desc: self.intro.clone(),
            extra: Some(ExtraInfo {
                play_count: self.playcnt.parse().ok(),
                music_count: self.songnum.parse().ok(),
            }),
        }
    }

    fn get_music_aggregators(
        &self,
        page: u32,
        limit: u32,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                    Output = Result<Vec<crate::music_aggregator::MusicAggregator>, anyhow::Error>,
                > + Send
                + '_,
        >,
    > {
        let playlist_id = self.playlistid.clone();
        Box::pin(async move {
            get_musics_of_music_list(&playlist_id, page, limit)
                .await
                .map(|(_info, musics)| {
                    musics
                        .into_iter()
                        .map(|m| Box::new(SearchMusicAggregator::from_music(m)) as MusicAggregator)
                        .collect()
                })
        })
    }

    fn source(&self) -> String {
        KUWO.to_string()
    }
}

pub fn gen_music_list_url(content: &str, page: u32, limit: u32) -> String {
    format!("http://search.kuwo.cn/r.s?all={}&pn={}&rn={limit}&rformat=json&encoding=utf8&ver=mbox&vipver=MUSIC_8.7.7.0_BCS37&plat=pc&devid=28156413&ft=playlist&pay=0&needliveshow=0",encode(content),page-1)
}

pub fn gen_get_musics_url(playlist_id: &str, page: u32, limit: u32) -> String {
    format!("http://nplserver.kuwo.cn/pl.svc?op=getlistinfo&pid={playlist_id}&pn={}&rn={limit}&encode=utf8&keyset=pl2012&identity=kuwo&pcmp4=1&vipver=MUSIC_9.0.5.0_W1&newver=1",page-1)
}

pub async fn search_music_list(
    content: &str,
    page: u32,
    limit: u32,
) -> Result<Vec<MusicList>, anyhow::Error> {
    let url = gen_music_list_url(content, page, limit);
    let mut text = CLIENT
        .get(&url)
        .send()
        .await?
        .text()
        .await?
        .replace("'", "\"");
    text = decode_html_entities(text);
    let search_result: SearchResult = serde_json::from_str(&text)?;
    Ok(search_result
        .abslist
        .into_iter()
        .map(|m| Box::new(m) as MusicList)
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct MusicListDetail {
    #[serde(default)]
    pub musiclist_id: String,
    musiclist: Vec<KuwoMusic>,
    pic: String,
    playnum: u32,
    title: String,
    info: String,
}

impl MusicListTrait for MusicListDetail {
    fn get_musiclist_info(&self) -> MusicListInfo {
        MusicListInfo {
            id: 0,
            name: self.title.clone(),
            art_pic: self.pic.clone(),
            desc: self.info.clone(),
            extra: Some(ExtraInfo {
                play_count: Some(self.playnum),
                music_count: None,
            }),
        }
    }

    fn get_music_aggregators<'a>(
        &'a self,
        page: u32,
        limit: u32,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                    Output = Result<Vec<crate::music_aggregator::MusicAggregator>, anyhow::Error>,
                > + Send
                + '_,
        >,
    > {
        let playlist_id = self.musiclist_id.clone();
        Box::pin(async move {
            get_musics_of_music_list(&playlist_id, page, limit)
                .await
                .map(|(_info, musics)| {
                    musics
                        .into_iter()
                        .map(|m| Box::new(SearchMusicAggregator::from_music(m)) as MusicAggregator)
                        .collect()
                })
        })
    }

    fn source(&self) -> String {
        KUWO.to_string()
    }
}

pub async fn get_musics_of_music_list(
    playlist_id: &str,
    page: u32,
    limit: u32,
) -> Result<(MusicList, Vec<Music>), anyhow::Error> {
    let url = gen_get_musics_url(playlist_id, page, limit);

    let mut musiclist: MusicListDetail = CLIENT.get(url).send().await?.json().await?;
    musiclist.musiclist_id = playlist_id.to_string();

    let mut musics = Vec::new();
    mem::swap(&mut musiclist.musiclist, &mut musics);

    let musics: Vec<Music> = musics
        .into_iter()
        .map(|mut music| {
            let qualities = process_qualities(KuWoQuality::parse_quality(&music.minfo));
            let default_quality = qualities.first().cloned().unwrap_or_default();
            music.quality = qualities;
            music.default_quality = default_quality;

            Box::new(music) as Music
        })
        .collect();

    Ok((Box::new(musiclist), musics))
}

#[tokio::test]
async fn test_get_musics() {
    let start_time = std::time::Instant::now();
    let playlists = search_music_list("张惠妹", 1, 30).await.unwrap();
    let first = playlists.first().unwrap();
    let info = first.get_musiclist_info();
    println!("{}", info);
    let musics = first.get_music_aggregators(1, 30).await.unwrap();
    musics.into_iter().for_each(|m| {
        println!("{}", m.get_default_music().get_music_info());
    });
    let elapsed_time = start_time.elapsed();
    println!("Elapsed time: {:?}", elapsed_time);
}
