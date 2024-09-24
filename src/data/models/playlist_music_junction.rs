use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "playlist_music_junction"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel, Serialize, Deserialize)]
pub struct Model {
    pub playlist_id: i64,
    pub music_aggregator_id: String,
    pub order: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    PlaylistId,
    MusicAggregatorId,
    Order,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    PlaylistId,
    MusicAggregatorId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = (i64, String);

    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Playlist,
    MusicAggregator,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Playlist => Entity::belongs_to(super::playlist::Entity)
                .from(Column::PlaylistId)
                .to(super::playlist::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .into(),
            Relation::MusicAggregator => Entity::belongs_to(super::music_aggregator::Entity)
                .from(Column::MusicAggregatorId)
                .to(super::music_aggregator::Column::Identity)
                .on_delete(ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Playlist.def()
    }
}

impl Related<super::music_aggregator::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MusicAggregator.def()
    }
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        ColumnType::Integer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new(playlist_id: i64, music_aggregator_id: String, order: i64) -> Self {
        Self {
            playlist_id: Set(playlist_id),
            music_aggregator_id: Set(music_aggregator_id),
            order: Set(order),
        }
    }
}
