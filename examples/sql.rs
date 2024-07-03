use music_api::{AggregatorOnlineFactory, MusicListInfo, SqlFactory, ALL, KUWO, WANGYI};
// sql+在线示例
#[tokio::main]
async fn main() {
    // 使用前必须先初始化数据库POOL链接池
    SqlFactory::init_from_path("_data/example.db")
        .await
        .unwrap();

    // 创建一个新的歌单
    let new_musiclist_info = MusicListInfo {
        id: 0,
        name: "test".to_string(),
        art_pic: "".to_string(),
        desc: "".to_string(),
        // extra主要用于在线搜索的到的歌单，可能包含歌曲数目和播放量信息
        extra: None,
    };
    SqlFactory::create_musiclist(&vec![new_musiclist_info.clone()])
        .await
        .unwrap();
    // 从KUWO搜索一些音乐，然后添加到歌单中
    let mut aggregator_search = AggregatorOnlineFactory::new();
    aggregator_search
        .search_music_aggregator(&[KUWO.to_string()], "张惠妹", 1, 30, None)
        .await
        .unwrap();
    SqlFactory::add_musics(
        &new_musiclist_info.name,
        &aggregator_search.get_aggregator_refs(),
    )
    .await
    .unwrap();

    // 获取歌单中的音乐
    // 在获取sql中的全部音乐时，默认只先获取默认音源的数据来缩短总耗时
    // 但是MusicAggregator的get方法均会在调用时若内部无则尝试调用sql查询
    // 而fetch方法均是内部->sql->网络,通过网络查询后会更新sql中的数据

    let mut aggregators = SqlFactory::get_all_musics(&new_musiclist_info)
        .await
        .unwrap();
    for aggregator in &mut aggregators {
        aggregator
            .fetch_musics(vec![ALL.to_string()])
            .await
            .unwrap();
    }

    // 验证数据库中的歌曲是否包含了WANGYI数据
    let aggregators = SqlFactory::get_all_musics(&new_musiclist_info)
        .await
        .unwrap();
    assert!(aggregators
        .iter()
        .find(|a| a.get_available_sources().contains(&WANGYI.to_string()))
        .is_some());
}
