use anyhow::anyhow;
use sea_orm::{
    prelude::Expr,
    sea_query::{Alias, Func, Query},
    ActiveValue::NotSet,
    ConnectionTrait, EntityTrait, ModelTrait, QueryFilter, Related, Set, Unchanged,
};
use serde::{Deserialize, Serialize};

use crate::data::models::{playlist, playlist_collection};

use super::{database::get_db, playlist::Playlist};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistCollection {
    pub id: i64,
    pub order: i64,
    pub name: String,
}

impl From<playlist_collection::Model> for PlaylistCollection {
    fn from(value: playlist_collection::Model) -> Self {
        Self {
            id: value.id,
            order: value.order,
            name: value.name,
        }
    }
}

impl PlaylistCollection {
    pub fn new(name: String) -> Self {
        Self {
            id: -1,
            order: -1,
            name,
        }
    }
    pub async fn get_playlists_from_db(&self) -> anyhow::Result<Vec<Playlist>> {
        let db = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        let models = playlist::Entity::find()
            .filter(Expr::col(playlist::Column::CollectionId).eq(self.id))
            .all(&db)
            .await?;

        let playlists = models
            .into_iter()
            .map(|p| p.into())
            .collect::<Vec<Playlist>>();

        Ok(playlists)
    }
    pub async fn insert_to_db(&self) -> anyhow::Result<i64> {
        let db: sea_orm::DatabaseConnection = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        let statement = Query::select()
            .expr(Func::max(Expr::col((
                Alias::new("playlist_collection"),
                playlist_collection::Column::Id,
            ))))
            .from(playlist_collection::Entity)
            .to_owned();
        let query_result = db
            .query_one(sea_orm::StatementBuilder::build(
                &statement,
                &db.get_database_backend(),
            ))
            .await?
            .ok_or(anyhow::anyhow!("Failed to get max id from playlist table."))?;
        let max_id: i64 = query_result.try_get_by_index(0).ok().unwrap_or(0);
        let playlist = playlist_collection::ActiveModel {
            id: NotSet,
            order: Set(max_id + 1),
            name: Set(self.name.clone()),
        };

        let result = playlist_collection::Entity::insert(playlist)
            .exec(&db)
            .await?;
        let last_id = result.last_insert_id;
        Ok(last_id)
    }

    pub async fn find_in_db(id: i64) -> anyhow::Result<Self> {
        let db: sea_orm::DatabaseConnection = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        let result = playlist_collection::Entity::find_by_id(id)
            .one(&db)
            .await?
            .ok_or(anyhow!("PlaylistCollection not found."))?;

        Ok(result.into())
    }

    pub async fn update_to_db(&self) -> anyhow::Result<Self> {
        if self.id == -1 || self.order == -1 {
            return Err(anyhow!("PlaylistCollection id or order is not set."));
        }
        let active = playlist_collection::ActiveModel {
            id: Unchanged(self.id),
            order: Set(self.order),
            name: Set(self.name.clone()),
        };
        let db: sea_orm::DatabaseConnection = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;
        let result = playlist_collection::Entity::update(active)
            .exec(&db)
            .await?;
        Ok(result.into())
    }

    pub async fn delete_from_db(&self) -> anyhow::Result<()> {
        if self.id == -1 {
            return Err(anyhow!("PlaylistCollection id is not set."));
        }
        let db: sea_orm::DatabaseConnection = get_db()
            .await
            .ok_or(anyhow::anyhow!("Database is not inited."))?;

        let result = playlist_collection::Entity::delete_by_id(self.id)
            .exec(&db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::interface::{database::set_db, playlist::Playlist};

    #[tokio::test]
    async fn test() {
        set_db("sqlite::memory:").await.unwrap();

        let playlist_collection = super::PlaylistCollection::new("test".to_string());
        println!("{:?}", playlist_collection);

        let id = playlist_collection.insert_to_db().await.unwrap();
        let new_playlist_collection = super::PlaylistCollection::find_in_db(id).await.unwrap();
        println!("{:?}", new_playlist_collection);

        let playlist1 = Playlist::new("test".to_string(), None, None, vec![]);
        let playlist2 = Playlist::new("test".to_string(), None, None, vec![]);

        let id = playlist1
            .insert_to_db(new_playlist_collection.id)
            .await
            .unwrap();
        let id = playlist2
            .insert_to_db(new_playlist_collection.id)
            .await
            .unwrap();

        let playlist1 = Playlist::find_in_db(id).await.unwrap();
        let playlist2 = Playlist::find_in_db(id).await.unwrap();

        let playlists = new_playlist_collection
            .get_playlists_from_db()
            .await
            .unwrap();
        assert!(playlists.len() == 2);

        playlist1.del_from_db().await.unwrap();

        new_playlist_collection.delete_from_db().await.unwrap();

        let playlists = Playlist::get_from_db().await.unwrap();
        assert_eq!(playlists.len(), 0);
    }
}
