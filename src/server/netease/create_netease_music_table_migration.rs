use async_trait::async_trait;
use sea_orm_migration::{
    prelude::*,
    schema::{integer_null, string, string_null},
};

use crate::data::{migrations::create_music_aggregator_table, models::music_aggregator};

use super::model::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NeteaseMusicTable::NeteaseMusic)
                    .col(
                        ColumnDef::new(Column::MusicId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(Column::Name))
                    .col(string(Column::Artists))
                    .col(string_null(Column::Album))
                    .col(string_null(Column::AlbumId))
                    .col(string(Column::Qualities))
                    .col(string(Column::Cover))
                    .col(integer_null(Column::Duration))
                    .foreign_key(
                        ForeignKey::create()
                            .from(NeteaseMusicTable::NeteaseMusic, Column::MusicId)
                            .to(
                                create_music_aggregator_table::MusicAggregatorTable::MusicAggragator,
                                music_aggregator::Column::NeteaseMusicId,
                            )
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                    )
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(NeteaseMusicTable::NeteaseMusic)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum NeteaseMusicTable {
    #[sea_orm(iden = "netease_music")]
    NeteaseMusic,
}