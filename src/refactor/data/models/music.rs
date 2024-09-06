use std::ops::Deref;

use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use super::music_platform::MusicPlatform;

#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "music")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    // 音乐的独特标识符
    pub identity: String,
    pub name: String,
    pub album_id: i64,
    pub music_platform: MusicPlatform,
    pub artist: ArtistVec,
    #[sea_orm(nullable)]
    pub duration: Option<u32>,
    #[sea_orm(nullable)]
    pub album: Option<String>,
    #[sea_orm(nullable)]
    pub qualities: QualityVec,
    #[sea_orm(nullable)]
    pub default_quality: Option<Quality>,
    #[sea_orm(nullable)]
    pub cover: Option<String>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct ArtistVec(pub Vec<String>);

impl Deref for ArtistVec {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct QualityVec(pub Vec<Quality>);

impl Deref for QualityVec {
    type Target = Vec<Quality>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Quality {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub size: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::local_album::Entity> for Entity {
    fn to() -> RelationDef {
        super::local_album_music_junction::Relation::Album.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::local_album_music_junction::Relation::Music
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
