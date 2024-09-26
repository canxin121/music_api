use async_trait::async_trait;
use sea_orm_migration::{
    prelude::*,
    schema::{big_integer_null, json, string, string_null},
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
                    .table(KuwoMusicTable::KuwoMusic)
                    .col(
                        ColumnDef::new(Column::MusicId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(Column::Name))
                    .col(json(Column::Artists))
                    .col(string_null(Column::Album))
                    .col(string_null(Column::AlbumId))
                    .col(json(Column::Qualities))
                    .col(string(Column::Cover))
                    // .col(string_null(Column::ArtistPic))
                    // .col(string_null(Column::AlbumPic))
                    .col(big_integer_null(Column::Duration))
                    .foreign_key(
                        ForeignKey::create()
                            .from(KuwoMusicTable::KuwoMusic, Column::MusicId)
                            .to(
                                create_music_aggregator_table::MusicAggregatorTable::MusicAggragator,
                                music_aggregator::Column::KuwoMusicId,
                            )
                            .on_delete(ForeignKeyAction::Cascade)
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
                    .table(KuwoMusicTable::KuwoMusic)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum KuwoMusicTable {
    #[sea_orm(iden = "kuwo_music")]
    KuwoMusic,
}
