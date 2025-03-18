// file_path: src/logic/window_manager/models.rs
// 包含所有窗口管理相关的数据结构定义

use serde::{Deserialize, Serialize};

// =============== 数据结构 ===============

/// 窗口状态枚举
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum WindowStatus {
    /// 前台窗口(当前激活)
    Foreground,
    /// 后台窗口(非激活)
    Background,
}

/// 窗口信息结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowInfo {
    /// 窗口标识符
    pub label: String,
    /// 窗口标题
    pub title: String,
    /// 窗口URL
    pub url: String,
    /// 窗口状态(前台/后台)
    pub status: WindowStatus,
    /// 窗口是否已加载完成
    pub loaded: bool,
    /// 窗口位置和大小信息
    pub position: Option<WindowPosition>,
    /// 图标URL
    pub icon: Option<String>,
    /// 快捷键
    pub shortcut: Option<String>,
    /// 关联的代理ID
    pub proxy_id: Option<i32>,
}

/// 窗口配置结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    /// 窗口标题
    pub title: String,
    /// 窗口URL
    pub url: String,
    /// 图标URL
    pub icon: Option<String>,
    /// 快捷键
    pub shortcut: Option<String>,
    /// 关联的代理ID
    pub proxy_id: Option<i32>,
}

/// 窗口位置和大小信息
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct WindowPosition {
    /// X坐标
    pub x: f64,
    /// Y坐标
    pub y: f64,
    /// 宽度
    pub width: f64,
    /// 高度
    pub height: f64,
}
