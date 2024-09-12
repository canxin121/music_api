pub mod interface;
pub mod migrations;
pub mod models;

use once_cell::sync::OnceCell;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use tokio::sync::RwLock;

static DB_POOL: OnceCell<Arc<RwLock<Option<DatabaseConnection>>>> = OnceCell::new();

pub async fn init_db(database_url: &str) -> Result<(), anyhow::Error> {
    let db = Database::connect(database_url).await?;
    let connection = Arc::new(RwLock::new(Some(db)));

    DB_POOL
        .set(connection)
        .map_err(|_| sea_orm::DbErr::Custom("Failed to set global connection".into()))?;

    Ok(())
}

pub async fn get_db() -> Option<DatabaseConnection> {
    let db = DB_POOL.get().cloned();
    if let Some(db) = &db {
        if let Some(conn) = db.read().await.clone() {
            return Some(conn);
        }
    }
    None
}

pub async fn close_db() {
    if let Some(db) = DB_POOL.get() {
        if let Some(conn) = db.read().await.clone() {
            let _ = conn.close().await;
        }
        *db.write().await = None;
    }
}

pub async fn db_inited() -> bool {
    if let Some(db) = DB_POOL.get() {
        db.read().await.clone().is_some()
    } else {
        false
    }
}
