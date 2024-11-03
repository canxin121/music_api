use std::{path::PathBuf, str::FromStr};

use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait as _;

use crate::{data::migrations::Migrator, DB_POOL};

pub async fn create_sqlite_db_file(database_url: &str) -> Result<(), anyhow::Error> {
    if database_url == "sqlite::memory:" {
        return Ok(());
    }
    let db_file: PathBuf = PathBuf::from_str(database_url.split("//").last().ok_or(
        anyhow::anyhow!("Invalid database url, use 'sqlite://path/to/database.db'"),
    )?)?;

    if db_file.parent().is_none() {
        tokio::fs::create_dir_all(db_file.parent().unwrap())
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create parent directory when create sqlite db file: {}",
                    e
                )
            })?;
    };

    if !db_file.exists() {
        tokio::fs::File::create(db_file).await?;
    }
    Ok(())
}

pub async fn set_db(database_url: &str) -> Result<(), anyhow::Error> {
    close_db().await?;

    if database_url.starts_with("sqlite") {
        create_sqlite_db_file(database_url).await?;
    }

    let db_connection = Database::connect(database_url).await?;

    Migrator::up(&db_connection, None).await?;

    let mut db_pool = DB_POOL.write().await;

    *db_pool = Some(db_connection);
    Ok(())
}

pub async fn get_db() -> Option<DatabaseConnection> {
    DB_POOL.read().await.clone()
}

pub async fn close_db() -> Result<(), anyhow::Error> {
    let mut db = DB_POOL.write().await;
    if let Some(db_conn) = db.clone() {
        db_conn.close().await?;
    }

    *db = None;

    Ok(())
}

pub async fn clear_db() -> Result<(), anyhow::Error> {
    let db = get_db()
        .await
        .ok_or(anyhow::anyhow!("Database is not inited"))?;
    Migrator::down(&db, None).await?;
    Ok(())
}

pub async fn reinit_db() -> Result<(), anyhow::Error> {
    let db = get_db()
        .await
        .ok_or(anyhow::anyhow!("Database is not inited"))?;
    Migrator::down(&db, None).await?;
    Migrator::up(&db, None).await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::interface::{
        music_aggregator::MusicAggregator, playlist::Playlist,
        playlist_collection::PlaylistCollection, server::MusicServer,
    };

    pub use super::*;

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
        let playlist_collection = PlaylistCollection::new("test".to_string());
        let id = playlist_collection.insert_to_db().await.unwrap();
        let new_playlist_collection = PlaylistCollection::find_in_db(id).await.unwrap();

        let new_playlist = Playlist::new("test".to_string(), None, None, vec![]);
        let id = new_playlist
            .insert_to_db(new_playlist_collection.id)
            .await
            .unwrap();
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
        let id = first_playlist
            .insert_to_db(new_playlist_collection.id)
            .await
            .unwrap();
        let inserted_playlist = Playlist::find_in_db(id).await.unwrap();
        inserted_playlist
            .add_aggs_to_db(&first_playlist.fetch_musics_online(1, 2333).await.unwrap())
            .await
            .unwrap();

        let music_aggs = inserted_playlist.get_musics_from_db().await.unwrap();
        assert!(music_aggs.len() > 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_sqlite() {
        let _ = tracing_subscriber::fmt::try_init();
        set_db("sqlite://./sample_data/test.db").await.unwrap();
        reinit_db().await.unwrap();
        test_op().await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_mysql() {
        let _ = tracing_subscriber::fmt::try_init();
        set_db("mysql://test:testpasswd@localhost:3306/app_rhyme")
            .await
            .unwrap();
        reinit_db().await.unwrap();
        test_op().await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_postgresql() {
        let _ = tracing_subscriber::fmt::try_init();
        set_db("postgresql://test:testpasswd@localhost:5432/app_rhyme")
            .await
            .unwrap();
        reinit_db().await.unwrap();
        test_op().await;
    }
}
