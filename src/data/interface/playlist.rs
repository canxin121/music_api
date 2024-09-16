use sea_orm::{
    ColumnTrait as _, Condition, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, Set,
};
use sea_query::Expr;
use serde::{Deserialize, Serialize};

use crate::data::{
    get_db,
    models::{music_aggregator, playlist, playlist_music_junction},
};
use anyhow::Result;

use super::{
    music_aggregator::MusicAggregator,
    playlist_subscription::{PlayListSubscription, PlayListSubscriptionVec},
    server::MusicServer,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaylistType {
    UserPlaylist,
    Album,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        Ok(models.into_iter().map(|m| m.into()).collect())
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
        let mut junctions = Vec::with_capacity(music_aggs.len());

        for music_agg in music_aggs {
            music_agg.save_to_db().await?;
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
        Ok(aggs)
    }
}

#[cfg(test)]
mod test_playlist {
    use sea_orm_migration::MigratorTrait as _;
    use serial_test::serial;

    use crate::data::{migrations::Migrator, set_db};

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
}
