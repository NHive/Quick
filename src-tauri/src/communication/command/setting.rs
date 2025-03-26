use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::infrastructure::data_access::{curd_proxies, curd_windows};
use crate::infrastructure::db::DB;
use crate::infrastructure::setup;
use crate::logic::events::global_shortcut::SHORTCUT_MANAGER;
use crate::logic::window_manager::operations::show_quick_window;

// 添加窗口配置
#[tauri::command]
pub async fn cmd_add_window(
    title: String,
    url: String,
    proxy_id: Option<i32>,
    shortcut: Option<String>,
    is_default: Option<bool>,
) -> Result<i32, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    // 设置默认排序顺序为0，icon为None (将由前端或其他逻辑自动生成)
    let result = curd_windows::Insert::create(
        &pool, title, url, None, // icon后续自动生成
        0,    // 默认排序顺序
        proxy_id, shortcut, is_default,
    )
    .await;

    match result {
        Ok(window) => Ok(window.id),
        Err(e) => Err(e.to_string()),
    }
}

// 删除窗口配置
#[tauri::command]
pub async fn cmd_delete_window(id: i32) -> Result<bool, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    match curd_windows::Delete::delete(&pool, id).await {
        Ok(success) => Ok(success),
        Err(e) => Err(e.to_string()),
    }
}

// 修改窗口配置
#[tauri::command]
pub async fn cmd_update_window(
    id: i32,
    title: Option<String>,
    url: Option<String>,
    icon: Option<Option<String>>,
    sort_order: Option<i32>,
    proxy_id: Option<Option<i32>>,
    shortcut: Option<Option<String>>,
    is_default: Option<bool>,
) -> Result<bool, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    match curd_windows::Update::update(
        &pool, id, title, url, icon, sort_order, proxy_id, shortcut, is_default,
    )
    .await
    {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

// 获取所有窗口配置
#[tauri::command]
pub async fn cmd_get_setting_window_configs() -> Result<Vec<serde_json::Value>, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    let windows = match curd_windows::Query::find_all(&pool).await {
        Ok(windows) => windows,
        Err(e) => return Err(e.to_string()),
    };

    // 将窗口数据转换为指定格式的JSON
    let result = windows
        .into_iter()
        .map(|window| {
            serde_json::json!({
                "id": window.id,
                "title": window.title,
                "url": window.url,
                "proxy_id": window.proxy_id,
                "shortcut": window.shortcut,
                "icon": window.icon,
                "sort_order": window.sort_order,
                "is_default": window.is_default
            })
        })
        .collect();

    Ok(result)
}

// 获取所有代理配置
#[tauri::command]
pub async fn cmd_get_setting_proxy_configs() -> Result<Vec<serde_json::Value>, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    let proxies = match curd_proxies::Query::find_all(&pool).await {
        Ok(proxies) => proxies,
        Err(e) => return Err(e.to_string()),
    };

    // 将代理数据转换为指定格式的JSON
    let result = proxies
        .into_iter()
        .map(|proxy| {
            serde_json::json!({
                "id": proxy.id,
                "type": proxy.r#type,
                "host": proxy.host,
                "port": proxy.port,
                "username": proxy.username,
                "password": proxy.password
            })
        })
        .collect();

    Ok(result)
}

// 新建代理
#[tauri::command]
pub async fn cmd_create_proxy(
    proxy_type: String,
    host: String,
    port: i32,
    username: Option<String>,
    password: Option<String>,
    title: Option<String>,
) -> Result<i32, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    match curd_proxies::Insert::create(&pool, proxy_type, host, port, username, password, title)
        .await
    {
        Ok(proxy) => Ok(proxy.id),
        Err(e) => Err(e.to_string()),
    }
}

// 删除代理
#[tauri::command]
pub async fn cmd_delete_proxy(id: i32) -> Result<bool, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    match curd_proxies::Delete::delete(&pool, id).await {
        Ok(success) => Ok(success),
        Err(e) => Err(e.to_string()),
    }
}

// 修改代理
#[tauri::command]
pub async fn cmd_update_proxy(
    id: i32,
    proxy_type: Option<String>,
    host: Option<String>,
    port: Option<i32>,
    username: Option<Option<String>>,
    password: Option<Option<String>>,
) -> Result<bool, String> {
    let pool = match DB::get_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(e.to_string()),
    };

    match curd_proxies::Update::update(&pool, id, proxy_type, host, port, username, password).await
    {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

// 设置默认打开快捷键,以及打开方式
#[tauri::command]
pub async fn cmd_set_default_open_window<R: Runtime>(
    app_handle: AppHandle<R>,
    shortcut: &str,
    open_default: bool,
) -> Result<bool, String> {
    // 设置快捷键
    match setup::set_default_open_window_shortcut_async(Some(shortcut.to_string())).await {
        Ok(_) => {}
        Err(e) => return Err(e.to_string()),
    }

    // 设置打开方式
    let method_result = if open_default {
        setup::set_default_open_window_method_async(Some("default".to_string())).await
    } else {
        setup::set_default_open_window_method_async(Some("last".to_string())).await
    };

    match SHORTCUT_MANAGER
        .write()
        .unwrap()
        .register_shortcut_from_string(
            "open_quick_window",
            &shortcut,
            "打开quick窗口",
            move |_| {
                let app_handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = show_quick_window(&app_handle_clone).await {
                        log::error!("切换窗口错误: {}", e);
                    }
                });
            },
        ) {
        Ok(_) => {
            debug!(
                "注册打开快速窗口快捷键: {} => {:?}",
                "open_quick_window", shortcut
            );
        }
        Err(e) => {
            warn!(
                "注册打开快速窗口快捷键失败: {} => {:?}, 错误: {}",
                "open_quick_window", shortcut, e
            );
        }
    };

    match method_result {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Serialize, Deserialize)]
pub struct DefaultOpenWindow {
    shortcut: String,
    open_default: bool,
}

// 获取默认打开快捷键,以及打开方式
#[tauri::command]
pub async fn cmd_get_default_open_window() -> Result<DefaultOpenWindow, String> {
    let shortcut = setup::get_default_open_window_shortcut_async().await;

    let method = setup::get_default_open_window_method_async().await;

    // 设置默认快捷键为"alt+g"
    let shortcut_value = if shortcut.is_none() || shortcut.as_ref().unwrap().is_empty() {
        "alt+g".to_string()
    } else {
        shortcut.unwrap()
    };

    // 判断打开方式，如果是default或者None(第一次使用)则默认为true
    let open_default = match method.as_deref() {
        Some("default") => true,
        Some("last") => false,
        _ => true, // 默认值为true
    };

    Ok(DefaultOpenWindow {
        shortcut: shortcut_value,
        open_default,
    })
}
