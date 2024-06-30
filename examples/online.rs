use music_api::{factory::online_factory::aggregator_search, ALL};
// 纯在线示例
#[tokio::main]
async fn main() {
    // 由于需要对不同平台的搜索结果进行聚合，所以aggregators被封装于AggregatorOnlineFactory中
    let mut aggregator_search = aggregator_search::AggregatorOnlineFactory::new();
    aggregator_search
        .search_music_aggregator(&[ALL.to_string()], "张国荣", 1, 30, None)
        .await
        .unwrap();

    aggregator_search.aggregators.iter().for_each(|aggregator| {
        println!("{}", aggregator);
    });

    let aggregator = aggregator_search.aggregators.first().unwrap();
    // 获取一个aggregator的默认音乐
    let music = aggregator.get_default_music();
    // 获取歌词
    let lyric = music.fetch_lyric().await.unwrap();
    println!("{}", lyric);
    
    // 尝试获取专辑，并非所有音乐都有专辑
    if let Ok((album, mut aggregators)) = aggregator.fetch_album(1, 30).await {
        // 这里的专辑就是一个MusicList
        println!("专辑：{}, 来源：{}", album, album.source());
        // 获取其中的更多音乐,如果还有剩余的话
        if let Ok(musics) = album.get_music_aggregators(2, 30).await {
            musics.iter().for_each(|music| {
                println!("{}", music);
            });
        } else {
            println!("没有更多音乐了");
        }

        // 使用aggregator获取来换源
        // 这里的aggregator由于是某一个平台的album返回的，内部一定只有当前平台的歌曲来源
        // 通过使用fetch可以尝试换源
        for aggregator in &mut aggregators {
            let _musics = aggregator
                .fetch_musics([ALL.to_string()].to_vec())
                .await
                .unwrap();
            println!("{}", aggregator);
        }
    }
}
