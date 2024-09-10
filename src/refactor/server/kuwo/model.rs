use crate::refactor::data::{
    common::{music::Music, quality::QualityVec},
    models::music_aggregator,
};
use anyhow::Result;
use sea_orm::entity::prelude::*;

use super::web_api::get_kuwo_lyric;

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "kuwo_music")]
pub struct Model {
    pub name: String,
    #[sea_orm(primary_key)]
    pub music_id: String,
    pub artist: String,
    pub artist_id: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub qualities: QualityVec,
    pub music_pic: String,
    pub artist_pic: Option<String>,
    pub album_pic: Option<String>,
    // pub mv_vid: Option<String>,
}

impl Into<Music> for Model {
    fn into(self) -> Music {
        Music {
            server: crate::refactor::data::common::MusicServer::Kuwo,
            indentity: self.music_id,
            name: self.name,
            artist: self.artist,
            artist_id: self.artist_id,
            album: self.album,
            album_id: self.album_id,
            qualities: self.qualities,
            music_pic: self.music_pic,
            artist_pic: self.artist_pic,
            album_pic: self.album_pic,
        }
    }
}

impl From<Music> for Model {
    fn from(value: Music) -> Self {
        Model {
            name: value.name,
            music_id: value.indentity,
            artist: value.artist,
            artist_id: value.artist_id,
            album: value.album,
            album_id: value.album_id,
            qualities: value.qualities,
            music_pic: value.music_pic,
            artist_pic: value.artist_pic,
            album_pic: value.album_pic,
            // mv_vid: None,
        }
    }
}

impl Model {
    pub async fn get_lyric(&self) -> Result<String> {
        let lyric = get_kuwo_lyric(&self.music_id).await?;
        Ok(lyric)
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
