use crate::{
    filter::MusicFilter,
    music_aggregator::{music_aggregator_online::SearchMusicAggregator, MusicAggregator},
    platform_integrator::{
        kuwo::{self, KUWO},
        wangyi::{self, WANGYI},
        ALL,
    },
    Music,
};
use futures::{future::BoxFuture, FutureExt as _};

pub struct AggregatorOnlineFactory {
    pub aggregators: Vec<MusicAggregator>,
}

impl AggregatorOnlineFactory {
    pub fn new() -> AggregatorOnlineFactory {
        AggregatorOnlineFactory {
            aggregators: Vec::new(),
        }
    }

    pub fn get_aggregator_refs(&self) -> Vec<&MusicAggregator> {
        self.aggregators.iter().map(|x| x).collect()
    }

    pub(crate) async fn insert_musics(&mut self, mut musics: Vec<Music>) {
        let mut left_musics = Vec::new();
        for music in musics.drain(..) {
            let mut inserted = false;
            for collection in self.aggregators.iter_mut() {
                if collection.belong_to(&music) {
                    collection.add_music(music.clone_()).await.ok();
                    inserted = true;
                    break;
                }
            }
            if !inserted {
                left_musics.push(music);
            }
        }
        for music in left_musics {
            let collection = SearchMusicAggregator::from_music(music);
            self.aggregators.push(Box::new(collection));
        }
    }

    pub async fn search_music_aggregator(
        &mut self,
        sources: &[String],
        content: &str,
        page: u32,
        limit: u32,
        filter: Option<&(dyn MusicFilter + Send + Sync)>,
    ) -> Result<(), anyhow::Error> {
        if page < 1 {
            return Err(anyhow::anyhow!("Page must be greater than 0"));
        }

        async fn filter_musics(
            musics: Vec<Music>,
            filter: Option<&(dyn MusicFilter + Send + Sync)>,
        ) -> Vec<Music> {
            if let Some(f) = filter {
                musics
                    .into_iter()
                    .filter(|music| f.matches(&music.get_music_info()))
                    .collect()
            } else {
                musics
            }
        }

        async fn search_source(
            source_fn: impl FnOnce(&str, u32, u32) -> BoxFuture<Result<Vec<Music>, anyhow::Error>>,
            content: &str,
            filter: Option<&(dyn MusicFilter + Send + Sync)>,
            page: u32,
            limit: u32,
        ) -> Result<Vec<Music>, anyhow::Error> {
            let musics = source_fn(content, page, limit).await?;
            Ok(filter_musics(musics, filter).await)
        }

        let mut tasks: Vec<BoxFuture<Result<Vec<Music>, anyhow::Error>>> = vec![];

        if sources.contains(&ALL.to_string()) {
            tasks.push(
                search_source(
                    |content, page, limit| kuwo::search_single_music(content, page, limit).boxed(),
                    content,
                    filter,
                    page,
                    limit,
                )
                .boxed(),
            );
            tasks.push(
                search_source(
                    |content, page, limit| {
                        wangyi::search_single_music(content, page, limit).boxed()
                    },
                    content,
                    filter,
                    page,
                    limit,
                )
                .boxed(),
            );
        } else {
            for source in sources {
                match source.as_str() {
                    KUWO => {
                        tasks.push(
                            search_source(
                                |content, page, limit| {
                                    kuwo::search_single_music(content, page, limit).boxed()
                                },
                                content,
                                filter,
                                page,
                                limit,
                            )
                            .boxed(),
                        );
                    }
                    WANGYI => {
                        tasks.push(
                            search_source(
                                |content, page, limit| {
                                    wangyi::search_single_music(content, page, limit).boxed()
                                },
                                content,
                                filter,
                                page,
                                limit,
                            )
                            .boxed(),
                        );
                    }
                    _ => return Err(anyhow::anyhow!("Not Supported Source")),
                }
            }
        }

        let results = futures::future::join_all(tasks).await;

        for result in results {
            if let Ok(musics) = result {
                self.insert_musics(musics).await;
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::factory::online_factory::AggregatorOnlineFactory;
    use crate::filter::MusicFuzzFilter;
    use crate::platform_integrator::kuwo::KUWO;
    use crate::platform_integrator::wangyi::WANGYI;
    use crate::platform_integrator::ALL;

    #[tokio::test]
    async fn test_aggregator_search_all() {
        let mut aggregator_factory = AggregatorOnlineFactory::new();
        aggregator_factory
            .search_music_aggregator(
                &[ALL.to_string()],
                "张惠妹",
                1,
                50,
                Some(&MusicFuzzFilter {
                    name: None,
                    artist: vec!["张惠妹".to_string()],
                    album: None,
                }),
            )
            .await
            .unwrap();
        let aggregators = aggregator_factory.aggregators;
        for (i, aggregator) in aggregators.iter().enumerate() {
            println!("{i}.{}", aggregator);
        }
    }
    #[tokio::test]
    async fn test_aggregator_search_some() {
        let mut aggregator_factory = AggregatorOnlineFactory::new();
        aggregator_factory
            .search_music_aggregator(
                &[KUWO.to_string(), WANGYI.to_string()],
                "张惠妹",
                1,
                50,
                Some(&MusicFuzzFilter {
                    name: None,
                    artist: vec!["张惠妹".to_string()],
                    album: None,
                }),
            )
            .await
            .unwrap();
        let aggregators = aggregator_factory.aggregators;
        for (i, aggregator) in aggregators.iter().enumerate() {
            println!("{i}.{}", aggregator);
        }
    }
}
