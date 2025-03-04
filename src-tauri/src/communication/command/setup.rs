use crate::infrastructure::setup::SetupService;
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

#[tauri::command]
pub async fn get_setup<R: Runtime>(
    app_handle: AppHandle<R>,
    key: String,
) -> Result<Option<Value>, String> {
    let data = app_handle
        .try_state::<SetupService>()
        .unwrap()
        .get_setup_async(&key)
        .await;
    match data {
        Ok(value) => Ok(value),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn set_setup<R: Runtime>(
    app_handle: AppHandle<R>,
    key: String,
    value: Value,
) -> Result<(), String> {
    let result = app_handle
        .try_state::<SetupService>()
        .unwrap()
        .set_setup_async(&key, &value)
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
