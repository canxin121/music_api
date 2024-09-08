use async_trait::async_trait;
use sea_orm_migration::{
    prelude::*,
    schema::{pk_auto, string, string_null},
};

use crate::refactor::data::models::playlist::Column;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlaylistTable::Playlist)
                    .col(pk_auto(Column::Id))
                    .col(string(Column::Name))
                    .col(string_null(Column::Summary))
                    .col(string_null(Column::Cover))
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlaylistTable::Playlist).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PlaylistTable {
    Playlist,
}
