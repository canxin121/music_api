use sea_orm::entity::prelude::*;

#[derive(Default, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(1))")]
pub enum MusicPlatform {
    #[default]
    #[sea_orm(string_value = "K")]
    Kuwo,
    #[sea_orm(string_value = "N")]
    Netease,
}
