use crate::interface::playlist_subscription::PlayListSubscriptionVec;
use anyhow::Result;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "playlist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub order: i64,
    pub collection_id: i64,
    pub name: String,
    #[sea_orm(nullable)]
    pub summary: Option<String>,
    #[sea_orm(nullable)]
    pub cover: Option<String>,
    pub subscriptions: Option<PlayListSubscriptionVec>,
}

impl ActiveModel {
    pub fn new(
        name: String,
        summary: Option<String>,
        cover: Option<String>,
        order: i64,
        collection_id: i64,
        subscriptions: Option<PlayListSubscriptionVec>,
    ) -> Self {
        Self {
            id: NotSet,
            name: Set(name),
            summary: Set(summary),
            cover: Set(cover),
            subscriptions: Set(subscriptions),
            order: Set(order),
            collection_id: Set(collection_id),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    PlaylistMusicJunction,
    PlaylistCollection,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::PlaylistMusicJunction => {
                Entity::has_many(super::playlist_music_junction::Entity)
                    .from(Column::Id)
                    .to(super::playlist_music_junction::Column::PlaylistId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .into()
            }

            Relation::PlaylistCollection => Entity::belongs_to(super::playlist_collection::Entity)
                .from(Column::CollectionId)
                .to(super::playlist_collection::Column::Id)
                .into(),
        }
    }
}

impl Related<super::music_aggregator::Entity> for Entity {
    // The final relation is Album -> MusicAlbumJunction -> Music
    fn to() -> RelationDef {
        super::playlist_music_junction::Relation::MusicAggregator.def()
    }

    fn via() -> Option<RelationDef> {
        // The original relation is MusicAlbumJunction -> Album,
        // after `rev` it becomes Album -> MusicAlbumJunction
        Some(
            super::playlist_music_junction::Relation::Playlist
                .def()
                .rev(),
        )
    }
}

impl Related<super::playlist_music_junction::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_music_junction::Relation::Playlist.def()
    }
}

impl Related<super::playlist_collection::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_collection::Relation::Playlist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
