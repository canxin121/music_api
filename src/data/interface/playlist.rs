use sea_orm::{
    prelude::Expr, ColumnTrait as _, Condition, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use crate::data::{
    interface::utils::find_duplicate_music_agg,
    models::{music_aggregator, playlist, playlist_music_junction},
};
use anyhow::Result;

use super::{
    database::get_db,
    music_aggregator::MusicAggregator,
    playlist_subscription::{PlayListSubscription, PlayListSubscriptionVec},
    results::PlaylistUpdateSubscriptionResult,
    server::MusicServer,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaylistType {
    UserPlaylist,
    Album,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playlist {
    pub from_db: bool,
    pub server: Option<MusicServer>,
    #[serde(rename = "type")]
    pub type_field: PlaylistType,
    pub identity: String,
    pub order: Option<i64>,
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
            order: Some(value.order),
        }
    }
}

impl Playlist {
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
            order: None,
        }
    }

    /// find db playlist by primary key `id`
    pub async fn find_in_db(id: i64) -> Option<Self> {
        let db = get_db().await.expect("Database is not inited.");
        let model: Option<playlist::Model> = playlist::Entity::find_by_id(id)
            .one(&db)
            .await
            .expect("Failed to find playlist by id.");
        model.and_then(|m| Some(m.into()))
    }

    /// update db playlist info
    pub async fn update_to_db(&self) -> Result<Self> {
        if !self.from_db || self.identity.is_empty() {
            return Err(anyhow::anyhow!(
                "Can't update playlist from non-database server."
            ));
        }
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        if let Ok(id) = self.identity.parse::<i64>() {
            let playlist = playlist::ActiveModel {
                id: Set(id),
                order: Set(self.order.unwrap_or(i64::MAX)),
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

    // insert a playlist to db
    pub async fn insert_to_db(&self) -> Result<i64> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        let order = playlist::Entity::find().count(&db).await?;
        let playlist = playlist::ActiveModel::new(
            self.name.clone(),
            self.summary.clone(),
            self.cover.clone(),
            order as i64,
            self.subscription
                .clone()
                .and_then(|s| Some(PlayListSubscriptionVec(s))),
        );
        let result = playlist::Entity::insert(playlist).exec(&db).await?;
        let last_id = result.last_insert_id;
        Ok(last_id)
    }

    /// delete a playlist from db
    /// this will also delete all junctions between the playlist and music
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

    /// get playlists from db
    pub async fn get_from_db() -> Result<Vec<Self>> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        let models = playlist::Entity::find().all(&db).await?;
        let mut playlists = models.into_iter().map(|m| m.into()).collect::<Vec<Self>>();
        playlists.sort_by(|a, b| {
            a.order
                .unwrap_or(i64::MAX)
                .cmp(&b.order.unwrap_or(i64::MAX))
        });

        Ok(playlists)
    }

    /// add playlist music aggregator junction to db
    /// this will also add the music and music aggregators to the db
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
        for music_agg in music_aggs {
            match music_agg.save_to_db().await {
                Ok(duplicate) => {
                    let junction = playlist_music_junction::ActiveModel::new(
                        self.identity.parse::<i64>()?,
                        duplicate.unwrap_or(music_agg.identity()),
                        order,
                    );
                    order += 1;

                    match playlist_music_junction::Entity::insert(junction)
                        .exec_without_returning(&db)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            if let sea_orm::DbErr::Exec(sea_orm::RuntimeErr::SqlxError(
                                sea_orm::SqlxError::Database(database_error),
                            )) = &e
                            {
                                let database_error_str = database_error.to_string();
                                // Sqlite Unique error
                                if database_error_str.contains("UNIQUE")
                                    // MySql Unique error
                                    ||database_error_str.contains("1062")
                                    // PgSql Unique error
                                    || database_error_str.contains("duplicate")
                                {
                                    continue;
                                } else if database_error_str.contains("1452")
                                    || database_error_str.contains("violates foreign key")
                                {
                                    // 因为某些平台的 不同名称的歌曲公用一个id, 所以可能会出现重复
                                    // 因此导致名称不同，但是内容相同的MusicAggregator插入失败
                                    // 此时应该根据id查找到已有的MusicAggregator，然后插入junction
                                    if let Some(found_music_agg_id) =
                                        find_duplicate_music_agg(&db, music_agg).await
                                    {
                                        let junction = playlist_music_junction::ActiveModel::new(
                                            self.identity.parse::<i64>()?,
                                            found_music_agg_id,
                                            order,
                                        );
                                        order += 1;
                                        if let Err(e) =
                                            playlist_music_junction::Entity::insert(junction)
                                                .exec_without_returning(&db)
                                                .await
                                        {
                                            log::error!(
                                                "Failed to try fix save playlist music junction for music agg: [{}]({}) and playlist: [{}]({}), error: ({})",
                                                music_agg.identity(),
                                                music_agg.name,
                                                self.identity,
                                                self.name,
                                                e.to_string()
                                            );
                                        } else {
                                            continue;
                                        }
                                    } else {
                                        log::error!(
                                            "Failed to try fix save playlist music junction for music agg: [{}]({}) and playlist: [{}]({}), error: Can't find the depulicate music agg.",
                                            music_agg.identity(),
                                            music_agg.name,
                                            self.identity,
                                            self.name,
                                        );
                                        continue;
                                    }
                                }
                            }
                            log::error!(
                                "Failed to save playlist music junction for music agg: [{}]({}) and playlist: [{}]({}), error: ({})",
                                music_agg.identity(),
                                music_agg.name,
                                self.identity,
                                self.name,
                                e.to_string()
                            );
                        }
                    };
                }
                Err(e) => log::error!(
                    "Failed to save music agg: [{}]({}), error: {}",
                    music_agg.identity(),
                    music_agg.name,
                    e.to_string()
                ),
            }
        }

        Ok(())
    }

    /// get all music aggregators from db
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
        let junctions = playlist_music_junction::Entity::find()
            .filter(
                Condition::all().add(Expr::col(playlist_music_junction::Column::PlaylistId).eq(id)),
            )
            .all(&db)
            .await?;

        let mut aggs = Vec::with_capacity(junctions.len());
        for junction in junctions {
            let agg = junction
                .find_related(music_aggregator::Entity)
                .one(&db)
                .await?
                .ok_or(anyhow::anyhow!("Can't find music aggregator in db"))?;
            aggs.push(agg.get_music_aggregator(&db, junction.order).await?);
        }
        aggs.sort_by(|a, b| a.order.cmp(&b.order));
        Ok(aggs)
    }

    pub async fn update_subscription(&self) -> Result<PlaylistUpdateSubscriptionResult> {
        if !self.from_db {
            return Err(anyhow::anyhow!(
                "Can't update subscription for non-database playlist"
            ));
        }

        if self.subscription.is_none() || self.subscription.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!(
                "The playlist has no subscription to update"
            ));
        }

        let subscriptions = self.subscription.clone().unwrap();
        let mut handles: Vec<tokio::task::JoinHandle<Result<Vec<MusicAggregator>>>> =
            Vec::with_capacity(subscriptions.len());

        for subscription in subscriptions {
            handles.push(tokio::spawn(async move {
                let playlist = Playlist::get_from_share(&subscription.share).await?;
                let musics = playlist.fetch_musics_online(1, 2333).await?;
                Ok(musics)
            }));
        }
        let mut result = PlaylistUpdateSubscriptionResult {
            errors: Vec::with_capacity((handles.len() / 2).max(1)),
        };

        for (handle, subscription) in handles.into_iter().zip(self.subscription.as_ref().unwrap()) {
            match handle.await {
                Ok(fetch_result) => match fetch_result {
                    Ok(aggs) => match self.add_aggs_to_db(&aggs).await {
                        Ok(_) => {}
                        Err(e) => {
                            result
                                .errors
                                .push((subscription.name.to_string(), e.to_string()));
                        }
                    },
                    Err(e) => {
                        result
                            .errors
                            .push((subscription.name.to_string(), e.to_string()));
                    }
                },
                Err(e) => {
                    result
                        .errors
                        .push((subscription.name.to_string(), e.to_string()));
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test_playlist {
    use sea_orm_migration::MigratorTrait as _;
    use serial_test::serial;

    use crate::data::{interface::database::set_db, migrations::Migrator};

    use super::*;
    async fn re_init_db() {
        // 初始化log
        let _ = tracing_subscriber::fmt::try_init();

        let db_file = "./sample_data/test.db";
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
        let aggs = MusicAggregator::search_online(
            aggs,
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "Aimer".to_string(),
            1,
            30,
        )
        .await
        .unwrap();

        playlist.add_aggs_to_db(&aggs).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_add_aggs_to_db2() {
        re_init_db().await;
        let playlists = Playlist::search_online(
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "Aimer".to_string(),
            1,
            30,
        );
        let playlist = playlists.await.unwrap().into_iter().next().unwrap();
        let aggs = playlist.fetch_musics_online(1, 30).await.unwrap();

        let insert_id = playlist.insert_to_db().await.unwrap();
        let playlist = Playlist::find_in_db(insert_id).await.unwrap();
        // 测量此处耗时
        let start = std::time::Instant::now();
        playlist.add_aggs_to_db(&aggs).await.unwrap();
        println!("Add aggs to db cost: {:?}", start.elapsed());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_musics_from_db() {
        re_init_db().await;
        let playlists = Playlist::search_online(
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "Aimer".to_string(),
            1,
            30,
        );
        let playlist = playlists.await.unwrap().into_iter().next().unwrap();
        let aggs = playlist.fetch_musics_online(1, 30).await.unwrap();

        let insert_id = playlist.insert_to_db().await.unwrap();
        let playlist = Playlist::find_in_db(insert_id).await.unwrap();
        playlist.add_aggs_to_db(&aggs).await.unwrap();

        let music_aggs = playlist.get_musics_from_db().await.unwrap();
        println!("{:?}", music_aggs);
        println!("Length: {}", music_aggs.len());
    }

    #[tokio::test]
    #[serial]
    async fn test_update_subscription() {
        re_init_db().await;
        let playlist = Playlist::new(
            "test".to_string(),
            None,
            None,
            vec![PlayListSubscription {
                name: "1".to_string(),
                share: "分享Z殘心的歌单《米津玄师》https://y.music.163.com/m/playlist?app_version=8.9.20&id=6614178314&userid=317416193&dlt=0846&creatorId=317416193 (@网易云音乐)".to_string(),
            }],
        );
        let playlist_id = playlist.insert_to_db().await.unwrap();
        let playlist = Playlist::find_in_db(playlist_id).await.unwrap();
        let result = playlist.update_subscription().await.unwrap();
        println!("{:?}", result);
        let reuslt = playlist.update_subscription().await.unwrap();
        println!("{:?}", reuslt);
    }
}
