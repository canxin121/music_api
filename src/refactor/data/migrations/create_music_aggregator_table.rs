use async_trait::async_trait;
use sea_orm_migration::prelude::*;

use crate::refactor::data::models::music_aggregator::Column;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MusicAggregatorTable::MusicAggragator)
                    .col(
                        ColumnDef::new(Column::Identity)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Column::KuwoMusicId)
                            .string()
                            .null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Column::NeteaseMusicId)
                            .string()
                            .null()
                            .unique_key(),
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
                    .table(MusicAggregatorTable::MusicAggragator)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum MusicAggregatorTable {
    #[sea_orm(iden = "music_aggregator")]
    MusicAggragator,
}
