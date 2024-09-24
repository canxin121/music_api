use async_trait::async_trait;
use sea_orm_migration::{
    prelude::*,
    schema::{big_integer, string},
};

use crate::data::models::{music_aggregator, playlist, playlist_music_junction::Column};

use super::{
    create_music_aggregator_table::MusicAggregatorTable, create_playlist_table::PlaylistTable,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlaylistMusicJunctionTable::PlaylistMusicJunction)
                    .col(big_integer(Column::PlaylistId).not_null())
                    .col(string(Column::MusicAggregatorId))
                    .col(big_integer(Column::Order).not_null())
                    .primary_key(
                        Index::create()
                            .table(PlaylistMusicJunctionTable::PlaylistMusicJunction)
                            .col(Column::PlaylistId)
                            .col(Column::MusicAggregatorId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                PlaylistMusicJunctionTable::PlaylistMusicJunction,
                                Column::PlaylistId,
                            )
                            .to(PlaylistTable::Playlist, playlist::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                PlaylistMusicJunctionTable::PlaylistMusicJunction,
                                Column::MusicAggregatorId,
                            )
                            .to(
                                MusicAggregatorTable::MusicAggragator,
                                music_aggregator::Column::Identity,
                            )
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
                    .table(PlaylistMusicJunctionTable::PlaylistMusicJunction)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PlaylistMusicJunctionTable {
    #[sea_orm(iden = "playlist_music_junction")]
    PlaylistMusicJunction,
}
