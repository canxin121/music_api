use crate::refactor::data::{common::quality::QualityVec, models::music_aggregator};
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
    pub mv_vid: Option<String>,
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
