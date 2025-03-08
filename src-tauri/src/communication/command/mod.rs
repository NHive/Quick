// file_path: src/communication/command/mod.rs
mod setup;
mod window;
mod window_setup;

use tauri::generate_handler;

use setup::*;
use window::*;
use window_setup::*;

pub fn register_commands<R: tauri::Runtime>() -> impl Fn(tauri::Builder<R>) -> tauri::Builder<R> {
    move |app_builder| {
        app_builder.invoke_handler(generate_handler![
            open_setting_window,
            cmd_load_configure_windows,
            cmd_switch_to_window,
            cmd_get_all_windows,
            cmd_get_active_window,
            cmd_get_previous_window,
            cmd_show_previous_window,
            get_setup,
            set_setup,
            init_setup,
            cmd_clear_cache,
            get_window_pin,
            set_window_pin,
            cmd_hide_control_window,
        ])
    }
}
