use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Quality {
    pub summary: String,
    pub bitrate: Option<String>,
    pub format: Option<String>,
    pub size: Option<String>,
}
