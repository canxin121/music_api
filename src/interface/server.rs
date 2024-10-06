use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum MusicServer {
    #[sea_orm(string_value = "K")]
    Kuwo,
    #[sea_orm(string_value = "N")]
    Netease,
}

impl MusicServer {
    pub fn length() -> usize {
        // todo: add more music server
        2
    }
    pub fn all() -> Vec<MusicServer> {
        vec![MusicServer::Kuwo, MusicServer::Netease]
    }
    pub fn to_string(&self) -> String {
        match self {
            MusicServer::Kuwo => "Kuwo".to_string(),
            MusicServer::Netease => "Netease".to_string(),
        }
    }
}
