// file_path: migration/src/m20250307_104412_window_proxy_manager.rs
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建proxies表
        manager
            .create_table(
                Table::create()
                    .table(Proxies::Table)
                    .if_not_exists()
                    .col(pk_auto(Proxies::Id))
                    .col(string(Proxies::Title).not_null().unique_key())
                    .col(string(Proxies::Type).not_null())
                    .col(string(Proxies::Host).not_null())
                    .col(integer(Proxies::Port).not_null())
                    .col(string_null(Proxies::Username).null())
                    .col(string_null(Proxies::Password).null())
                    .col(string(Proxies::CreatedAt).default(Expr::current_timestamp()))
                    .col(string(Proxies::UpdatedAt).default(Expr::current_timestamp()))
                    .check(Expr::col(Proxies::Type).is_in(vec!["http", "socks"]))
                    .to_owned(),
            )
            .await?;

        // 创建windows表 (包含ProxyId外键)
        manager
            .create_table(
                Table::create()
                    .table(Windows::Table)
                    .if_not_exists()
                    .col(pk_auto(Windows::Id))
                    .col(string(Windows::Title).not_null().unique_key())
                    .col(string(Windows::Url).not_null())
                    .col(string_null(Windows::Icon).null())
                    .col(integer(Windows::SortOrder).not_null())
                    .col(integer_null(Windows::ProxyId).null())
                    .col(string_null(Windows::Shortcut).null())
                    .col(string(Windows::CreatedAt).default(Expr::current_timestamp()))
                    .col(string(Windows::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Windows::Table, Windows::ProxyId)
                            .to(Proxies::Table, Proxies::Id)
                            .on_delete(ForeignKeyAction::SetNull), // 代理删除时设置为null
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除表，注意删除顺序：先删除有外键引用的表
        manager
            .drop_table(Table::drop().table(Windows::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Proxies::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Windows表结构 (添加ProxyId字段)
#[derive(DeriveIden)]
enum Windows {
    Table,
    Id,
    Title,
    Url,
    Icon,
    SortOrder,
    ProxyId,
    Shortcut,
    CreatedAt,
    UpdatedAt,
}

// Proxies表结构
#[derive(DeriveIden)]
enum Proxies {
    Table,
    Id,
    Title,
    Type,
    Host,
    Port,
    Username,
    Password,
    CreatedAt,
    UpdatedAt,
}
