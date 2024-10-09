use async_trait::async_trait;
use sea_orm_migration::{
    prelude::*,
    schema::{big_integer, string},
};

use crate::data::models::playlist_collection::Column;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlaylistCollectionTable::PlaylistCollection)
                    .col(
                        big_integer(Column::Id)
                            .auto_increment()
                            .primary_key()
                            .not_null(),
                    )
                    .col(string(Column::Name))
                    .col(big_integer(Column::Order).not_null())
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(PlaylistCollectionTable::PlaylistCollection)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum PlaylistCollectionTable {
    PlaylistCollection,
}
