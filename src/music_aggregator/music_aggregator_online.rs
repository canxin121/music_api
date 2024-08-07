use std::{collections::HashMap, pin::Pin};

use futures::Future;

use crate::{
    factory::online_factory::aggregator_search,
    filter::{MusicFilter, MusicFuzzFilter},
    music_aggregator::Music,
    music_list::MusicList,
};

use super::{MusicAggregator, MusicAggregatorTrait};
// 由搜索得来的音乐集合，无需存储，只需对本身负责
#[derive(Clone)]
pub struct SearchMusicAggregator {
    pub filter: MusicFuzzFilter,
    pub default_source: String,
    // 从多个音乐中生成音乐集合, 默认使用第一个音乐的source作为默认source
    // 注意这里的音乐musics必须是同一首音乐（能通过filter match），只是不同的source
    pub musics: HashMap<String, Music>,
}

impl SearchMusicAggregator {
    pub fn from_music(music: Music) -> Self {
        let info = music.get_music_info();
        let filter = MusicFuzzFilter {
            name: Some(info.name.clone()),
            artist: info.artist.clone(),
            album: None,
        };
        let mut musics = HashMap::new();
        musics.insert(info.source.to_string(), music);
        Self {
            filter,
            default_source: info.source.to_string(),
            musics,
        }
    }
}

impl MusicAggregatorTrait for SearchMusicAggregator {
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
            if self.musics.contains_key(&source) {
                self.default_source = source;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Source not found"))
            }
        })
    }

    fn get_music(
        &mut self,
        source: &str,
    ) -> Pin<Box<dyn Future<Output = Option<&Music>> + std::marker::Send + '_>> {
        let music = self.musics.get(source);
        Box::pin(async move { music })
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
        Box::pin(async move {
            let agg = find_best_match_music_aggregator(&info, &sources, Some(&self.filter)).await?;
            self.join(agg).await?;
            Ok(self.get_all_musics())
        })
    }

    fn belong_to(&self, music: &Music) -> bool {
        if self.musics.contains_key(&music.source()) {
            return false;
        }
        let info = music.get_music_info();
        if self.filter.matches(&info) {
            if let Some(name) = &self.filter.name {
                return name.len() == info.name.len();
            }
        }
        false
    }

    fn add_music(
        &mut self,
        music: Music,
    ) -> Pin<Box<dyn futures::Future<Output = Result<(), anyhow::Error>> + std::marker::Send>> {
        let info = music.get_music_info();
        self.musics.insert(info.source.to_string(), music);
        Box::pin(async { Ok(()) })
    }

    fn get_default_music(&self) -> &Music {
        self.musics.get(&self.default_source).unwrap()
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
        Box::pin(async { Ok(self.get_default_music().fetch_lyric().await?) })
    }

    fn fetch_album(
        &self,
        page: u32,
        limit: u32,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<(MusicList, Vec<MusicAggregator>), anyhow::Error>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move { Ok(self.get_default_music().fetch_album(page, limit).await?) })
    }
}

pub async fn merge_music_aggregators(
    aggregators: Vec<MusicAggregator>,
) -> Result<Vec<MusicAggregator>, anyhow::Error> {
    let mut result: Vec<MusicAggregator> = Vec::new();
    for aggregator in aggregators {
        match result.iter_mut().find(|a| *a == &aggregator) {
            Some(result_agg) => {
                result_agg.join(aggregator).await?;
            }
            None => {
                result.push(aggregator);
            }
        }
    }
    Ok(result)
}

pub async fn find_best_match_music_aggregator(
    info: &crate::MusicInfo,
    sources: &[String],
    filter: Option<&(dyn MusicFilter + Send + Sync)>,
) -> Result<MusicAggregator, anyhow::Error> {
    let mut aggregator_search = aggregator_search::AggregatorOnlineFactory::new();
    aggregator_search
        .search_music_aggregator(
            sources,
            &format!("{} {}", info.name, info.artist.join(" ")),
            1,
            50,
            filter,
        )
        .await?;
    let mut best_match: Option<MusicAggregator> = None;
    // 名称长度相同的优先
    // 名称长度更长的次之
    // 否则取第一个
    if aggregator_search.aggregators.len() > 0 {
        if let Some(best) = aggregator_search
            .aggregators
            .iter()
            .find(|a| a.get_default_music().get_music_info().name.len() == info.name.len())
        {
            best_match = Some(best.clone());
        } else if let Some(best) = aggregator_search
            .aggregators
            .iter()
            .find(|a| a.get_default_music().get_music_info().name.len() > info.name.len())
        {
            best_match = Some(best.clone());
        } else if let Some(first) = aggregator_search.aggregators.first() {
            best_match = Some(first.clone());
        }
    }
    best_match.ok_or(anyhow::anyhow!("No music aggregator found"))
}

#[cfg(test)]
mod test {
    use crate::{
        factory::online_factory::{aggregator_search, AggregatorOnlineFactory},
        filter::MusicFuzzFilter,
        platform_integrator::{wangyi::WANGYI, ALL},
    };

    #[tokio::test]
    async fn test() {
        let mut aggregator_search = aggregator_search::AggregatorOnlineFactory::new();
        aggregator_search
            .search_music_aggregator(
                &vec![WANGYI.to_string()],
                "张惠妹",
                1,
                5,
                Some(&MusicFuzzFilter {
                    name: None,
                    artist: vec!["张惠妹".to_string()],
                    album: None,
                }),
            )
            .await
            .unwrap();
        for aggregator in &mut aggregator_search.aggregators {
            let _ = aggregator
                .fetch_musics([ALL.to_string()].to_vec())
                .await
                .unwrap();
            println!("{}", aggregator);
        }
        aggregator_search
            .aggregators
            .iter()
            .find(|a| a.get_available_sources().contains(&WANGYI.to_string()))
            .unwrap();
        let first = aggregator_search.aggregators.first().clone().unwrap();
        let lyric = first.fetch_lyric().await.unwrap();
        println!("{}", lyric);
    }

    #[tokio::test]
    async fn test_album() {
        let mut aggregator_search = AggregatorOnlineFactory::new();
        aggregator_search
            .search_music_aggregator(
                &vec![ALL.to_string()],
                "张惠妹",
                1,
                5,
                Some(&MusicFuzzFilter {
                    name: None,
                    artist: vec!["张惠妹".to_string()],
                    album: None,
                }),
            )
            .await
            .unwrap();
        let first = aggregator_search.aggregators.first().clone().unwrap();
        let (music_list, musics) = first.fetch_album(1, 5).await.unwrap();
        println!("{}", music_list);
        musics.iter().for_each(|m| println!("{}", m));
    }
}
