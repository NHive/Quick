// file_path: migration/src/main.rs
use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
