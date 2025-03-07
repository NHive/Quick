// file_path: migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250307_104412_window_proxy_manager;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250307_104412_window_proxy_manager::Migration),
        ]
    }
}
