// file_path: src/lib.rs
mod communication;
mod infrastructure;
mod logic;
mod types;
mod window;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

use infrastructure::db::DB;
use infrastructure::log::init_logger;
use infrastructure::setup::SetupService;
use logic::events::app_events::handle_app_events;
use logic::events::global_shortcut::{global_shortcuts_handle, register_shortcuts};
use logic::tools::path::AppPath;

#[cfg(debug_assertions)]
use communication::api::start_server;

// 窗口标签常量
pub const CONTROL_WINDOW_LABEL: &str = "control"; // 控制窗口标签
pub const SETTING_WINDOW_LABEL: &str = "setting"; // 设置窗口标签

/// 设置应用程序
fn setup_app(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建托盘
    infrastructure::tray::menu(app.handle())?;

    // 管理应用路径
    let app_path = AppPath::new()?;

    // 初始化日志记录器
    let logger = init_logger(&app_path)?;

    app.manage(logger);

    let db_app_path = app_path.clone();
    tauri::async_runtime::spawn(async move {
        match DB::init(&db_app_path).await {
            Ok(_) => {
                log::info!("数据库初始化成功");
            }
            Err(e) => {
                log::error!("数据库初始化失败: {:?}", e);
            }
        }
    });

    app.manage(app_path);

    // 初始化设置服务
    let setup_service = SetupService::new();
    app.manage(setup_service);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();

    // 构建应用程序
    let mut app_builder = tauri::Builder::default()
        // 注册基础插件
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        // 注册全局快捷键插件
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    global_shortcuts_handle(app, shortcut, event)
                })
                .build(),
        )
        // 注册开机启动插件
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::AppleScript,
            None,
        ))
        // 注册应用程序初始化函数
        .setup(|app: &mut tauri::App| setup_app(&*app));

    // 注册命令
    app_builder = communication::command::register_commands()(app_builder);

    // 构建应用程序
    let mut app = app_builder.build(context).expect("无法构建 Tauri 应用程序");

    // 在 macOS 上设置为辅助应用
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    // 注册全局快捷键
    if let Err(e) = register_shortcuts(&app) {
        eprintln!("注册全局快捷键失败: {}", e);
    }

    // 在调试模式下启动服务器
    #[cfg(debug_assertions)]
    if let Err(e) = start_server(app.app_handle().clone()) {
        eprintln!("启动开发服务器失败: {}", e);
    }

    // 运行应用程序
    app.run(handle_app_events);
}
