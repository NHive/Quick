use tauri::{AppHandle, Runtime};

use crate::app_window::quick_window;

#[tauri::command]
pub async fn open_window_by_url<R: Runtime>(
    app_handle: AppHandle<R>,
    url: String,
) -> Result<(), String> {
    // 如果 path 为空，则默认打开用户设置页面
    match quick_window::create_quick_window(&app_handle, &url) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
