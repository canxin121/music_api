use std::{path::PathBuf, str::FromStr};

use anyhow::anyhow;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, EntityTrait, IntoActiveModel as _};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{music_aggregator, playlist, playlist_collection, playlist_music_junction},
    server::{kuwo, netease},
};

use super::{
    database::{get_db, reinit_db},
    music_aggregator::MusicAggregator,
    playlist::Playlist,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseJson {
    pub kuwo_table: Vec<kuwo::model::Model>,
    pub netease_table: Vec<netease::model::Model>,
    pub playlists: Vec<playlist::Model>,
    pub playlist_collection: Vec<playlist_collection::Model>,
    pub music_aggregators: Vec<music_aggregator::Model>,
    pub playlist_music_junctions: Vec<playlist_music_junction::Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistJson {
    pub playlist: Playlist,
    pub music_aggregators: Vec<MusicAggregator>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistJsonVec(pub Vec<PlaylistJson>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MusicAggregatorJsonVec(pub Vec<MusicAggregator>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MusicDataJson {
    Database(DatabaseJson),
    Playlists(PlaylistJsonVec),
    MusicAggregators(MusicAggregatorJsonVec),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MusicDataType {
    Database,
    Playlists,
    MusicAggregators,
}

impl MusicDataJson {
    pub fn get_type(&self) -> MusicDataType {
        match self {
            MusicDataJson::Database(_) => MusicDataType::Database,
            MusicDataJson::Playlists(_) => MusicDataType::Playlists,
            MusicDataJson::MusicAggregators(_) => MusicDataType::MusicAggregators,
        }
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub async fn save_to(&self, path: &str) -> anyhow::Result<()> {
        let path = PathBuf::from_str(path)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_string(self)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load_from(path: &str) -> anyhow::Result<Self> {
        let json = tokio::fs::read_to_string(path).await?;
        let db = serde_json::from_str(&json)?;
        Ok(db)
    }

    /// takes ownership
    pub async fn apply_to_db(
        self,
        playlist_id: Option<i64>,
        playlist_collection_id: Option<i64>,
    ) -> anyhow::Result<()> {
        match self {
            MusicDataJson::Database(database_json) => database_json.apply_to_db().await,
            MusicDataJson::Playlists(playlist_json_vec) => {
                playlist_json_vec
                    .insert_to_db(
                        playlist_collection_id
                            .ok_or(anyhow!("No Playlist Collection id provided"))?,
                    )
                    .await
            }
            MusicDataJson::MusicAggregators(music_aggregator_json_vec) => {
                Playlist::find_in_db(playlist_id.ok_or(anyhow::anyhow!("No Playlist id provided"))?)
                    .await
                    .ok_or(anyhow::anyhow!(
                        "Failed to find playlist with id: {:?}",
                        playlist_id
                    ))?
                    .add_aggs_to_db(&music_aggregator_json_vec.0)
                    .await
            }
        }
    }

    pub async fn from_database() -> anyhow::Result<Self> {
        Ok(MusicDataJson::Database(DatabaseJson::get_from_db().await?))
    }

    pub async fn from_playlists(playlists: Vec<Playlist>) -> anyhow::Result<Self> {
        Ok(MusicDataJson::Playlists(
            PlaylistJsonVec::from_playlists(playlists).await?,
        ))
    }

    pub async fn from_music_aggregators(
        music_aggregators: Vec<MusicAggregator>,
    ) -> anyhow::Result<Self> {
        Ok(MusicDataJson::MusicAggregators(MusicAggregatorJsonVec(
            music_aggregators,
        )))
    }
}

impl DatabaseJson {
    async fn get_from_db() -> anyhow::Result<Self> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not initialized"))?;

        let kuwo_table = kuwo::model::Entity::find().all(&db).await?;
        let netease_table = netease::model::Entity::find().all(&db).await?;
        let playlists = playlist::Entity::find().all(&db).await?;
        let music_aggregators = music_aggregator::Entity::find().all(&db).await?;
        let playlist_collection = playlist_collection::Entity::find().all(&db).await?;
        let playlist_music_junctions = playlist_music_junction::Entity::find().all(&db).await?;

        Ok(Self {
            kuwo_table,
            netease_table,
            playlists,
            music_aggregators,
            playlist_music_junctions,
            playlist_collection,
        })
    }

    async fn apply_to_db(self) -> anyhow::Result<()> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not initialized"))?;

        reinit_db().await?;

        if self.playlists.is_empty() {
            return Ok(());
        }

        playlist_collection::Entity::insert_many(
            self.playlist_collection
                .into_iter()
                .map(|m| {
                    let mut m = m.into_active_model().reset_all();
                    m.id = NotSet;
                    m
                })
                .collect::<Vec<playlist_collection::ActiveModel>>(),
        )
        .exec_without_returning(&db)
        .await?;

        playlist::Entity::insert_many(
            self.playlists
                .into_iter()
                .map(|m| {
                    let mut m = m.into_active_model().reset_all();
                    m.id = NotSet;
                    m
                })
                .collect::<Vec<playlist::ActiveModel>>(),
        )
        .exec_without_returning(&db)
        .await?;

        if self.music_aggregators.is_empty() {
            return Ok(());
        }

        music_aggregator::Entity::insert_many(
            self.music_aggregators
                .into_iter()
                .map(|m| {
                    let m = m.into_active_model().reset_all();
                    m
                })
                .collect::<Vec<music_aggregator::ActiveModel>>(),
        )
        .exec_without_returning(&db)
        .await?;

        kuwo::model::Entity::insert_many(
            self.kuwo_table
                .into_iter()
                .map(|m| {
                    let active = m.into_active_model().reset_all();
                    active
                })
                .collect::<Vec<kuwo::model::ActiveModel>>(),
        )
        .exec_without_returning(&db)
        .await?;

        netease::model::Entity::insert_many(
            self.netease_table
                .into_iter()
                .map(|m| {
                    let active = m.into_active_model().reset_all();
                    active
                })
                .collect::<Vec<netease::model::ActiveModel>>(),
        )
        .exec_without_returning(&db)
        .await?;

        playlist_music_junction::Entity::insert_many(
            self.playlist_music_junctions
                .into_iter()
                .map(|m| m.into_active_model().reset_all())
                .collect::<Vec<playlist_music_junction::ActiveModel>>(),
        )
        .exec_without_returning(&db)
        .await?;

        Ok(())
    }
}

impl PlaylistJsonVec {
    async fn from_playlists(playlists: Vec<Playlist>) -> anyhow::Result<Self> {
        let len = playlists.len();
        let mut handle = Vec::with_capacity(len);

        for playlist in playlists {
            handle.push(tokio::spawn(async move {
                match playlist.from_db {
                    true => (playlist.get_musics_from_db().await, playlist),
                    false => (playlist.fetch_musics_online(1, 2333).await, playlist),
                }
            }));
        }
        let mut result = Self(Vec::with_capacity(len));
        for handle in handle {
            let (musics, playlist) = handle.await?;
            result.0.push(PlaylistJson {
                playlist,
                music_aggregators: musics?,
            });
        }
        Ok(result)
    }

    async fn insert_to_db(self, playlist_collection_id: i64) -> anyhow::Result<()> {
        for playlistjson in self.0 {
            let id = playlistjson
                .playlist
                .insert_to_db(playlist_collection_id)
                .await?;
            let inserted_playlist = Playlist::find_in_db(id).await.ok_or(anyhow::anyhow!(
                "Failed to find playlist with id {} after insertion",
                id
            ))?;
            inserted_playlist
                .add_aggs_to_db(&playlistjson.music_aggregators)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sea_orm_migration::MigratorTrait as _;

    use crate::{
        data::migrations::Migrator,
        interface::{
            database::{get_db, set_db},
            music_aggregator::MusicAggregator,
            playlist::Playlist,
            playlist_collection::PlaylistCollection,
            server::MusicServer,
        },
    };

    async fn re_init_db() {
        let _ = tracing_subscriber::fmt::try_init();
        let db_file = "./sample_data/test.db";
        let path = std::path::Path::new(db_file);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }

        std::fs::File::create(path).unwrap();

        set_db(&("sqlite://".to_owned() + db_file)).await.unwrap();
        Migrator::up(&get_db().await.unwrap(), None).await.unwrap();
        let playlist_collection1 = PlaylistCollection {
            id: -1,
            order: -1,
            name: "1".to_string(),
        };
        let inserted_playlist_collection1 = playlist_collection1.insert_to_db().await.unwrap();
        let playlist_collection2 = PlaylistCollection {
            id: -1,
            order: -1,
            name: "2".to_string(),
        };
        let inserted_playlist_collection2 = playlist_collection2.insert_to_db().await.unwrap();

        let playlist1 = Playlist::new("1".to_string(), None, None, Vec::new());
        let inserted_playlist1_id = playlist1
            .insert_to_db(inserted_playlist_collection1)
            .await
            .unwrap();
        let playlist2 = Playlist::new("2".to_string(), None, None, Vec::new());
        let inserted_playlist2_id = playlist2
            .insert_to_db(inserted_playlist_collection2)
            .await
            .unwrap();

        let musics = MusicAggregator::search_online(
            Vec::new(),
            MusicServer::all(),
            "张国荣".to_string(),
            1,
            100,
        )
        .await
        .unwrap();
        let playlist1 = Playlist::find_in_db(inserted_playlist1_id).await.unwrap();
        playlist1.add_aggs_to_db(&musics).await.unwrap();

        let musics = MusicAggregator::search_online(
            Vec::new(),
            MusicServer::all(),
            "周杰伦".to_string(),
            1,
            100,
        )
        .await
        .unwrap();
        let playlist2 = Playlist::find_in_db(inserted_playlist2_id).await.unwrap();
        playlist2.add_aggs_to_db(&musics).await.unwrap();
    }

    #[tokio::test]
    async fn test_database_sqlite() {
        re_init_db().await;
        let db = MusicDataJson::from_database().await.unwrap();
        let json = db.to_json().unwrap();
        let db = MusicDataJson::from_json(&json).unwrap();
        db.clone().apply_to_db(None, None).await.unwrap();

        set_db("mysql://test:testpasswd@localhost:3306/app_rhyme")
            .await
            .unwrap();
        db.clone().apply_to_db(None, None).await.unwrap();

        set_db("postgresql://test:testpasswd@localhost:5432/app_rhyme")
            .await
            .unwrap();
        db.clone().apply_to_db(None, None).await.unwrap();
    }
}
