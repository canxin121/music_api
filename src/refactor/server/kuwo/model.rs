use crate::refactor::data::{
    interface::{music_aggregator::Music, quality::QualityVec},
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
    pub duration: Option<i64>,
    pub artist: String,
    pub artist_id: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub qualities: QualityVec,
    pub cover: Option<String>,
    // pub artist_pic: Option<String>,
    // pub album_pic: Option<String>,
    // pub mv_vid: Option<String>,
}

impl Model {
    pub async fn get_lyric(&self) -> Result<String> {
        let lyric = get_kuwo_lyric(&self.music_id).await?;
        Ok(lyric)
    }
}

impl Model {
    pub fn into_music(self, from_db: bool) -> Music {
        Music {
            from_db,
            server: crate::refactor::data::interface::MusicServer::Kuwo,
            indentity: self.music_id,
            duration: self.duration,
            name: self.name,
            artist: self.artist,
            artist_id: self.artist_id,
            album: self.album,
            album_id: self.album_id,
            qualities: self.qualities,
            cover: self.cover,
            // artist_pic: self.artist_pic,
            // album_pic: self.album_pic,
        }
    }
}

impl From<Music> for Model {
    fn from(music: Music) -> Self {
        Self {
            name: music.name,
            music_id: music.indentity,
            duration: music.duration,
            artist: music.artist,
            artist_id: music.artist_id,
            album: music.album,
            album_id: music.album_id,
            qualities: music.qualities,
            cover: music.cover,
            // artist_pic: music.artist_pic,
            // album_pic: music.album_pic,
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
