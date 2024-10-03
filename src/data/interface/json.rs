use std::{path::PathBuf, str::FromStr};

use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel as _};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{music_aggregator, playlist, playlist_music_junction},
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
    pub async fn apply_to_db(self, playlist_id: Option<i64>) -> anyhow::Result<()> {
        match self {
            MusicDataJson::Database(database_json) => database_json.apply_to_db().await,
            MusicDataJson::Playlists(playlist_json_vec) => playlist_json_vec.insert_to_db().await,
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
        let playlist_music_junctions = playlist_music_junction::Entity::find().all(&db).await?;

        Ok(Self {
            kuwo_table,
            netease_table,
            playlists,
            music_aggregators,
            playlist_music_junctions,
        })
    }

    async fn apply_to_db(self) -> anyhow::Result<()> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not initialized"))?;

        reinit_db().await?;

        playlist::Entity::insert_many(
            self.playlists
                .into_iter()
                .map(|m| m.into_active_model().reset_all())
                .collect::<Vec<playlist::ActiveModel>>(),
        )
        .exec_without_returning(&db)
        .await?;

        music_aggregator::Entity::insert_many(
            self.music_aggregators
                .into_iter()
                .map(|m| {
                    let active = m.into_active_model().reset_all();
                    // println!("{:?}", active);
                    active
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

    /// takes ownership
    async fn insert_to_db(self) -> anyhow::Result<()> {
        for playlistjson in self.0 {
            let id = playlistjson.playlist.insert_to_db().await?;
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
