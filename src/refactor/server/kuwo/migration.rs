use async_trait::async_trait;
use sea_orm_migration::{
    prelude::*,
    schema::{pk_auto, string, string_null},
};

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
                    .col(pk_auto(Column::MusicId))
                    .col(string(Column::Name))
                    .col(string(Column::Artist))
                    .col(string(Column::AlbumId))
                    .col(string_null(Column::Album))
                    .col(string(Column::ArtistId))
                    .col(string(Column::Qualities))
                    .col(string(Column::MusicPic))
                    .col(string_null(Column::ArtistPic))
                    .col(string_null(Column::AlbumPic))
                    // .col(string_null(Column::MvVid))
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KuwoMusicTable::KuwoMusic).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum KuwoMusicTable {
    #[sea_orm(iden = "kuwo_music")]
    KuwoMusic,
}
