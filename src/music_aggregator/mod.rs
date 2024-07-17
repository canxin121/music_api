pub mod music_aggregator_online;
pub mod music_aggregator_sql;

use std::{fmt::Display, pin::Pin};

use futures::Future;

use crate::{
    filter::{fuzzy_match, MusicFilter as _, MusicFuzzFilter},
    music_list::MusicList,
    Music,
};

pub type MusicAggregator = Box<dyn MusicAggregatorTrait + Send + Sync>;
impl Clone for MusicAggregator {
    fn clone(&self) -> Self {
        self.clone_()
    }
}
impl Display for MusicAggregator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]{}",
            self.get_available_sources().join(","),
            self.get_default_music().get_music_info()
        )
    }
}

impl PartialEq for MusicAggregator {
    fn eq(&self, other: &Self) -> bool {
        let self_info = self.get_default_music().get_music_info();
        let other_info = other.get_default_music().get_music_info();
        if !fuzzy_match(&self_info.name, &other_info.name) {
            return false;
        }
        if self_info.artist != other_info.artist {
            return false;
        }
        true
    }
}

// get => 直接返回实例内部或者sql本地的
// fetch => 实例内部不存在则从网络获取(会插入到实例内部和sql本地)
pub trait MusicAggregatorTrait {
    // 此处的id为自定义歌单中的id，是借由sql构造时传入的，与音乐平台无关的值
    fn get_music_id(&self) -> i64 {
        0
    }
    // clone的传递实现
    fn clone_(&self) -> MusicAggregator;

    // 插入一个新的源的Music
    fn add_music(
        &mut self,
        music: Music,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send + '_>>;

    // 判断一个Music是否属于此MusicAggregator
    fn belong_to(&self, music: &Music) -> bool;

    // 合并两个MusicAggregator
    fn join(
        &mut self,
        other: MusicAggregator,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send + '_>>
    where
        Self: std::marker::Send,
    {
        Box::pin(async move {
            let musics = other.get_all_musics();
            for music in musics {
                if self.get_music(&music.source()).await.is_none() {
                    self.add_music(music.clone_()).await?;
                }
            }
            Ok(())
        })
    }

    // 判断一个Music是否符合过滤器
    fn match_filter(&self, filter: &MusicFuzzFilter) -> bool {
        let music = self.get_default_music();
        filter.matches(&music.get_music_info())
    }

    // 设置默认来源
    fn set_default_source(
        &mut self,
        source: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send + '_>>;

    // 获取所有可用的源
    fn get_available_sources(&self) -> Vec<String>;

    // 获取默认使用的源
    fn get_default_source(&self) -> String;

    // 获取默认源的Music
    fn get_default_music(&self) -> &Music;

    // 获取对应源的Music
    // 为了便于不同场景下的储存，我们认为get_music是可以改变自身
    fn get_music(
        &mut self,
        source: &str,
    ) -> Pin<Box<dyn Future<Output = Option<&Music>> + Send + '_>>;

    // 获取所有可用的Music
    fn get_all_musics(&self) -> Vec<&Music>;

    // 获取所有拥有的Music的实例，而非引用
    fn get_all_musics_owned(&self) -> Vec<Music> {
        self.get_all_musics()
            .into_iter()
            .map(|x| x.clone_())
            .collect()
    }

    // 获取指定的Music
    fn fetch_musics(
        &mut self,
        sources: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<&Music>, anyhow::Error>> + std::marker::Send + '_>>;

    // 获取歌曲的歌词
    fn fetch_lyric(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<String, anyhow::Error>> + Send + '_>>;

    // 获取歌曲的专辑及其中的歌曲
    // 目前支持的平台均是一次性获取到所有歌曲，不必返回MusicList Trait Object
    // 这里的page，limit并不能完全保证所有平台都遵循，可能直接返回全部
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
    >;
}
