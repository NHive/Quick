// file_path: src/communication/api/debug.rs
use actix_web::web::Query;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use tauri::Manager;

use crate::logic::window_manager::manager::WindowManager;
use crate::logic::window_manager::operations::{
    clear_window_cache, close_window, hide_window, load_window_configs_from_db,
    show_previous_window, switch_to_window,
};

// 添加日志相关的导入
use crate::infrastructure::log::{LogQuery, TempLogger};

use crate::infrastructure::setup::SetupService;
use serde_json::Value;

use super::AppState;

// 基础健康检查接口
#[get("/gen_204")]
async fn gen_204() -> impl Responder {
    HttpResponse::NoContent()
}

// 用于获取所有窗口信息的接口
#[get("/api/debug/windows")]
async fn get_windows() -> impl Responder {
    let windows = WindowManager::get_windows();
    HttpResponse::Ok().json(windows)
}

// 获取当前活动窗口信息
#[get("/api/debug/windows/active")]
async fn get_active() -> impl Responder {
    match WindowManager::get_active_window() {
        Some(window) => HttpResponse::Ok().json(window),
        None => HttpResponse::NotFound().json(json!({"error": "No active window"})),
    }
}

// 获取上一个活动窗口信息
#[get("/api/debug/windows/previous")]
async fn get_previous() -> impl Responder {
    match WindowManager::get_previous_active_window() {
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

    match show_previous_window(&app_handle).await {
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
async fn get_window_info(query: Query<WindowLabelQuery>) -> impl Responder {
    match WindowManager::get_window_info(&query.label) {
        Some(window) => return HttpResponse::Ok().json(window),
        None => return HttpResponse::NotFound().json(json!({"error": "Window not found"})),
    }
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

    match switch_to_window(&app_handle, &req.label).await {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 加载窗口配置
#[post("/api/debug/windows/load_configs")]
async fn configure_window_list() -> impl Responder {
    match load_window_configs_from_db().await {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 获取窗口管理器当前状态的详细信息
#[get("/api/debug/windows/state")]
async fn get_window_manager_state() -> impl Responder {
    let response = json!({
        "windows": WindowManager::get_windows(),
        "active_window": WindowManager::get_active_window(),
        "previous_active_window": WindowManager::get_previous_active_window(),
        "quick_common_position": WindowManager::get_quick_common_position(),
    });
    return HttpResponse::Ok().json(response);
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

// 获取特定设置的请求参数
#[derive(Deserialize)]
struct SetupKeyQuery {
    key: String,
}

// 获取特定配置的接口
#[get("/api/debug/setup")]
async fn get_setup(app_state: web::Data<AppState>, query: Query<SetupKeyQuery>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match app_handle.try_state::<SetupService>() {
        Some(setup_service) => match setup_service.get_setup_async(&query.key).await {
            Ok(value) => HttpResponse::Ok().json(value),
            Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        },
        None => HttpResponse::InternalServerError()
            .json(json!({"error": "Setup service not available"})),
    }
}

// 设置配置请求参数
#[derive(Deserialize)]
struct SetupRequest {
    key: String,
    value: Value,
}

// 设置配置的接口
#[post("/api/debug/setup")]
async fn set_setup(app_state: web::Data<AppState>, req: web::Json<SetupRequest>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match app_handle.try_state::<SetupService>() {
        Some(setup_service) => match setup_service.set_setup_async(&req.key, &req.value).await {
            Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
            Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        },
        None => HttpResponse::InternalServerError()
            .json(json!({"error": "Setup service not available"})),
    }
}

// 获取所有配置的接口
#[get("/api/debug/setup/all")]
async fn get_all_setups(app_state: web::Data<AppState>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match app_handle.try_state::<SetupService>() {
        Some(setup_service) => match setup_service.get_all_setups_async().await {
            Ok(values) => HttpResponse::Ok().json(values),
            Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        },
        None => HttpResponse::InternalServerError()
            .json(json!({"error": "Setup service not available"})),
    }
}

// 初始化设置请求参数
#[derive(Deserialize)]
struct InitSetupRequest {
    defaults: std::collections::HashMap<String, Value>,
}

// 初始化设置的接口
#[post("/api/debug/setup/init")]
async fn init_setup(
    app_state: web::Data<AppState>,
    req: web::Json<InitSetupRequest>,
) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match app_handle.try_state::<SetupService>() {
        Some(setup_service) => match setup_service.init_setup_async(req.defaults.clone()).await {
            Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
            Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        },
        None => HttpResponse::InternalServerError()
            .json(json!({"error": "Setup service not available"})),
    }
}

// 清除指定窗口的缓存
#[post("/api/debug/windows/clear_cache")]
async fn clear_cache(app_state: web::Data<AppState>) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match clear_window_cache(&app_handle).await {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true, "message": "窗口缓存已清除"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 关闭窗口请求参数
#[derive(Deserialize)]
struct CloseWindowRequest {
    label: String,
}

// 关闭指定窗口的接口
#[post("/api/debug/windows/close")]
async fn close_window_handler(
    app_state: web::Data<AppState>,
    req: web::Json<CloseWindowRequest>,
) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match close_window(&app_handle, &req.label) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true, "message": "窗口已关闭"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

// 隐藏窗口请求参数
#[derive(Deserialize)]
struct HideWindowRequest {
    label: String,
}

// 隐藏指定窗口的接口
#[post("/api/debug/windows/hide")]
async fn hide_window_handler(
    app_state: web::Data<AppState>,
    req: web::Json<HideWindowRequest>,
) -> impl Responder {
    let app_handle = match app_state.app_handle.lock() {
        Ok(handle) => handle.clone(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to lock app handle"}))
        }
    };

    match hide_window(&app_handle, &req.label) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true, "message": "窗口已隐藏"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
