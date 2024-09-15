pub mod create_music_aggregator_table;
pub mod create_playlist_music_junction_table;
pub mod create_playlist_table;

use async_trait::async_trait;
pub use sea_orm_migration::*;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(create_playlist_table::Migration),
            Box::new(create_music_aggregator_table::Migration),
            Box::new(create_playlist_music_junction_table::Migration),
            // todo: add more music server
            Box::new(crate::server::kuwo::create_kuwo_music_table_migration::Migration),
            Box::new(crate::server::netease::create_netease_music_table_migration::Migration),
        ]
    }
}
