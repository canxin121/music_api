use futures::Future;
use std::pin::Pin;

use crate::{music_aggregator::MusicAggregator, music_list::MusicListTrait, MusicListInfo};

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
        _page: u32,
        _limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<MusicAggregator>, anyhow::Error>> + 'b>> {
        Box::pin(async move { SqlFactory::get_all_musics(&self.info).await })
    }

    fn source(&self) -> String {
        "Local".to_string()
    }
}
