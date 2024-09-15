pub mod interface;
pub(crate) mod migrations;
pub(crate) mod models;

use migrations::Migrator;
use once_cell::sync::OnceCell;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use tokio::sync::RwLock;

static DB_POOL: OnceCell<Arc<RwLock<Option<DatabaseConnection>>>> = OnceCell::new();

pub async fn set_db(database_url: &str) -> Result<(), anyhow::Error> {
    let db = Database::connect(database_url).await?;
    Migrator::up(&db, None).await?;
    let connection = Arc::new(RwLock::new(Some(db)));
    let _ = DB_POOL.set(connection);
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
