mod app_window;
mod command;
mod error;
mod system_tools;
mod tray;

use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();

    let mut app_builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 创建托盘
            let _ = tray::menu(app.handle());
            Ok(())
        });

    app_builder = command::register_commands()(app_builder);

    let mut app = app_builder
        // 注册http请求插件
        .plugin(tauri_plugin_http::init())
        // 注册系统api插件
        .plugin(tauri_plugin_os::init())
        // 注册全局快捷键插件
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // 注册系统原生通知插件
        .plugin(tauri_plugin_notification::init())
        // 注册shell插件
        .plugin(tauri_plugin_shell::init())
        // 注册文件读写插件
        .plugin(tauri_plugin_fs::init())
        // 注册开机启动插件
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::AppleScript,
            None,
        ))
        .build(context)
        .expect("Failed to build Tauri application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(handle_app_events);
}

fn handle_app_events(app_handle: &tauri::AppHandle, event: tauri::RunEvent) {
    if let tauri::RunEvent::WindowEvent { label, event, .. } = event {}
}
