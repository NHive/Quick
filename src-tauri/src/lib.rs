mod window;
use std::sync::{Arc, Mutex};
mod communication;
mod infrastructure;
mod logic;

use tauri::Manager;

use communication::events::app_events::WindowFocusState;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use window::quick_window::{WindowManager, WindowManagerState};
use window::window_layout::{WindowPositionTracker, WindowPositionTrackerState};

use communication::events::app_events::handle_app_events;
use communication::events::global_shortcut::global_shortcuts_handle;

pub const CONTROL_WINDOW_LABEL: &str = "control"; // 控制窗口标签
pub const SETTING_WINDOW_LABEL: &str = "setting"; // 设置窗口标签

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();

    let mut app_builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // 注册全局快捷键插件
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    global_shortcuts_handle(app, shortcut, event)
                })
                .build(),
        )
        .setup(|app| {
            // 创建托盘
            let _ = infrastructure::tray::menu(app.handle());
            let window_manager = WindowManager::new();
            app.manage(WindowManagerState(Arc::new(Mutex::new(window_manager))));
            app.manage(WindowFocusState::default());
            app.manage(WindowPositionTrackerState(Arc::new(Mutex::new(
                WindowPositionTracker::new(),
            ))));
            Ok(())
        });

    app_builder = communication::command::register_commands()(app_builder);

    let mut app = app_builder
        // 注册http请求插件
        .plugin(tauri_plugin_http::init())
        // 注册系统api插件
        .plugin(tauri_plugin_os::init())
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

    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

    let open_control_window = Shortcut::new(Some(Modifiers::ALT), Code::KeyC);
    app.global_shortcut().register(open_control_window).unwrap();

    app.run(handle_app_events);
}
