use sea_orm::{entity::prelude::*, FromJsonQueryResult, Set};
use serde::{Deserialize, Serialize};


#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "playlist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub name: String,
    #[sea_orm(nullable)]
    pub summary: Option<String>,
    #[sea_orm(nullable)]
    pub cover: Option<String>,
    pub subscriptions: Option<PlayListSubscriptionVec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PlayListSubscription {
    #[serde(rename = "type")]
    pub type_field: String,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PlayListSubscriptionVec(pub Vec<PlayListSubscription>);
impl ActiveModel {
    pub fn new(name: String, summary: Option<String>, cover: Option<String>) -> Self {
        Self {
            id: Default::default(),
            name: Set(name),
            summary: Set(summary),
            cover: Set(cover),
            subscriptions: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    PlaylistMusicJunction,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::PlaylistMusicJunction => {
                Entity::has_many(super::playlist_music_junction::Entity)
                    .from(Column::Id)
                    .to(super::playlist_music_junction::Column::PlaylistId)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .on_update(sea_query::ForeignKeyAction::Cascade)
                    .into()
            }
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

impl ActiveModelBehavior for ActiveModel {}
