use std::ops::Deref;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Quality {
    pub summary: String,
    pub bitrate: Option<String>,
    pub format: Option<String>,
    pub size: Option<String>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct QualityVec(pub Vec<Quality>);

impl From<Vec<Quality>> for QualityVec {
    fn from(qualities: Vec<Quality>) -> Self {
        Self(qualities)
    }
}

impl Deref for QualityVec {
    type Target = Vec<Quality>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
