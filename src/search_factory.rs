use crate::{kuwo, KuwoSearch, Music, SearchTrait};

pub struct SearchFactory {}
impl SearchFactory {
    pub async fn search(
        source: &str,
        content: &str,
        page: u32,
    ) -> Result<Vec<Music>, anyhow::Error> {
        match source {
            kuwo::KUWO => Ok(KuwoSearch {}.search(content, page, 30).await?),
            _ => Err(anyhow::anyhow!("Not Supportted source")),
        }
    }
}

#[tokio::test]
async fn test() {
    use std::time::Instant;

    let start_time = Instant::now(); // 记录开始时间

    let musics = SearchFactory::search(&kuwo::KUWO, "邓紫棋", 1)
        .await
        .unwrap();

    musics.iter().for_each(|m| {
        println!("{}", m.get_music_info());
    });

    let elapsed_time = start_time.elapsed(); // 计算运行时间
    println!("代码运行时间: {:?}", elapsed_time);
}
