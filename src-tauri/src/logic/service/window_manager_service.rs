use crate::infrastructure::error::AppError;
use crate::logic::service::setting_window::WindowInfoService;
use crate::logic::window_manager::models::WindowConfig;

pub struct WindowManagerService;

impl WindowManagerService {
    // 从数据库加载所有窗口配置
    pub async fn load_window_configs() -> Result<Vec<WindowConfig>, AppError> {
        // 调用WindowInfoService获取所有窗口信息
        let window_infos = WindowInfoService::get_all().await?;

        // 将数据库中的窗口信息转换为窗口配置
        let configs = window_infos
            .into_iter()
            .map(|window| WindowConfig {
                title: window.title,
                url: window.url,
                icon: window.icon,
                shortcut: window.shortcut,
                proxy_id: window.proxy_id,
            })
            .collect();

        Ok(configs)
    }

    // 获取指定ID的窗口配置
    pub async fn get_window_config_by_id(id: i32) -> Result<WindowConfig, AppError> {
        // 获取指定ID的窗口信息
        let window = WindowInfoService::get_by_id(id).await?;

        // 转换为窗口配置
        let config = WindowConfig {
            title: window.title,
            url: window.url,
            icon: window.icon,
            shortcut: window.shortcut,
            proxy_id: window.proxy_id,
        };

        Ok(config)
    }
}
