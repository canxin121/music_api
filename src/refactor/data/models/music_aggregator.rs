use sea_orm::entity::prelude::*;

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "music_aggregator")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub identity: String,
    pub kuwo_music_id: Option<String>,
    pub netease_music_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    PlaylistMusicJunction,
    KuwoMusic,
    // NeteaseMusicId,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::PlaylistMusicJunction => {
                Entity::has_many(super::playlist_music_junction::Entity)
                    .from(Column::Identity)
                    .to(super::playlist_music_junction::Column::MusicAggregatorId)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .on_update(sea_query::ForeignKeyAction::Cascade)
                    .into()
            }
            Relation::KuwoMusic => Entity::has_one(crate::refactor::server::kuwo::model::Entity)
                .from(Column::KuwoMusicId)
                .to(crate::refactor::server::kuwo::model::Column::MusicId)
                .on_delete(sea_query::ForeignKeyAction::Cascade)
                .on_update(sea_query::ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_music_junction::Relation::Playlist.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::playlist_music_junction::Relation::MusicAggregator
                .def()
                .rev(),
        )
    }
}

impl Related<super::playlist_music_junction::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_music_junction::Relation::MusicAggregator.def()
    }
}

impl Related<crate::refactor::server::kuwo::model::Entity> for Entity {
    fn to() -> RelationDef {
        crate::refactor::server::kuwo::model::Relation::MusicAggregator.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}