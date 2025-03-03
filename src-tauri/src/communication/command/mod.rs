// file_path: src/communication/command/mod.rs
mod window;

use tauri::generate_handler;

use window::*;

pub fn register_commands<R: tauri::Runtime>() -> impl Fn(tauri::Builder<R>) -> tauri::Builder<R> {
    move |app_builder| {
        app_builder.invoke_handler(generate_handler![
            cmd_create_window,
            open_setting_window,
            cmd_configure_windows,
            cmd_switch_to_window,
            cmd_get_all_windows,
            cmd_get_active_window,
            cmd_get_previous_window,
            cmd_show_previous_window,
            cmd_generate_window_label,
        ])
    }
}
