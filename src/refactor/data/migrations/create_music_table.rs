use sea_orm_migration::prelude::*;
use async_trait::async_trait;

use crate::refactor::data::models::music::Column;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MusicTable::Music).col(ColumnDef::new(Column::Id).integer().primary_key().auto_increment()).to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(  )
            .await
    }
}


#[derive(DeriveIden)]
enum MusicTable {
    Music

}