use sea_orm::{
    ColumnTrait as _, Condition, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, Set,
    Unchanged,
};
use serde::{Deserialize, Serialize};

use crate::refactor::data::{
    get_db,
    models::{
        music_aggregator,
        playlist::{self, PlayListSubscription, PlayListSubscriptionVec},
        playlist_music_junction,
    },
};
use anyhow::Result;

use super::{music_aggregator::MusicAggregator, MusicServer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaylistType {
    // 用户自己创建的歌单(来自平台或数据库)
    UserPlaylist,
    // 专辑
    Album,
}

/// 歌单
/// 一个歌单可以是用户自己创建的(数据库中)， 也可以是来自其他平台(在线歌单)
/// 使用 `server` 字段来区分
/// 特殊的， 如果是新建一个数据库歌单， `identity` 字段为空， 此时save将插入数据库
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub from_db: bool,
    pub server: Option<MusicServer>,
    #[serde(rename = "type")]
    pub type_field: PlaylistType,
    pub identity: String,
    pub name: String,
    pub summary: Option<String>,
    pub cover: Option<String>,
    pub creator: Option<String>,
    pub creator_id: Option<String>,
    pub play_time: Option<i64>,
    pub music_num: Option<i64>,
    pub subscription: Option<Vec<PlayListSubscription>>,
}

impl From<playlist::Model> for Playlist {
    fn from(value: playlist::Model) -> Self {
        Self {
            from_db: true,
            server: None,
            type_field: PlaylistType::UserPlaylist,
            identity: value.id.to_string(),
            name: value.name,
            summary: value.summary,
            cover: value.cover,
            creator: None,
            creator_id: None,
            play_time: None,
            music_num: None,
            subscription: value.subscriptions.and_then(|s| Some(s.0)),
        }
    }
}

impl Playlist {
    /// 创建一个新的歌单， 但是不会保存到数据库
    pub fn new(
        name: String,
        summary: Option<String>,
        cover: Option<String>,
        subscriptions: Vec<PlayListSubscription>,
    ) -> Self {
        Self {
            from_db: false,
            server: None,
            type_field: PlaylistType::UserPlaylist,
            identity: String::with_capacity(0),
            name,
            summary,
            cover,
            creator: None,
            creator_id: None,
            play_time: None,
            music_num: None,
            subscription: Some(subscriptions),
        }
    }

    pub async fn find_in_db(id: i64) -> Option<Self> {
        let db = get_db().await.expect("Database is not inited.");
        let model: Option<playlist::Model> = playlist::Entity::find_by_id(id)
            .one(&db)
            .await
            .expect("Failed to find playlist by id.");
        model.and_then(|m| Some(m.into()))
    }

    pub async fn update_to_db(&self) -> Result<Self> {
        if !self.from_db || self.identity.is_empty() {
            return Err(anyhow::anyhow!(
                "Can't update playlist from non-database server."
            ));
        }
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        // 如果是数据库歌单， 且有id， 则更新数据库中的歌单
        if let Ok(id) = self.identity.parse::<i64>() {
            let playlist = playlist::ActiveModel {
                id: Set(id),
                order: Unchanged(0),
                name: Set(self.name.clone()),
                summary: Set(self.summary.clone()),
                cover: Set(self.cover.clone()),
                subscriptions: Set(self
                    .subscription
                    .clone()
                    .and_then(|s| Some(PlayListSubscriptionVec(s)))),
            };
            let model = playlist::Entity::update(playlist).exec(&db).await?;
            Ok(model.into())
        } else {
            return Err(anyhow::anyhow!("Invalid playlist id of Database playlist."));
        }
    }

    pub async fn insert_to_db(&self) -> Result<i64> {
        if self.from_db && !self.identity.is_empty() {
            return Err(anyhow::anyhow!("Playlist from db, can't insert."));
        }

        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        let order = playlist::Entity::find().count(&db).await?;
        let playlist = playlist::ActiveModel::new(
            self.name.clone(),
            self.summary.clone(),
            self.cover.clone(),
            order as i64,
        );

        let last_id = playlist::Entity::insert(playlist)
            .exec(&db)
            .await?
            .last_insert_id;
        Ok(last_id)
    }

    /// 从数据库中删除一个歌单, 同时删除歌单和音乐的关联
    /// 如果是其他平台的歌单， 则无法删除
    /// 注意这将取走歌单的所有权， ffi时应当注意生命周期
    pub async fn del_from_db(self) -> Result<()> {
        if !self.from_db {
            return Err(anyhow::anyhow!(
                "Can't delete playlist from non-database server."
            ));
        }
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        playlist::Entity::delete_by_id(self.identity.parse::<i64>()?)
            .exec(&db)
            .await?;
        Ok(())
    }

    /// 从数据库中获取所有歌单
    pub async fn get_from_db() -> Result<Vec<Self>> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        let models = playlist::Entity::find().all(&db).await?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    /// 将音乐添加到数据库歌单中
    pub async fn add_aggs_to_db(&self, music_aggs: &Vec<MusicAggregator>) -> Result<()> {
        if !self.from_db {
            return Err(anyhow::anyhow!(
                "Can't add music aggregators to non-database playlist"
            ));
        }

        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        let count =
            playlist_music_junction::Entity::find()
                .filter(Condition::all().add(
                    playlist_music_junction::Column::PlaylistId.eq(self.identity.parse::<i64>()?),
                ))
                .count(&db)
                .await?;
        let mut order = count as i64;
        let mut junctions = Vec::with_capacity(music_aggs.len());

        for music_agg in music_aggs {
            music_agg.insert_to_db().await?;
            let junction = playlist_music_junction::ActiveModel::new(
                self.identity.parse::<i64>()?,
                music_agg.identity(),
                order,
            );
            order += 1;
            junctions.push(junction);
        }
        playlist_music_junction::Entity::insert_many(junctions)
            .on_conflict_do_nothing()
            .exec(&db)
            .await?;
        Ok(())
    }

    /// 获取数据库歌单中的音乐
    pub async fn get_musics_from_db(&self) -> Result<Vec<MusicAggregator>> {
        if !self.from_db {
            return Err(anyhow::anyhow!(
                "Can't get music from non-database playlist"
            ));
        }

        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        let id = self.identity.parse::<i64>()?;
        let playlist = playlist::Entity::find_by_id(id)
            .one(&db)
            .await?
            .ok_or(anyhow::anyhow!("Can't find playlist in db"))?;
        let aggs_links = playlist
            .find_related(music_aggregator::Entity)
            .all(&db)
            .await?;
        let mut aggs = Vec::with_capacity(aggs_links.len());
        for agg_link in aggs_links {
            aggs.push(agg_link.get_music_aggregator(db.clone()).await?)
        }
        Ok(aggs)
    }
}

#[cfg(test)]
mod test_playlist {
    use sea_orm_migration::MigratorTrait as _;
    use serial_test::serial;

    use crate::refactor::data::{close_db, migrations::Migrator, set_db};

    use super::*;
    async fn re_init_db() {
        // 初始化log
        // tracing_subscriber::fmt::init();

        let db_file = "./test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();

        set_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
        Migrator::up(&get_db().await.unwrap(), None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_to_db() {
        re_init_db().await;
        let playlist = Playlist::new(
            "test".to_string(),
            Some("test playlist".to_string()),
            None,
            vec![],
        );
        println!("{:?}", playlist);
        playlist.insert_to_db().await.unwrap();
        let playlists = Playlist::get_from_db().await.unwrap();
        assert!(playlists.len() == 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_del_from_db() {
        re_init_db().await;
        let playlist = Playlist::new(
            "test".to_string(),
            Some("test playlist".to_string()),
            None,
            vec![],
        );
        playlist.insert_to_db().await.unwrap();
        let playlists = Playlist::get_from_db().await.unwrap();
        assert!(playlists.len() == 1);
        playlists
            .into_iter()
            .next()
            .unwrap()
            .del_from_db()
            .await
            .unwrap();
        assert!(Playlist::get_from_db().await.unwrap().len() == 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_add_aggs_to_db1() {
        re_init_db().await;
        let playlist = Playlist::new(
            "test".to_string(),
            Some("test playlist".to_string()),
            None,
            vec![],
        );
        playlist.insert_to_db().await.unwrap();

        let playlist = Playlist::get_from_db()
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        let aggs = vec![];
        let aggs =
            MusicAggregator::search(aggs, vec![MusicServer::Kuwo], "Aimer".to_string(), 1, 30)
                .await
                .unwrap();

        playlist.add_aggs_to_db(&aggs).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_add_aggs_to_db2() {
        re_init_db().await;
        let playlists = Playlist::search(vec![MusicServer::Kuwo], "Aimer".to_string(), 1, 30);
        let playlist = playlists.await.unwrap().into_iter().next().unwrap();
        let aggs = playlist.fetch_musics(1, 30).await.unwrap();

        let insert_id = playlist.insert_to_db().await.unwrap();
        let playlist = Playlist::find_in_db(insert_id).await.unwrap();
        // 测量此处耗时
        let start = std::time::Instant::now();
        playlist.add_aggs_to_db(&aggs).await.unwrap();
        println!("Add aggs to db cost: {:?}", start.elapsed());
    }
}
