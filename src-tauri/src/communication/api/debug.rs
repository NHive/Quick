// file_path: src/communication/api/debug.rs
use actix_web::web::Query;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use tauri::Manager;

use crate::logic::window_manager::operations::{
    configure_windows, create_or_switch_window, get_active_window, get_all_windows,
    get_previous_active_window, show_previous_window, switch_to_window,
};

use crate::logic::window_manager::models::{WindowConfig, WindowManagerState};
// 添加日志相关的导入
use crate::infrastructure::log::{LogQuery, TempLogger};

use super::AppState;

// 基础健康检查接口
#[get("/gen_204")]
async fn gen_204() -> impl Responder {
    HttpResponse::NoContent()
}

// 用于获取所有窗口信息的接口
#[get("/api/debug/windows")]
async fn get_windows(app_state: web::Data<AppState>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    let windows = get_all_windows(&app_handle);
    HttpResponse::Ok().json(windows)
}

// 获取当前活动窗口信息
#[get("/api/debug/windows/active")]
async fn get_active(app_state: web::Data<AppState>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match get_active_window(&app_handle) {
        Some(window) => HttpResponse::Ok().json(window),
        None => HttpResponse::NotFound().json(json!({"error": "No active window"})),
    }
}

// 获取上一个活动窗口信息
#[get("/api/debug/windows/previous")]
async fn get_previous(app_state: web::Data<AppState>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match get_previous_active_window(&app_handle) {
        Some(window) => HttpResponse::Ok().json(window),
        None => HttpResponse::NotFound().json(json!({"error": "No previous window"})),
    }
}

// 显示之前的窗口
#[post("/api/debug/windows/show_previous")]
async fn show_previous(app_state: web::Data<AppState>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match show_previous_window(&app_handle) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 获取特定窗口信息的请求参数
#[derive(Deserialize)]
struct WindowLabelQuery {
    label: String,
}

// 获取特定窗口信息
#[get("/api/debug/windows/info")]
async fn get_window_info(
    app_state: web::Data<AppState>,
    query: Query<WindowLabelQuery>,
) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    if let Some(window_manager_state) = app_handle.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            match window_manager.get_window_info(&query.label) {
                Some(window) => return HttpResponse::Ok().json(window),
                None => return HttpResponse::NotFound().json(json!({"error": "Window not found"})),
            }
        }
    }

    HttpResponse::InternalServerError().json(json!({"error": "Failed to access window manager"}))
}

// 切换窗口请求参数
#[derive(Deserialize)]
struct SwitchWindowRequest {
    label: String,
}

// 切换到指定窗口
#[post("/api/debug/windows/switch")]
async fn switch_window(
    app_state: web::Data<AppState>,
    req: web::Json<SwitchWindowRequest>,
) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match switch_to_window(&app_handle, &req.label) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 创建窗口请求参数
#[derive(Deserialize)]
struct CreateWindowRequest {
    url: String,
    title: String,
}

// 创建或切换到窗口
#[post("/api/debug/windows/create")]
async fn create_window(
    app_state: web::Data<AppState>,
    req: web::Json<CreateWindowRequest>,
) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match create_or_switch_window(&app_handle, &req.url, &req.title) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 配置窗口请求参数
#[derive(Deserialize)]
struct ConfigureWindowsRequest {
    configs: Vec<WindowConfig>,
}

// 配置窗口列表
#[post("/api/debug/windows/configure")]
async fn configure_window_list(
    app_state: web::Data<AppState>,
    req: web::Json<ConfigureWindowsRequest>,
) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match configure_windows(&app_handle, req.configs.clone()) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 获取窗口管理器当前状态的详细信息
#[get("/api/debug/windows/state")]
async fn get_window_manager_state(app_state: web::Data<AppState>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    if let Some(window_manager_state) = app_handle.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            let response = json!({
                "windows": window_manager.get_windows(),
                "active_window": window_manager.get_active_window(),
                "previous_active_window": window_manager.get_previous_active_window(),
                "control_position": window_manager.get_control_position(),
                "window_configs": window_manager.get_window_configs(),
            });
            return HttpResponse::Ok().json(response);
        }
    }

    HttpResponse::InternalServerError().json(json!({"error": "Failed to access window manager"}))
}

// 查询日志的接口
#[get("/api/debug/logs")]
async fn get_logs(app_state: web::Data<AppState>, query: Query<LogQuery>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    // 尝试获取日志管理器
    match app_handle.try_state::<TempLogger>() {
        Some(logger) => {
            let logs = logger.read_log(&query);
            HttpResponse::Ok().json(logs)
        }
        None => HttpResponse::InternalServerError().json(json!({"error": "Logger not available"})),
    }
}
