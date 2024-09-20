use crate::data::{
    interface::{
        artist::ArtistVec, music_aggregator::Music, quality::QualityVec, server::MusicServer,
    },
    models::music_aggregator,
};
use anyhow::Result;
use sea_orm::entity::prelude::*;

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "kuwo_music")]
pub struct Model {
    pub name: String,
    #[sea_orm(primary_key)]
    pub music_id: String,
    pub duration: Option<i64>,
    pub artists: ArtistVec,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub qualities: QualityVec,
    pub cover: Option<String>,
}

impl Model {
    pub fn into_music(self, from_db: bool) -> Music {
        Music {
            from_db,
            server: MusicServer::Kuwo,
            identity: self.music_id,
            duration: self.duration,
            name: self.name,
            album: self.album,
            album_id: self.album_id,
            qualities: self.qualities.0,
            cover: self.cover,
            artists: self.artists.0,
        }
    }
}

impl From<Music> for Model {
    fn from(music: Music) -> Self {
        Self {
            name: music.name,
            music_id: music.identity,
            duration: music.duration,
            artists: music.artists.into(),
            album: music.album,
            album_id: music.album_id,
            qualities: music.qualities.into(),
            cover: music.cover,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    MusicAggregator,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::MusicAggregator => Entity::belongs_to(music_aggregator::Entity)
                .from(Column::MusicId)
                .to(music_aggregator::Column::KuwoMusicId)
                .into(),
        }
    }
}

impl Related<music_aggregator::Entity> for Entity {
    fn to() -> RelationDef {
        music_aggregator::Relation::KuwoMusic.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
