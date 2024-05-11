use lazy_static::lazy_static;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use crate::{kuwo, KuwoSearch, Music, MusicInfo, MusicList, SearchTrait};

lazy_static! {
    pub static ref CLIENT: ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .build();
}

pub struct SearchFactory {}
impl SearchFactory {
    pub async fn search(
        source: &str,
        content: &str,
        page: u32,
    ) -> Result<Vec<Music>, anyhow::Error> {
        match source {
            kuwo::KUWO => Ok(KuwoSearch {}.search_song(content, page, 30).await?),
            _ => Err(anyhow::anyhow!("Not Supportted Source")),
        }
    }

    pub async fn search_album(
        music: &Music,
        page: u32,
    ) -> Result<(MusicList, Vec<Music>), anyhow::Error> {
        match music.source() {
            kuwo::KUWO => {
                Ok(kuwo::kuwo_album::get_music_album(music.get_album_info(), page).await?)
            }
            _ => Err(anyhow::anyhow!("Not Supported Source")),
        }
    }

    pub async fn search_music_list(
        source: &str,
        content: &str,
        page: u32,
    ) -> Result<Vec<(String, MusicList)>, anyhow::Error> {
        match source {
            kuwo::KUWO => Ok(kuwo::kuwo_music_list::search_music_list(content, page).await?),
            _ => Err(anyhow::anyhow!("Not Supported Source")),
        }
    }

    pub async fn get_musics_from_music_list(
        source: &str,
        payload: &str,
        page: u32,
    ) -> Result<Vec<Music>, anyhow::Error> {
        match source {
            kuwo::KUWO => Ok(kuwo::kuwo_music_list::get_musics_of_music_list(payload, page).await?),
            _ => Err(anyhow::anyhow!("Not Supported Source")),
        }
    }
}

#[tokio::test]
async fn test() {
    use std::time::Instant;

    let start_time = Instant::now();

    let musics = SearchFactory::search(&kuwo::KUWO, "邓紫棋", 1)
        .await
        .unwrap();

    musics.iter().for_each(|m| {
        println!("{}", m.get_music_info());
    });

    let elapsed_time = start_time.elapsed();
    println!("代码运行时间: {:?}", elapsed_time);
}

#[tokio::test]
async fn test_album() {
    let musics = SearchFactory::search(&kuwo::KUWO, "Taylor", 1)
        .await
        .unwrap();
    let music = musics.first().unwrap();
    println!(
        "{}-{}",
        music.get_music_info().name,
        music.get_music_info().album.unwrap()
    );
    let (musiclist, album_musics) = SearchFactory::search_album(music, 1).await.unwrap();
    println!(
        "{},{},{}",
        musiclist.name, musiclist.desc, musiclist.art_pic
    );

    album_musics.iter().for_each(|m| {
        println!("{}", m.get_music_info().name);
    });
    println!("{}", album_musics.first().unwrap().get_music_info())
}

#[tokio::test]
async fn test_music_list() {
    let result = SearchFactory::search_music_list(&kuwo::KUWO, "sia", 1)
        .await
        .unwrap();
    let (payload, musiclist) = result.first().unwrap();
    println!("{}", musiclist);
    let musics = SearchFactory::get_musics_from_music_list(&kuwo::KUWO, &payload, 1)
        .await
        .unwrap();
    let music_infos = musics
        .iter()
        .map(|m| m.get_music_info())
        .collect::<Vec<MusicInfo>>();
    music_infos.iter().for_each(|m| println!("{}", m));
    // musics
    //     .iter()
    //     .for_each(|m| println!("{}", m.get_music_info()));
}
