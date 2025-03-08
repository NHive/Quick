// file_path: src/communication/api/mod.rs
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

mod debug;
use debug::*;

use crate::infrastructure::error::AppError;

#[derive(Debug, Clone)]
struct AppState {
    app_handle: Arc<Mutex<AppHandle>>,
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(gen_204);
    cfg.service(get_windows);
    cfg.service(get_active);
    cfg.service(get_previous);
    cfg.service(show_previous);
    cfg.service(get_window_info);
    cfg.service(switch_window);
    cfg.service(configure_window_list);
    cfg.service(get_window_manager_state);
    cfg.service(get_logs);
    cfg.service(get_setup);
    cfg.service(set_setup);
    cfg.service(get_all_setups);
    cfg.service(init_setup);
    cfg.service(clear_cache);
    cfg.service(close_window_handler);
    cfg.service(hide_window_handler);
}

pub fn start_server(app_handle: AppHandle) -> Result<(), AppError> {
    tauri::async_runtime::spawn(async_start_server(app_handle));

    Ok(())
}

async fn async_start_server(app_handle: AppHandle) -> Result<(), AppError> {
    let state = AppState {
        app_handle: Arc::new(Mutex::new(app_handle)),
    };
    let server_url = "0.0.0.0:6743";

    log::info!("Starting HTTP server at {}", server_url);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|_origin, _req_head| true)
            .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["*"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .configure(init)
    })
    .bind(server_url)?
    .run();

    server.await?;
    Ok(())
}
