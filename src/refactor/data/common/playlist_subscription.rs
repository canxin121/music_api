use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PlayListSubscription {
    #[serde(rename = "type")]
    pub type_field: String,
    pub identity: String,
}
