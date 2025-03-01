mod window;

use tauri::generate_handler;

use window::*;

pub fn register_commands<R: tauri::Runtime>() -> impl Fn(tauri::Builder<R>) -> tauri::Builder<R> {
    move |app_builder| {
        app_builder.invoke_handler(generate_handler![open_window_by_url, open_setting_window])
    }
}
