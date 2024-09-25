use std::{path::PathBuf, str::FromStr};

use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel as _};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{music_aggregator, playlist, playlist_music_junction},
    server::{kuwo, netease},
};

use super::database::{get_db, reinit_db};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseJson {
    pub kuwo_table: Vec<kuwo::model::Model>,
    pub netease_table: Vec<netease::model::Model>,
    pub playlists: Vec<playlist::Model>,
    pub music_aggregators: Vec<music_aggregator::Model>,
    pub playlist_music_junctions: Vec<playlist_music_junction::Model>,
}

impl DatabaseJson {
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

    pub async fn get_from_db() -> anyhow::Result<Self> {
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

    pub async fn apply_to_db(self) -> anyhow::Result<()> {
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

#[cfg(test)]
mod test {

    use crate::data::interface::{
        database::set_db, json::DatabaseJson, music_aggregator::MusicAggregator,
        playlist::Playlist, server::MusicServer,
    };

    async fn test_op() {
        let music_aggs = MusicAggregator::search_online(
            vec![],
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "米津玄师".to_string(),
            1,
            30,
        )
        .await
        .unwrap();
        let new_playlist = Playlist::new("test".to_string(), None, None, vec![]);
        let id = new_playlist.insert_to_db().await.unwrap();
        let new_playlist = Playlist::find_in_db(id).await.unwrap();
        new_playlist.add_aggs_to_db(&music_aggs).await.unwrap();
        new_playlist.add_aggs_to_db(&music_aggs).await.unwrap();

        let music_aggs = new_playlist.get_musics_from_db().await.unwrap();
        assert!(music_aggs.len() > 0);

        let playlist = Playlist::search_online(
            vec![MusicServer::Kuwo, MusicServer::Netease],
            "米津玄师".to_string(),
            1,
            30,
        )
        .await
        .unwrap();

        let first_playlist = playlist.first().unwrap();
        let id = first_playlist.insert_to_db().await.unwrap();
        let inserted_playlist = Playlist::find_in_db(id).await.unwrap();

        inserted_playlist
            .add_aggs_to_db(&first_playlist.fetch_musics_online(1, 2333).await.unwrap())
            .await
            .unwrap();

        let music_aggs = inserted_playlist.get_musics_from_db().await.unwrap();
        assert!(music_aggs.len() > 0);
    }

    #[tokio::test]
    async fn test_apply() {
        tracing_subscriber::fmt::init();
        set_db("sqlite::memory:").await.unwrap();
        test_op().await;
        let database_json = DatabaseJson::get_from_db().await.unwrap();
        database_json
            .save_to("/mnt/disk/git/music_api/sample_data/database_json.json")
            .await
            .unwrap();
        set_db("mysql://test:testpasswd@localhost:3306/app_rhyme")
            .await
            .unwrap();

        database_json.apply_to_db().await.unwrap();

        let playlists = Playlist::get_from_db().await.unwrap();
        assert!(playlists.len() > 0);
    }
}
