use tauri::{AppHandle, Runtime};

use crate::window::quick_window;
use crate::window::setting;

#[tauri::command]
pub fn cmd_create_window<R: Runtime>(
    app_handle: AppHandle<R>,
    url: String,
    title: String,
) -> Result<(), String> {
    match quick_window::create_or_switch_window(&app_handle, &url, &title) {
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
pub fn cmd_configure_windows<R: Runtime>(
    app_handle: AppHandle<R>,
    configs: Vec<quick_window::WindowConfig>,
) -> Result<(), String> {
    match quick_window::configure_windows(&app_handle, configs) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 切换窗口
#[tauri::command]
pub fn cmd_switch_to_window<R: Runtime>(
    app_handle: AppHandle<R>,
    label: String,
) -> Result<(), String> {
    match quick_window::switch_to_window(&app_handle, &label) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 获取所有窗口信息
#[tauri::command]
pub fn cmd_get_all_windows<R: Runtime>(app_handle: AppHandle<R>) -> Vec<quick_window::WindowInfo> {
    quick_window::get_all_windows(&app_handle)
}

// 获取活动窗口信息
#[tauri::command]
pub fn cmd_get_active_window<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Option<quick_window::WindowInfo> {
    quick_window::get_active_window(&app_handle)
}

// 获取之前活动的窗口信息
#[tauri::command]
pub fn cmd_get_previous_window<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Option<quick_window::WindowInfo> {
    quick_window::get_previous_active_window(&app_handle)
}

// 显示上一个活动窗口
#[tauri::command]
pub fn cmd_show_previous_window<R: Runtime>(app_handle: AppHandle<R>) -> Result<(), String> {
    match quick_window::show_previous_window(&app_handle) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// 从url生成窗口标签
#[tauri::command]
pub fn cmd_generate_window_label(url: String) -> String {
    quick_window::generate_window_label(&url)
}
