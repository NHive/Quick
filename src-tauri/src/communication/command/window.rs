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
