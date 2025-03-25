// file_path: src/logic/service/setting_window.rs
use crate::infrastructure::data_access::curd_proxies;
use crate::infrastructure::data_access::curd_windows;
use crate::infrastructure::db::DB;
use crate::infrastructure::error::AppError;
use crate::types::window::{CreateWindowRequest, UpdateWindowRequest, WindowInfo};

// 设置页添加窗口
pub struct WindowInfoService;

impl WindowInfoService {
    // 添加新窗口
    pub async fn create(
        title: String,
        url: String,
        icon: Option<String>,
        proxy_id: Option<i32>,
        shortcut: Option<String>,
        is_default: Option<bool>,
    ) -> Result<WindowInfo, AppError> {
        let db = DB::get_connection().await?;

        // 如果提供了代理ID，确保代理存在
        if let Some(proxy_id) = proxy_id {
            if let Err(_) = curd_proxies::Query::find_by_id_or_error(&db, proxy_id).await {
                return Err(AppError::new_validation_error(format!(
                    "ID为{}的代理不存在",
                    proxy_id
                )));
            }
        }

        // 获取当前最大排序值并+1
        let windows = curd_windows::Query::find_all(&db).await?;
        let max_sort_order = windows.iter().map(|w| w.sort_order).max().unwrap_or(0);
        let sort_order = max_sort_order + 1;

        // 创建窗口
        let window = curd_windows::Insert::create(
            &db, title, url, icon, sort_order, proxy_id, shortcut, is_default,
        )
        .await?;

        // 转换为返回类型
        Ok(WindowInfo {
            id: window.id,
            title: window.title,
            url: window.url,
            icon: window.icon,
            sort_order: window.sort_order,
            proxy_id: window.proxy_id,
            shortcut: window.shortcut,
            is_default: window.is_default,
            created_at: window.created_at,
            updated_at: window.updated_at,
        })
    }

    // 从请求创建窗口
    pub async fn create_from_request(request: CreateWindowRequest) -> Result<WindowInfo, AppError> {
        Self::create(
            request.title,
            request.url,
            request.icon,
            request.proxy_id,
            request.shortcut,
            request.is_default,
        )
        .await
    }

    // 更新窗口
    pub async fn update(request: UpdateWindowRequest) -> Result<WindowInfo, AppError> {
        let db = DB::get_connection().await?;

        // 如果提供了代理ID，确保代理存在
        if let Some(Some(proxy_id)) = request.proxy_id {
            if let Err(_) = curd_proxies::Query::find_by_id_or_error(&db, proxy_id).await {
                return Err(AppError::new_validation_error(format!(
                    "ID为{}的代理不存在",
                    proxy_id
                )));
            }
        }

        // 更新窗口
        let window = curd_windows::Update::update(
            &db,
            request.id,
            request.title,
            request.url,
            request.icon,
            request.sort_order,
            request.proxy_id,
            request.shortcut,
            request.is_default,
        )
        .await?;

        // 转换为返回类型
        Ok(WindowInfo {
            id: window.id,
            title: window.title,
            url: window.url,
            icon: window.icon,
            sort_order: window.sort_order,
            proxy_id: window.proxy_id,
            shortcut: window.shortcut,
            is_default: window.is_default,
            created_at: window.created_at,
            updated_at: window.updated_at,
        })
    }

    // 删除窗口
    pub async fn delete(id: i32) -> Result<bool, AppError> {
        let db = DB::get_connection().await?;
        curd_windows::Delete::delete(&db, id).await
    }

    // 获取所有窗口
    pub async fn get_all() -> Result<Vec<WindowInfo>, AppError> {
        let db = DB::get_connection().await?;
        let windows = curd_windows::Query::find_all(&db).await?;

        let mut window_infos = windows
            .into_iter()
            .map(|window| WindowInfo {
                id: window.id,
                title: window.title,
                url: window.url,
                icon: window.icon,
                sort_order: window.sort_order,
                proxy_id: window.proxy_id,
                shortcut: window.shortcut,
                is_default: window.is_default,
                created_at: window.created_at,
                updated_at: window.updated_at,
            })
            .collect::<Vec<WindowInfo>>();

        // 按照sort_order排序
        window_infos.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));

        Ok(window_infos)
    }

    // 获取单个窗口
    pub async fn get_by_id(id: i32) -> Result<WindowInfo, AppError> {
        let db = DB::get_connection().await?;
        let window = curd_windows::Query::find_by_id_or_error(&db, id).await?;

        Ok(WindowInfo {
            id: window.id,
            title: window.title,
            url: window.url,
            icon: window.icon,
            sort_order: window.sort_order,
            proxy_id: window.proxy_id,
            shortcut: window.shortcut,
            is_default: window.is_default,
            created_at: window.created_at,
            updated_at: window.updated_at,
        })
    }
}
