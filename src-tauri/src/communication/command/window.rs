// file_path: src/communication/command/window.rs
use tauri::{AppHandle, Runtime};

// use crate::window::quick_window;
use crate::logic::window_manager::manager::WindowManager;
use crate::logic::window_manager::models;
use crate::logic::window_manager::operations;

use crate::window::{quick, setting};

// 打开设置窗口
#[tauri::command]
pub async fn open_setting_window<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    match setting::create_settings_window(&app_handle, "customSetting") {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 加载窗口配置
#[tauri::command]
pub async fn cmd_load_configure_windows<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<(), String> {
    match operations::load_window_configs_from_db(&app_handle).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 切换窗口
#[tauri::command]
pub async fn cmd_switch_to_window<R: Runtime>(
    app_handle: AppHandle<R>,
    label: String,
) -> Result<(), String> {
    match operations::switch_to_window(&app_handle, &label).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 获取所有窗口信息
#[tauri::command]
pub async fn cmd_get_all_windows() -> Vec<models::WindowInfo> {
    WindowManager::get_windows()
}

// 获取活动窗口信息
#[tauri::command]
pub async fn cmd_get_active_window() -> Option<models::WindowInfo> {
    WindowManager::get_active_window()
}

// 获取之前活动的窗口信息
#[tauri::command]
pub async fn cmd_get_previous_window() -> Option<models::WindowInfo> {
    WindowManager::get_previous_active_window()
}

// 显示上一个活动窗口
#[tauri::command]
pub async fn cmd_show_previous_window<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    match operations::show_quick_window(&app_handle).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 清理指定所有的浏览器缓存
#[tauri::command]
pub async fn cmd_clear_cache<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    match operations::clear_window_cache(&app_handle).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 隐藏控制窗口
#[tauri::command]
pub async fn cmd_hide_control_window<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    match quick::hide_control_window(&app_handle).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
