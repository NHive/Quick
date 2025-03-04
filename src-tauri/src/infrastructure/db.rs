// file_path: src/infrastructure/db.rs
use crate::infrastructure::error::AppError;
use crate::logic::tools::path::AppPath;
use migration::{Migrator, MigratorTrait};
use once_cell::sync::Lazy;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

type Result<T> = std::result::Result<T, AppError>;

static DB_CONN: Lazy<Arc<RwLock<Option<DatabaseConnection>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

pub struct DB;

impl DB {
    pub async fn init(app_path: &AppPath) -> Result<Self> {
        let conn = Self::get_or_create_connection(app_path).await?;

        // 获取当前数据库版本
        let current_version = match Migrator::get_applied_migrations(&conn).await {
            Ok(versions) => {
                for version in &versions {
                    log::info!("已应用的迁移版本: {}", version.name());
                }

                versions.last().and_then(|m| {
                    m.name().strip_prefix('m').and_then(|s| {
                        let parts: Vec<&str> = s.split('_').collect();
                        if parts.len() >= 2 {
                            format!("{}{}", parts[0], parts[1]).parse::<i64>().ok()
                        } else {
                            None
                        }
                    })
                })
            }
            Err(e) => {
                log::warn!("获取已应用的迁移失败: {:?}", e);
                None
            }
        };

        // 获取待处理的迁移
        let pending_migrations = match Migrator::get_pending_migrations(&conn).await {
            Ok(migrations) => {
                for migration in &migrations {
                    log::info!("待处理的迁移: {}", migration.name());
                }
                migrations
            }
            Err(e) => {
                log::warn!("获取待处理的迁移失败: {:?}", e);
                Vec::new()
            }
        };

        if !pending_migrations.is_empty() {
            log::info!(
                "数据库迁移状态 - 当前版本: {}, 待处理的迁移: {}",
                current_version.map_or("未初始化".to_string(), |v| v.to_string()),
                pending_migrations.len()
            );

            // 运行所有迁移
            if let Err(e) = Migrator::up(&conn, None).await {
                log::error!("运行迁移失败: {:?}", e);
                return Err(AppError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "数据库迁移失败",
                )));
            }
        }

        Ok(Self)
    }

    pub async fn get_connection() -> Result<DatabaseConnection> {
        let conn_guard = DB_CONN.read().await;
        conn_guard.clone().ok_or_else(|| {
            AppError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "数据库连接未初始化",
            ))
        })
    }

    async fn get_or_create_connection(app_path: &AppPath) -> Result<DatabaseConnection> {
        let mut conn_guard = DB_CONN.write().await;
        if conn_guard.is_none() {
            let db_path = app_path.base_path.join("store.sqlite");
            let db_url = format!(
                "sqlite://{}/store.sqlite?mode=rwc",
                app_path.base_path.to_string_lossy()
            );

            let mut opt = ConnectOptions::new(&db_url);

            // 在生产环境中禁用 SQL 日志记录
            opt.sqlx_logging(cfg!(debug_assertions));

            // 连接数据库
            match Database::connect(opt).await {
                Ok(conn) => {
                    // 验证数据库连接
                    match conn
                        .execute(sea_orm::Statement::from_string(
                            sea_orm::DatabaseBackend::Sqlite,
                            "SELECT 1".to_owned(),
                        ))
                        .await
                    {
                        Ok(_) => {
                            // 启用 WAL 模式以提高性能
                            if let Err(e) = conn
                                .execute(sea_orm::Statement::from_string(
                                    sea_orm::DatabaseBackend::Sqlite,
                                    "PRAGMA journal_mode=WAL".to_owned(),
                                ))
                                .await
                            {
                                log::error!("启用 WAL 模式失败: {:?}", e);
                            }
                            *conn_guard = Some(conn);
                        }
                        Err(e) => {
                            log::error!("数据库验证失败: {:?}", e);
                            return Err(AppError::IOError(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("数据库验证失败: {}", e),
                            )));
                        }
                    }
                }
                Err(e) => {
                    log::error!("连接数据库失败: {:?}", e);
                    return Err(AppError::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("数据库连接失败: {}", e),
                    )));
                }
            }
        }

        conn_guard.clone().ok_or_else(|| {
            AppError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "初始化数据库连接失败",
            ))
        })
    }

    /// 删除数据库文件
    pub fn remove_database(app_path: &AppPath) -> Result<()> {
        // 删除主数据库文件
        let db_path = app_path.base_path.join("store.sqlite");
        if db_path.exists() {
            if let Err(e) = fs::remove_file(&db_path) {
                log::error!("删除数据库文件失败: {:?}", e);
                return Err(AppError::IOError(e));
            }
        }

        // 删除 WAL 和 SHM 文件
        let wal_path = db_path.with_extension("sqlite-wal");
        let shm_path = db_path.with_extension("sqlite-shm");

        if wal_path.exists() {
            if let Err(e) = fs::remove_file(&wal_path) {
                log::error!("删除 WAL 文件失败: {:?}", e);
            }
        }

        if shm_path.exists() {
            if let Err(e) = fs::remove_file(&shm_path) {
                log::error!("删除 SHM 文件失败: {:?}", e);
            }
        }

        Ok(())
    }
}
