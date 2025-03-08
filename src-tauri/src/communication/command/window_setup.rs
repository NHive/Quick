// file_path: src/communication/command/window_setup.rs
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager, Runtime};

use crate::logic::events::app_events::WindowFocusState;

#[tauri::command]
pub async fn get_window_pin<R: Runtime>(app_handle: AppHandle<R>) -> Result<bool, String> {
    let state = app_handle.state::<Arc<RwLock<WindowFocusState>>>();
    let state = state.read().unwrap(); // 获取读锁
    let is_pinned = state.is_pinned();
    Ok(is_pinned)
}

#[tauri::command]
pub async fn set_window_pin<R: Runtime>(app_handle: AppHandle<R>, pin: bool) -> Result<(), String> {
    let state = app_handle.state::<Arc<RwLock<WindowFocusState>>>();
    let mut state = state.write().unwrap(); // 获取写锁
    state.set_pinned(pin);
    Ok(())
}
