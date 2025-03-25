// file_path: src/infrastructure/data_access/curd_proxies.rs
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, Set,
};

use crate::infrastructure::entity;
use crate::infrastructure::entity::proxies::{ActiveModel, Column, Entity, Model};
use crate::infrastructure::error::AppError;

#[allow(unused)]
pub struct Query;
#[allow(unused)]
pub struct Insert;
#[allow(unused)]
pub struct Update;
#[allow(unused)]
pub struct Delete;

impl Query {
    // 获取所有代理
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Model>, AppError> {
        let proxies = Entity::find().all(db).await?;
        Ok(proxies)
    }

    // 通过ID查找代理
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, AppError> {
        let proxy = Entity::find_by_id(id).one(db).await?;
        Ok(proxy)
    }

    // 获取指定代理或返回NotFoundError
    pub async fn find_by_id_or_error(db: &DatabaseConnection, id: i32) -> Result<Model, AppError> {
        let proxy = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::new_not_found(format!("找不到ID为{}的代理", id)))?;
        Ok(proxy)
    }

    // 通过代理类型查找代理
    pub async fn find_by_type(
        db: &DatabaseConnection,
        proxy_type: &str,
    ) -> Result<Vec<Model>, AppError> {
        let proxies = Entity::find()
            .filter(Column::Type.eq(proxy_type))
            .all(db)
            .await?;
        Ok(proxies)
    }

    // 获取代理及其关联的所有窗口
    pub async fn find_with_windows(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Option<(Model, Vec<entity::windows::Model>)>, AppError> {
        let proxy_with_windows = Entity::find_by_id(id)
            .find_with_related(entity::windows::Entity)
            .all(db)
            .await?;

        if proxy_with_windows.is_empty() {
            return Ok(None);
        }

        Ok(Some((
            proxy_with_windows[0].0.clone(),
            proxy_with_windows[0].1.clone(),
        )))
    }
}

impl Insert {
    // 添加新代理
    pub async fn create(
        db: &DatabaseConnection,
        proxy_type: String,
        host: String,
        port: i32,
        username: Option<String>,
        password: Option<String>,
        title: Option<String>, // 新增title参数，可选
    ) -> Result<Model, AppError> {
        // 验证输入
        if proxy_type.is_empty() {
            return Err(AppError::new_validation_error("代理类型不能为空"));
        }
        if host.is_empty() {
            return Err(AppError::new_validation_error("代理主机地址不能为空"));
        }
        if port <= 0 || port > 65535 {
            return Err(AppError::new_validation_error("无效的端口号(1-65535)"));
        }

        let now = Utc::now().to_rfc3339();

        // 如果没有提供title，则使用host:port作为默认值
        let default_title = format!("{}:{}", host, port);
        let title = title.unwrap_or(default_title);

        let proxy = ActiveModel {
            r#type: Set(proxy_type),
            host: Set(host.clone()),
            port: Set(port),
            username: Set(username),
            password: Set(password),
            title: Set(title), // 设置title字段
            created_at: Set(now.clone()),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = proxy.insert(db).await?;
        Ok(result)
    }
}

impl Update {
    // 更新代理信息
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        proxy_type: Option<String>,
        host: Option<String>,
        port: Option<i32>,
        username: Option<Option<String>>,
        password: Option<Option<String>>,
    ) -> Result<Model, AppError> {
        // 验证输入
        if let Some(ref proxy_type) = proxy_type {
            if proxy_type.is_empty() {
                return Err(AppError::new_validation_error("代理类型不能为空"));
            }
        }
        if let Some(ref host) = host {
            if host.is_empty() {
                return Err(AppError::new_validation_error("代理主机地址不能为空"));
            }
        }
        if let Some(port) = port {
            if port <= 0 || port > 65535 {
                return Err(AppError::new_validation_error("无效的端口号(1-65535)"));
            }
        }

        let proxy = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::new_not_found(format!("找不到ID为{}的代理", id)))?;

        let now = Utc::now().to_rfc3339();
        let mut proxy: ActiveModel = proxy.into();

        if let Some(proxy_type) = proxy_type {
            proxy.r#type = Set(proxy_type);
        }

        if let Some(host) = host {
            proxy.host = Set(host);
        }

        if let Some(port) = port {
            proxy.port = Set(port);
        }

        if let Some(username) = username {
            proxy.username = Set(username);
        }

        if let Some(password) = password {
            proxy.password = Set(password);
        }

        proxy.updated_at = Set(now);

        let updated_proxy = proxy.update(db).await?;
        Ok(updated_proxy)
    }
}

impl Delete {
    // 删除代理
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<bool, AppError> {
        let proxy = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::new_not_found(format!("找不到ID为{}的代理", id)))?;

        let result = proxy.delete(db).await?;
        Ok(result.rows_affected > 0)
    }
}
