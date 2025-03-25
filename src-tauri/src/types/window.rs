// file_path: src/types/window.rs

use serde::{Deserialize, Serialize};

/// 窗口信息
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub proxy_id: Option<i32>,
    pub shortcut: Option<String>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建窗口的请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWindowRequest {
    pub title: String,
    pub url: String,
    pub icon: Option<String>,
    pub proxy_id: Option<i32>,
    pub shortcut: Option<String>,
    pub is_default: Option<bool>,
}

/// 更新窗口的请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWindowRequest {
    pub id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub icon: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub proxy_id: Option<Option<i32>>,
    pub shortcut: Option<Option<String>>,
    pub is_default: Option<bool>,
}
