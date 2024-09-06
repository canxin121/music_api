use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "albulm_music_junction"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub album_id: i32,
    pub music_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    AlbumId,
    MusicId,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    AlbumId,
    MusicId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = (i32, i32);

    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Album,
    Music,
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        ColumnType::Integer.def()
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Album => Entity::belongs_to(super::local_album::Entity)
                .from(Column::AlbumId)
                .to(super::local_album::Column::Id)
                .into(),
            Self::Music => Entity::belongs_to(super::music::Entity)
                .from(Column::MusicId)
                .to(super::music::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
