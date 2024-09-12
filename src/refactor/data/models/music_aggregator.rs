use sea_orm::entity::prelude::*;

use crate::refactor::{
    data::interface::{music_aggregator::MusicAggregator, utils::split_string, MusicServer},
    server::kuwo,
};

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

impl Model {
    pub async fn get_music_aggregator(
        &self,
        db: DatabaseConnection,
    ) -> anyhow::Result<MusicAggregator> {
        let (name, artist) = split_string(&self.identity)?;
        let kuwo_musics = self.find_related(kuwo::model::Entity).one(&db).await?;
        let mut musics = Vec::with_capacity(MusicServer::length());
        if let Some(kuwo_music) = kuwo_musics {
            musics.push(kuwo_music.into_music(true));
        }
        let agg = MusicAggregator {
            name,
            artist,
            from_db: true,
            musics,
        };
        Ok(agg)
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
