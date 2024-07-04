use crate::{music_list::MusicListTrait, MusicListInfo};

use super::SqlFactory;

pub struct SqlMusicList {
    info: MusicListInfo,
}
impl SqlMusicList {
    pub fn new(info: MusicListInfo) -> Self {
        Self { info }
    }
}

impl MusicListTrait for SqlMusicList {
    fn get_musiclist_info(&self) -> crate::MusicListInfo {
        self.info.clone()
    }

    fn get_music_aggregators<'b>(
        &'b self,
        page: u32,
        _limit: u32,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                    Output = Result<Vec<crate::music_aggregator::MusicAggregator>, anyhow::Error>,
                > + Send
                + '_,
        >,
    > {
        // 如果不是第一页，直接返回空
        // 这是为了防止请求后续页时，错误的返回了重复的音乐
        // 导致调用方认为仍有后续页面，不断请求
        if page != 1 {
            return Box::pin(async move { Ok(vec![]) });
        } else {
            Box::pin(async move { SqlFactory::get_all_musics(&self.info).await })
        }
    }

    fn source(&self) -> String {
        "Local".to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        factory::online_factory::aggregator_search, MusicFuzzFilter, MusicListInfo, SqlFactory, WANGYI,
    };
    macro_rules! init_test_env {
        ($db_name:expr) => {{
            let path = format!("./_data/{}", $db_name);
            // 如果已存在，先删除
            if std::path::Path::new(&path).exists() {
                std::fs::remove_file(&path).unwrap();
            }
            SqlFactory::init_from_path(&path).await.unwrap();
            let mut aggregator_search = aggregator_search::AggregatorOnlineFactory::new();
            aggregator_search
                .search_music_aggregator(
                    &[WANGYI.to_string()],
                    "张国荣",
                    1,
                    30,
                    Some(&MusicFuzzFilter {
                        name: None,
                        artist: vec!["张国荣".to_string()],
                        album: None,
                    }),
                )
                .await
                .unwrap();
            aggregator_search.aggregators.iter().for_each(|aggregator| {
                println!("{}", aggregator);
            });

            let musiclist_info = MusicListInfo {
                id: 0,
                name: "歌单1".to_string(),
                art_pic: "".to_string(),
                desc: "".to_string(),
                extra: None,
            };
            // 创建歌单
            SqlFactory::create_musiclist(&vec![musiclist_info.clone()])
                .await
                .unwrap();

            // 插入音乐
            SqlFactory::add_musics(
                &musiclist_info.name,
                &aggregator_search.get_aggregator_refs(),
            )
            .await
            .unwrap();
            (musiclist_info, aggregator_search)
        }};
    }
    #[tokio::test]
    async fn test_get() {
        let (_musiclist_info, _aggregator_search) = init_test_env!("sql_music_list_test_get.db");
        let musiclists = SqlFactory::get_all_musiclists().await.unwrap();
        let first = musiclists.first().unwrap();
        let aggs = first.get_music_aggregators(1, 1).await.unwrap();
        println!("{}", aggs.len());
    }
}
