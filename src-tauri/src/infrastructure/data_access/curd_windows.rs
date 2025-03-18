// file_path: src/infrastructure/data_access/curd_windows.rs
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, Set,
};

use crate::infrastructure::entity::windows::{ActiveModel, Column, Entity, Model};
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
    // 获取所有窗口
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Model>, AppError> {
        let windows = Entity::find().all(db).await?;
        Ok(windows)
    }

    // 通过ID查找窗口
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, AppError> {
        let window = Entity::find_by_id(id).one(db).await?;
        Ok(window)
    }

    // 通过代理ID查找窗口
    pub async fn find_by_proxy_id(
        db: &DatabaseConnection,
        proxy_id: i32,
    ) -> Result<Vec<Model>, AppError> {
        let windows = Entity::find()
            .filter(Column::ProxyId.eq(proxy_id))
            .all(db)
            .await?;
        Ok(windows)
    }

    // 获取指定窗口或返回NotFoundError
    pub async fn find_by_id_or_error(db: &DatabaseConnection, id: i32) -> Result<Model, AppError> {
        let window = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::new_not_found(format!("找不到ID为{}的窗口", id)))?;
        Ok(window)
    }
}

impl Insert {
    // 添加新窗口
    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        url: String,
        icon: Option<String>,
        sort_order: i32,
        proxy_id: Option<i32>,
        shortcut: Option<String>,
    ) -> Result<Model, AppError> {
        // 验证输入
        if title.is_empty() {
            return Err(AppError::new_validation_error("窗口标题不能为空"));
        }
        if url.is_empty() {
            return Err(AppError::new_validation_error("窗口URL不能为空"));
        }

        let now = Utc::now().to_rfc3339();

        let window = ActiveModel {
            title: Set(title),
            url: Set(url),
            icon: Set(icon),
            sort_order: Set(sort_order),
            proxy_id: Set(proxy_id),
            shortcut: Set(shortcut),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = window.insert(db).await?;
        Ok(result)
    }
}

impl Update {
    // 更新窗口信息
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        title: Option<String>,
        url: Option<String>,
        icon: Option<Option<String>>,
        sort_order: Option<i32>,
        proxy_id: Option<Option<i32>>,
        shortcut: Option<Option<String>>,
    ) -> Result<Model, AppError> {
        // 验证输入
        if let Some(ref title) = title {
            if title.is_empty() {
                return Err(AppError::new_validation_error("窗口标题不能为空"));
            }
        }
        if let Some(ref url) = url {
            if url.is_empty() {
                return Err(AppError::new_validation_error("窗口URL不能为空"));
            }
        }

        let window = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::new_not_found(format!("找不到ID为{}的窗口", id)))?;

        let now = Utc::now().to_rfc3339();
        let mut window: ActiveModel = window.into();

        if let Some(title) = title {
            window.title = Set(title);
        }

        if let Some(url) = url {
            window.url = Set(url);
        }

        if let Some(icon) = icon {
            window.icon = Set(icon);
        }

        if let Some(sort_order) = sort_order {
            window.sort_order = Set(sort_order);
        }

        if let Some(proxy_id) = proxy_id {
            window.proxy_id = Set(proxy_id);
        }

        if let Some(shortcut) = shortcut {
            window.shortcut = Set(shortcut);
        }

        window.updated_at = Set(now);

        let updated_window = window.update(db).await?;
        Ok(updated_window)
    }
}

impl Delete {
    // 删除窗口
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<bool, AppError> {
        let window = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::new_not_found(format!("找不到ID为{}的窗口", id)))?;

        let result = window.delete(db).await?;
        Ok(result.rows_affected > 0)
    }

    // 删除与特定代理关联的所有窗口
    pub async fn delete_by_proxy_id(
        db: &DatabaseConnection,
        proxy_id: i32,
    ) -> Result<u64, AppError> {
        let result = Entity::delete_many()
            .filter(Column::ProxyId.eq(proxy_id))
            .exec(db)
            .await?;

        Ok(result.rows_affected)
    }
}
