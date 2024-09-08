use futures::{join, stream, StreamExt as _};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::{platform_integrator::kuwo::kuwo_quality::process_qualities, util::CLIENT, Music};

use super::{kuwo_music::KuwoMusic, kuwo_pic::get_pic_url, kuwo_quality::KuWoQuality};

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    abslist: Vec<KuwoMusic>,
}

pub async fn search_single_music(
    content: &str,
    page: u32,
    limit: u32,
) -> Result<Vec<Music>, anyhow::Error> {
    assert!(page >= 1, "Page must be greater than 0");
    let url = gen_search_url(content, page, limit);
    let musics: SearchResult = CLIENT.get(&url).send().await?.json().await?;

    let music_futures = musics.abslist.into_iter().map(|music| {
        task::spawn(async move {
            // 并行运行 get_pic_url, 和 parse_quality
            let (pic_result, quality_result) = join!(get_pic_url(&music.music_rid), async {
                let mut qualities: Vec<KuWoQuality> = KuWoQuality::parse_quality(&music.minfo);
                qualities.append(&mut KuWoQuality::parse_quality(&music.n_minfo));
                qualities = process_qualities(qualities);
                qualities
            });

            // 检查每个任务的结果
            let pic_url = pic_result.unwrap_or(String::with_capacity(0));
            // let lyric_string = lyric_result.unwrap_or(String::with_capacity(0));
            let qualities = quality_result;

            let mut music_info = music;
            music_info.pic = pic_url;
            match qualities.first() {
                Some(quality) => {
                    music_info.default_quality = quality.clone();
                }
                None => {
                    return Err(anyhow::anyhow!("No Quality Fount, Can't Play"));
                }
            }
            music_info.quality = qualities;
            // music_info.lyric = lyric_string;
            Ok::<_, anyhow::Error>(Box::new(music_info) as Music)
        })
    });

    let results = stream::FuturesUnordered::from_iter(music_futures)
        .collect::<Vec<_>>()
        .await;

    let music_vec: Vec<Music> = results
        .into_iter()
        .filter_map(|res| match res {
            Ok(Ok(music)) => Some(music),
            _ => None,
        })
        .collect();

    Ok(music_vec)
}

pub fn gen_search_url(content: &str, page: u32, limit: u32) -> String {
    return format!("http://search.kuwo.cn/r.s?client=kt&all={}&pn={}&rn={}&uid=794762570&ver=kwplayer_ar_9.2.2.1&vipver=1&show_copyright_off=1&newver=1&ft=music&cluster=0&strategy=2012&encoding=utf8&rformat=json&vermerge=1&mobi=1&issubtitle=1",urlencoding::encode(content),page-1,limit);
}

#[tokio::test]
async fn test_search_single_music() {
    let musics = search_single_music("张惠妹", 1, 30).await.unwrap();
    musics.iter().for_each(|m| {
        println!("{}", m.get_music_info());
    });
    println!("length:{}", musics.len())
}
