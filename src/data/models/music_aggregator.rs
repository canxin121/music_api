use sea_orm::entity::prelude::*;

use crate::{
    data::interface::{
        music_aggregator::MusicAggregator, server::MusicServer, utils::split_string,
    },
    server::{kuwo, netease},
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "music_aggregator")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub identity: String,
    pub default_server: MusicServer,
    pub kuwo_music_id: Option<String>,
    pub netease_music_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    PlaylistMusicJunction,
    KuwoMusic,
    NeteaseMusicId,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::PlaylistMusicJunction => {
                Entity::has_many(super::playlist_music_junction::Entity)
                    .from(Column::Identity)
                    .to(super::playlist_music_junction::Column::MusicAggregatorId)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .into()
            }
            Relation::KuwoMusic => Entity::has_one(crate::server::kuwo::model::Entity)
                .from(Column::KuwoMusicId)
                .to(crate::server::kuwo::model::Column::MusicId)
                .on_delete(sea_query::ForeignKeyAction::Cascade)
                .into(),
            Relation::NeteaseMusicId => Entity::has_one(crate::server::netease::model::Entity)
                .from(Column::NeteaseMusicId)
                .to(crate::server::netease::model::Column::MusicId)
                .on_delete(sea_query::ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Model {
    pub async fn get_music_aggregator(
        &self,
        db: &DatabaseConnection,
        order: i64,
    ) -> anyhow::Result<MusicAggregator> {
        let (name, artist) = split_string(&self.identity)?;
        let mut musics = Vec::with_capacity(MusicServer::length());

        // todo: add more music server
        if let Some(id) = self.kuwo_music_id.as_ref() {
            let kuwo_music = kuwo::model::Entity::find_by_id(id).one(db).await?;

            if let Some(kuwo_music) = kuwo_music {
                musics.push(kuwo_music.into_music(true));
            }
        }

        if let Some(id) = self.netease_music_id.as_ref() {
            let netease_music = netease::model::Entity::find_by_id(id).one(db).await?;
            if let Some(netease_music) = netease_music {
                musics.push(netease_music.into_music(true));
            }
        }

        let agg = MusicAggregator {
            name,
            artist,
            from_db: true,
            musics,
            default_server: self.default_server.clone(),
            order: Some(order),
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

impl Related<crate::server::kuwo::model::Entity> for Entity {
    fn to() -> RelationDef {
        crate::server::kuwo::model::Relation::MusicAggregator.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
