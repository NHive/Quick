// file_path: src/window/control.rs
// use super::quick_window::show_previous_window;
use tauri::{self, Error, Manager, Runtime};

use crate::logic::events::app_events::hide_all_managed_windows;
use crate::logic::window_manager::operations;
use crate::CONTROL_WINDOW_LABEL;

pub async fn show_control_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Error> {
    // 显示quick窗口时会显示控制窗口
    operations::show_previous_window(app).await?;
    Ok(())
}

pub async fn hide_control_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Error> {
    hide_all_managed_windows(app).await;
    Ok(())
}

pub async fn toggle_control_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Error> {
    let Some(control_window) = app.get_webview_window(CONTROL_WINDOW_LABEL) else {
        return Ok(());
    };

    if control_window.is_visible()? {
        hide_all_managed_windows(app).await;
    } else {
        // 显示quick窗口时会显示控制窗口
        operations::show_previous_window(app).await?;
    }

    Ok(())
}
