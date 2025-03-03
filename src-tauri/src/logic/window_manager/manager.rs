// 窗口管理器的核心实现

use std::collections::HashMap;
use tauri::AppHandle;
use tauri::Manager;

use super::models::*;

/// 窗口管理器
/// 负责管理所有窗口的状态和信息
pub struct WindowManager {
    /// 存储所有窗口信息的哈希表
    windows: HashMap<String, WindowInfo>,
    /// 当前激活的窗口标签
    active_window: Option<String>,
    /// 前一个激活的窗口标签
    previous_active_window: Option<String>,
    /// 窗口配置列表
    window_configs: Vec<WindowConfig>,
    /// 控制窗口的位置
    control_position: Option<WindowPosition>,
    /// 是否正在更新状态(避免递归更新)
    is_updating: bool,
}

impl WindowManager {
    /// 创建新的窗口管理器实例
    pub fn new() -> Self {
        WindowManager {
            windows: HashMap::new(),
            active_window: None,
            previous_active_window: None,
            window_configs: Vec::new(),
            control_position: None,
            is_updating: false,
        }
    }

    /// 获取所有窗口信息
    pub fn get_windows(&self) -> Vec<WindowInfo> {
        self.windows.values().cloned().collect()
    }

    /// 获取特定窗口的信息
    pub fn get_window_info(&self, label: &str) -> Option<WindowInfo> {
        self.windows.get(label).cloned()
    }

    /// 获取当前激活窗口的信息
    pub fn get_active_window(&self) -> Option<WindowInfo> {
        match &self.active_window {
            Some(label) => self.windows.get(label).cloned(),
            None => None,
        }
    }

    /// 获取前一个激活窗口的信息
    pub fn get_previous_active_window(&self) -> Option<WindowInfo> {
        match &self.previous_active_window {
            Some(label) => self.windows.get(label).cloned(),
            None => None,
        }
    }

    /// 设置窗口配置列表
    pub fn set_window_configs(&mut self, configs: Vec<WindowConfig>) {
        self.window_configs = configs;
    }

    /// 获取窗口配置列表
    pub fn get_window_configs(&self) -> Vec<WindowConfig> {
        self.window_configs.clone()
    }

    /// 添加新窗口
    pub fn add_window(&mut self, label: String, title: String, url: String) {
        // 将当前激活窗口更新为后台状态
        if let Some(active_label) = &self.active_window {
            // 保存为前一个激活窗口
            self.previous_active_window = Some(active_label.clone());

            if let Some(active_window) = self.windows.get_mut(active_label) {
                active_window.status = WindowStatus::Background;
            }
        }

        // 创建新窗口信息
        let window_info = WindowInfo {
            label: label.clone(),
            title,
            url,
            status: WindowStatus::Foreground,
            loaded: false,
            position: None,
        };

        self.windows.insert(label.clone(), window_info);
        self.active_window = Some(label);
    }

    /// 切换到指定窗口
    pub fn switch_to_window(&mut self, label: &str) -> bool {
        if !self.windows.contains_key(label) {
            return false;
        }

        // 如果已经是激活状态，不做任何操作
        if let Some(active_label) = &self.active_window {
            if active_label == label {
                return true;
            }

            // 保存当前激活窗口为前一个窗口
            self.previous_active_window = Some(active_label.clone());

            // 更新前一个激活窗口的状态
            if let Some(active_window) = self.windows.get_mut(active_label) {
                active_window.status = WindowStatus::Background;
            }
        }

        // 更新新激活窗口的状态
        if let Some(window) = self.windows.get_mut(label) {
            window.status = WindowStatus::Foreground;
            window.loaded = true;
        }

        self.active_window = Some(label.to_string());
        true
    }

    /// 标记窗口已加载完成
    pub fn mark_window_loaded(&mut self, label: &str) -> bool {
        if let Some(window) = self.windows.get_mut(label) {
            window.loaded = true;
            return true;
        }
        false
    }

    /// 更新控制窗口位置
    pub fn update_control_position(&mut self, position: WindowPosition) -> bool {
        // 避免递归更新
        if self.is_updating {
            return false;
        }

        // 检查位置是否有显著变化
        if let Some(existing) = &self.control_position {
            if (existing.x - position.x).abs() < 1.0
                && (existing.y - position.y).abs() < 1.0
                && (existing.width - position.width).abs() < 1.0
                && (existing.height - position.height).abs() < 1.0
            {
                return false;
            }
        }

        // 更新控制窗口位置
        self.control_position = Some(position);
        true
    }

    /// 更新窗口位置
    pub fn update_window_position(&mut self, label: &str, position: WindowPosition) -> bool {
        // 避免递归更新
        if self.is_updating {
            return false;
        }

        // 检查位置变化是否显著
        if let Some(window) = self.windows.get_mut(label) {
            if let Some(existing) = &window.position {
                if (existing.x - position.x).abs() < 1.0
                    && (existing.y - position.y).abs() < 1.0
                    && (existing.width - position.width).abs() < 1.0
                    && (existing.height - position.height).abs() < 1.0
                {
                    return false;
                }
            }

            // 更新位置信息
            window.position = Some(position);
            return true;
        }

        false
    }

    /// 获取窗口位置
    pub fn get_window_position(&self, label: &str) -> Option<WindowPosition> {
        if let Some(window) = self.windows.get(label) {
            window.position
        } else {
            None
        }
    }

    /// 获取控制窗口位置
    pub fn get_control_position(&self) -> Option<WindowPosition> {
        self.control_position
    }
}

/// 初始化窗口管理器
pub fn init_window_manager<R: tauri::Runtime>(app: &AppHandle<R>) {
    // 创建窗口管理器实例
    let window_manager = WindowManager::new();
    let window_manager_state = super::models::WindowManagerState(std::sync::Arc::new(
        std::sync::Mutex::new(window_manager),
    ));

    // 注册为应用状态
    app.manage(window_manager_state);
}
