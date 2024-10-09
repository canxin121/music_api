use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "playlist_collection")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub order: i64,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Playlist,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Playlist => Entity::has_many(super::playlist::Entity)
                .from(Column::Id)
                .to(super::playlist::Column::CollectionId)
                .on_delete(sea_orm::sea_query::ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist::Relation::PlaylistCollection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
