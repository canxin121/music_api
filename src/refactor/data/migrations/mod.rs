pub mod create_music_table;

pub use sea_orm_migration::*;
use async_trait::async_trait;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
        ]
    }
}