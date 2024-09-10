use sea_orm::entity::prelude::*;

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "music_aggregator")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub kuwo_music_id: String,
    pub netease_music_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    PlaylistMusicJunction,
    // KuwoMusic,
    // NeteaseMusicId,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::PlaylistMusicJunction => {
                Entity::has_many(super::playlist_music_junction::Entity)
                    .from(Column::Id)
                    .to(super::playlist_music_junction::Column::MusicAggregatorId)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .on_update(sea_query::ForeignKeyAction::Cascade)
                    .into()
            } // Relation::KuwoMusic => Entity::has_one(crate::refactor::adapter::kuwo::model::Entity)
              //     .from(Column::KuwoMusicId)
              //     .to(crate::refactor::adapter::kuwo::model::Column::Musicrid)
              //     .on_delete(sea_query::ForeignKeyAction::Cascade)
              //     .on_update(sea_query::ForeignKeyAction::Cascade)
              //     .into(),
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

// impl Related<crate::refactor::adapter::kuwo::model::Entity> for Entity {
//     fn to() -> RelationDef {
//         crate::refactor::adapter::kuwo::model::Relation::MusicAggregator.def()
//     }
// }

impl ActiveModelBehavior for ActiveModel {}
