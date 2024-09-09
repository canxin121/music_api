use crate::refactor::adapter::{kuwo::model::KuwoMusics, CLIENT};

pub async fn search_kuwo_musics(
    content: &str,
    page: u32,
    limit: u32,
) -> Result<KuwoMusics, anyhow::Error> {
    assert!(page >= 1, "Page must be greater than 0");

    let url = format!("http://search.kuwo.cn/r.s?client=kt&all={}&pn={}&rn={}&uid=794762570&ver=kwplayer_ar_9.2.2.1&vipver=1&show_copyright_off=1&newver=1&ft=music&cluster=0&strategy=2012&encoding=utf8&rformat=json&vermerge=1&mobi=1&issubtitle=1",urlencoding::encode(content),page-1,limit);

    let result: KuwoMusics = CLIENT.get(&url).send().await?.json().await?;
    Ok(result)
}

#[tokio::test]
async fn test_search_single_music() {
    let musics = search_kuwo_musics("张惠妹", 1, 30).await.unwrap().abslist;

    musics.iter().for_each(|m| {
        println!("{:?}", m.album_pic());
        println!("{:?}", m.artist_pic());
    });
    println!("length:{}", musics.len())
}
