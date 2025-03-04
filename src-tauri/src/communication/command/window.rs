// file_path: src/communication/command/window.rs
use tauri::{AppHandle, Runtime};

// use crate::window::quick_window;
use crate::logic::window_manager::models;
use crate::logic::window_manager::operations;

use crate::window::setting;

#[tauri::command]
pub async fn cmd_create_window<R: Runtime>(
    app_handle: AppHandle<R>,
    url: String,
    title: String,
) -> Result<(), String> {
    match operations::create_or_switch_window(&app_handle, &url, &title) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn open_setting_window<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    match setting::create_settings_window(&app_handle, "customSetting") {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

//  配置窗口列表
#[tauri::command]
pub async fn cmd_configure_windows<R: Runtime>(
    app_handle: AppHandle<R>,
    configs: Vec<models::WindowConfig>,
) -> Result<(), String> {
    match operations::configure_windows(&app_handle, configs) {
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
    match operations::switch_to_window(&app_handle, &label) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 获取所有窗口信息
#[tauri::command]
pub async fn cmd_get_all_windows<R: Runtime>(app_handle: AppHandle<R>) -> Vec<models::WindowInfo> {
    operations::get_all_windows(&app_handle)
}

// 获取活动窗口信息
#[tauri::command]
pub async fn cmd_get_active_window<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Option<models::WindowInfo> {
    operations::get_active_window(&app_handle)
}

// 获取之前活动的窗口信息
#[tauri::command]
pub async fn cmd_get_previous_window<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Option<models::WindowInfo> {
    operations::get_previous_active_window(&app_handle)
}

// 显示上一个活动窗口
#[tauri::command]
pub async fn cmd_show_previous_window<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    match operations::show_previous_window(&app_handle) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 清理指定标签的浏览器缓存
#[tauri::command]
pub async fn cmd_clear_cache<R: Runtime>(
    app_handle: AppHandle<R>,
    label: String,
) -> Result<(), String> {
    match operations::clear_window_cache(&app_handle, &label) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
