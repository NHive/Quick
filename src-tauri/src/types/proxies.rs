// file_path: src/types/proxies.rs

use serde::{Deserialize, Serialize};

/// 代理信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub id: i32,
    pub r#type: String,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建代理的请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProxyRequest {
    pub r#type: String,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// 更新代理的请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProxyRequest {
    pub id: i32,
    pub r#type: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<Option<String>>,
    pub password: Option<Option<String>>,
}
