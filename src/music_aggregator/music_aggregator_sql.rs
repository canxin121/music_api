use std::{collections::HashMap, pin::Pin};

use futures::Future;

use crate::{
    factory::sql_factory::SqlFactory,
    filter::{MusicFilter, MusicFuzzFilter},
    platform_integrator::{ALL, PLATFORM_NUM},
    Music, MusicListInfo,
};

use super::{
    music_aggregator_online::find_best_match_music_aggregator, MusicAggregator,
    MusicAggregatorTrait,
};

// sql查询的到的本地音乐合集，需对sql储存负责
#[derive(Clone)]
pub struct SqlMusicAggregator {
    // 自定义歌单中的索引
    pub id: i64,
    // 音乐合集所属的自定义歌单的引用
    pub music_list_info: MusicListInfo,
    pub filter: MusicFuzzFilter,
    pub default_source: String,
    pub musics: HashMap<String, Music>,
}

impl SqlMusicAggregator {
    pub fn from_musics(
        id: i64,
        music_list_info: MusicListInfo,
        default_source: String,
        musics: Vec<Music>,
    ) -> Self {
        let default_music = musics
            .iter()
            .find(|m| m.source() == default_source)
            .unwrap_or_else(|| musics.first().unwrap());
        let default_filter = MusicFuzzFilter {
            name: Some(default_music.get_music_info().name.clone()),
            artist: default_music.get_music_info().artist.clone(),
            album: None,
        };
        let musics_map = musics
            .into_iter()
            .map(|m| (m.source().to_string(), m))
            .collect();
        Self {
            filter: default_filter,
            default_source,
            musics: musics_map,
            music_list_info,
            id,
        }
    }
}

impl MusicAggregatorTrait for SqlMusicAggregator {
    fn get_music_id(&self) -> i64 {
        self.id
    }
    fn get_default_music(&self) -> &Music {
        self.musics.get(&self.default_source).unwrap()
    }

    fn get_available_sources(&self) -> Vec<String> {
        self.musics.keys().map(|s| s.to_string()).collect()
    }

    fn get_default_source(&self) -> String {
        self.default_source.clone()
    }

    fn set_default_source(
        &mut self,
        source: &str,
    ) -> Pin<Box<dyn futures::Future<Output = Result<(), anyhow::Error>> + std::marker::Send + '_>>
    {
        let source = source.to_string();
        Box::pin(async move {
            if self.get_available_sources().contains(&source) {
                SqlFactory::change_music_default_source(
                    &self.music_list_info.name,
                    vec![self.id],
                    vec![source.to_string()],
                )
                .await?;
                self.default_source = source;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Source Not Found"))
            }
        })
    }

    fn get_music(
        &mut self,
        source: &str,
    ) -> Pin<Box<dyn Future<Output = Option<&Music>> + Send + '_>> {
        let source = source.to_string();

        if !self.get_available_sources().contains(&source) {
            return Box::pin(async { None });
        }

        Box::pin(async move {
            if !self.musics.contains_key(&source) {
                if let Ok(aggregator) =
                    SqlFactory::get_music_by_id(&self.music_list_info, self.id, &[&source]).await
                {
                    if let Some(music) = aggregator.get_all_musics_owned().pop() {
                        self.musics.insert(source.clone(), music);
                    }
                }
            }
            self.musics.get(&source)
        })
    }

    fn fetch_musics(
        &mut self,
        sources: Vec<String>,
    ) -> Pin<
        Box<
            dyn futures::Future<Output = Result<Vec<&Music>, anyhow::Error>>
                + std::marker::Send
                + '_,
        >,
    > {
        let info = self.get_default_music().get_music_info();
        // 如果self.musics不包含所有可用源的音乐，需要从数据库中获取
        Box::pin(async move {
            // 实例内不包含sql中的所有音乐，则从sql中获取
            if self.musics.len() != self.get_available_sources().len() {
                // 直接获取所有源
                let aggregator =
                    SqlFactory::get_music_by_id(&self.music_list_info, self.id, &[ALL]).await?;
                // 更新本身的音乐
                let musics = aggregator.get_all_musics_owned();
                self.musics = musics
                    .into_iter()
                    .map(|m| (m.source().to_string(), m))
                    .collect();
            }
            // 从sql中获取的音乐不包含所有源，需要从网络获取，但是不一定成功
            if self.musics.len() != PLATFORM_NUM {
                let agg =
                    find_best_match_music_aggregator(&info, &sources, Some(&self.filter)).await?;
                self.join(agg).await?;
                SqlFactory::replace_musics(
                    &self.music_list_info.name,
                    vec![self.id],
                    vec![self.clone_()],
                )
                .await?;
            }
            Ok(self.musics.values().collect())
        })
    }

    fn belong_to(&self, music: &Music) -> bool {
        self.filter.matches(&music.get_music_info())
    }

    fn add_music(
        &mut self,
        music: Music,
    ) -> Pin<Box<dyn futures::Future<Output = Result<(), anyhow::Error>> + std::marker::Send + '_>>
    {
        let music_list_name = self.music_list_info.name.clone();
        self.musics.insert(music.source().to_string(), music);
        let music_aggregator_clone = MusicAggregatorTrait::clone_(self);
        // 将新的源添加到数据库中
        Box::pin(async move {
            SqlFactory::replace_musics(
                &music_list_name,
                vec![self.id],
                vec![music_aggregator_clone],
            )
            .await?;
            Ok(())
        })
    }

    fn get_all_musics(&self) -> Vec<&Music> {
        self.musics.values().collect()
    }
    fn clone_(&self) -> MusicAggregator {
        let self_clone = self.clone();
        Box::new(self_clone)
    }

    fn fetch_lyric(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<String, anyhow::Error>> + Send + '_>> {
        let info = self.get_default_music().get_music_info();
        if info.lyric.is_some() {
            return Box::pin(async move { Ok(info.lyric.clone().unwrap()) });
        }
        Box::pin(async move {
            let music = self.get_default_music().clone();
            let mut info = music.get_music_info();
            let lyric = music.fetch_lyric().await?;
            info.lyric = Some(lyric.clone());
            SqlFactory::change_music_info(&[music], vec![info]).await?;
            Ok(lyric)
        })
    }

    fn fetch_album(
        &self,
        page: u32,
        limit: u32,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        (crate::music_list::MusicList, Vec<MusicAggregator>),
                        anyhow::Error,
                    >,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move { Ok(self.get_default_music().fetch_album(page, limit).await?) })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        factory::{online_factory::aggregator_search, sql_factory::SqlFactory},
        filter::MusicFuzzFilter,
        platform_integrator::{kuwo::KUWO, wangyi::WANGYI, ALL},
        MusicListInfo,
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
    pub async fn test_sql_music_aggregator_fetch_music() {
        // 测试获取所有音乐源
        let (musiclist_info, _) = init_test_env!("test_sql_music_aggregator.db");
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        for mut music in musics {
            println!("Default: {}", music);
            let _ = music.fetch_musics(vec![KUWO.to_string()]).await;
            println!("After Fetch: {}", music);
        }

        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        for music in &musics {
            println!("After Fetch Default: {}", music);
        }
        let one = musics
            .iter()
            .find(|m| m.get_available_sources().contains(&KUWO.to_string()));
        assert!(one.is_some());

        // 测试切换默认来源
        let mut music = musics
            .iter()
            .find(|m| m.get_available_sources().len() > 1)
            .unwrap()
            .clone();

        music.set_default_source(&WANGYI).await.unwrap();
        let musics_ = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let music_ = musics_
            .iter()
            .find(|m| m.get_music_id() == music.get_music_id())
            .unwrap();
        assert!(music_.get_default_source() == WANGYI);
    }

    #[tokio::test]
    pub async fn test_sql_music_aggregator() {
        // 测试获取所有音乐源
        let (musiclist_info, _) = init_test_env!("test_sql_music_aggregator.db");
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        for mut music in musics {
            println!("Default: {}", music);
            music.fetch_musics(vec![ALL.to_string()]).await.unwrap();
            println!("After Fetch: {}", music);
        }

        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        for music in &musics {
            println!("After Fetch Default: {}", music);
        }
        let one = musics
            .iter()
            .find(|m| m.get_available_sources().contains(&KUWO.to_string()));
        assert!(one.is_some());

        // 测试切换默认来源
        let mut music = musics
            .iter()
            .find(|m| m.get_available_sources().len() > 1)
            .unwrap()
            .clone();

        music.set_default_source(&WANGYI).await.unwrap();
        let musics_ = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let music_ = musics_
            .iter()
            .find(|m| m.get_music_id() == music.get_music_id())
            .unwrap();
        assert!(music_.get_default_source() == WANGYI);
    }

    #[tokio::test]
    pub async fn test_sql_music_aggregator_get_lyric() {
        let (musiclist_info, _) = init_test_env!("test_sql_music_aggregator_get_lyric.db");
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let first = musics.first().unwrap();
        first.fetch_lyric().await.unwrap();
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let first = musics.first().unwrap();
        assert!(first.get_default_music().get_music_info().lyric.is_some());
    }
}
