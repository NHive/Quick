// file_path: migration/src/m20250325_034426_default_window.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 向 Windows 表添加 IsDefault 列，表示是否为默认窗口
        manager
            .alter_table(
                Table::alter()
                    .table(Windows::Table)
                    .add_column(
                        ColumnDef::new(Windows::IsDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除 IsDefault 列
        manager
            .alter_table(
                Table::alter()
                    .table(Windows::Table)
                    .drop_column(Windows::IsDefault)
                    .to_owned(),
            )
            .await
    }
}

// Windows 表结构（添加 IsDefault 字段）
#[derive(DeriveIden)]
enum Windows {
    Table,
    IsDefault,
}
