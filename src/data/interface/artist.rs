use std::ops::Deref;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Artist {
    pub name: String,
    pub id: Option<i64>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct ArtistVec(pub Vec<Artist>);

impl From<Vec<Artist>> for ArtistVec {
    fn from(qualities: Vec<Artist>) -> Self {
        Self(qualities)
    }
}

impl Deref for ArtistVec {
    type Target = Vec<Artist>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
