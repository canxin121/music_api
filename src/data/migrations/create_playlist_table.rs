use async_trait::async_trait;
use sea_orm_migration::{
    prelude::*,
    schema::{big_integer, json_null, string, string_null, text_null},
};

use crate::data::models::playlist::Column;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlaylistTable::Playlist)
                    .col(
                        big_integer(Column::Id)
                            .auto_increment()
                            .primary_key()
                            .not_null(),
                    )
                    .col(string(Column::Name))
                    .col(text_null(Column::Summary))
                    .col(string_null(Column::Cover))
                    .col(big_integer(Column::Order).not_null())
                    .col(json_null(Column::Subscriptions))
                    .col(big_integer(Column::CollectionId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlaylistTable::Playlist, Column::CollectionId)
                            .to(super::create_playlist_collection_table::PlaylistCollectionTable::PlaylistCollection, crate::data::models::playlist_collection::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
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
                    .table(PlaylistTable::Playlist)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum PlaylistTable {
    Playlist,
}
