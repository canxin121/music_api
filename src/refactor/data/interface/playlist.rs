use sea_orm::{
    ColumnTrait as _, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set, Unchanged,
};
use serde::{Deserialize, Serialize};

use crate::refactor::data::{
    get_db,
    models::{
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
    pub server: MusicServer,
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
            server: MusicServer::Database,
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
            server: MusicServer::Database,
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

    pub async fn update_to_db(&self)->Result<Self>{
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        // 如果是数据库歌单， 且有id， 则更新数据库中的歌单
        if self.server == MusicServer::Database && !self.identity.is_empty() {
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
        else{
            return Err(anyhow::anyhow!("Can't update playlist from non-database server."));
        }
    }

    pub async fn insert_to_db(&self)->Result<()>{
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        if self.server == MusicServer::Database && !self.identity.is_empty() {
            return Err(anyhow::anyhow!("Playlist from db, can't insert."));
        }

        let order = playlist::Entity::find().count(&db).await?;
        let playlist = playlist::ActiveModel::new(
            self.name.clone(),
            self.summary.clone(),
            self.cover.clone(),
            order as i64,
        );

        playlist::Entity::insert(playlist)
            .exec(&db)
            .await?;
        Ok(())
    }

    /// 从数据库中删除一个歌单, 同时删除歌单和音乐的关联
    /// 如果是其他平台的歌单， 则无法删除
    /// 注意这将取走歌单的所有权， ffi时应当注意生命周期
    pub async fn del_from_db(self) -> Result<()> {
        if self.server != MusicServer::Database {
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

        for music_agg in music_aggs {
            music_agg.insert_to_db().await?;

            let junction = playlist_music_junction::ActiveModel::new(
                self.identity.parse::<i64>()?,
                music_agg.identity(),
                order,
            );
            println!("{:?}", junction);
            playlist_music_junction::Entity::insert(junction)
                .exec(&db)
                .await?;
            order += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_playlist {
    use sea_orm_migration::MigratorTrait as _;

    use crate::refactor::data::{init_db, migrations::Migrator};

    use super::*;
    async fn re_init_db() {
        // 初始化log
        tracing_subscriber::fmt::init();
        
        let db_file = "./test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::File::create(path).unwrap();

        init_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
        Migrator::up(&get_db().await.unwrap(), None).await.unwrap();
    }

    #[tokio::test]
    async fn test_save_to_db() {
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
        playlists.into_iter().next().unwrap().del_from_db().await.unwrap();
        assert!(Playlist::get_from_db().await.unwrap().len() == 0);
    }

    #[tokio::test]
    async fn test_add_aggs_to_db() {
        re_init_db().await;
        let playlist = Playlist::new(
            "test".to_string(),
            Some("test playlist".to_string()),
            None,
            vec![],
        );
        playlist.insert_to_db().await.unwrap();
        
        let playlist = Playlist::get_from_db().await.unwrap().into_iter().next().unwrap();
        
        let aggs = vec![];
        let aggs =
            MusicAggregator::search(aggs, vec![MusicServer::Kuwo], "Aimer".to_string(), 1, 30)
                .await
                .unwrap();
        
        playlist.add_aggs_to_db(&aggs).await.unwrap();
    }
}
