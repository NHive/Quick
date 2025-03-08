// file_path: src/logic/window_manager/manager.rs
// 窗口管理器的核心实现

use std::collections::HashMap;
use tauri::AppHandle;
use tauri::Manager;

use super::models::*;
use super::utils;

/// 窗口管理器
/// 负责管理所有窗口的状态和信息
pub struct WindowManager {
    /// 存储所有窗口信息的哈希表
    windows: HashMap<String, WindowInfo>,
    /// 当前激活的窗口标签
    active_window: Option<String>,
    /// 前一个激活的窗口标签
    previous_active_window: Option<String>,
    /// 快速窗口的共享位置
    quick_common_position: Option<WindowPosition>,
    /// 是否正在更新状态(避免递归更新)
    is_updating: bool,
    /// 最后一次更新的窗口标签
    updating_source: Option<String>,
    /// 最后一次更新的时间戳
    last_update_time: std::time::Instant,
    /// 更新锁定时间(毫秒)
    update_lock_duration: u64,
}

impl WindowManager {
    /// 创建新的窗口管理器实例
    pub fn new() -> Self {
        WindowManager {
            windows: HashMap::new(),
            active_window: None,
            previous_active_window: None,
            quick_common_position: None,
            is_updating: false,
            updating_source: None,
            last_update_time: std::time::Instant::now(),
            update_lock_duration: 50, // 默认锁定时间,
        }
    }

    /// 获取所有窗口信息
    pub fn get_windows(&self) -> Vec<WindowInfo> {
        self.windows.values().cloned().collect()
    }

    /// 获取快速窗口共享位置
    pub fn get_quick_common_position(&self) -> Option<WindowPosition> {
        self.quick_common_position
    }

    /// 更新快速窗口共享位置
    pub fn update_quick_common_position(&mut self, position: WindowPosition) -> bool {
        // 检查位置是否有显著变化
        if let Some(existing) = &self.quick_common_position {
            if (existing.x - position.x).abs() < 1.0
                && (existing.y - position.y).abs() < 1.0
                && (existing.width - position.width).abs() < 1.0
                && (existing.height - position.height).abs() < 1.0
            {
                return false;
            }
        }

        // 更新共享位置
        self.quick_common_position = Some(position);
        true
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
    /// 将配置转换为WindowInfo并保存到windows哈希表中
    pub fn set_window_configs(&mut self, configs: Vec<WindowConfig>) {
        // 为每个配置创建一个WindowInfo
        for (_index, config) in configs.iter().enumerate() {
            let label = utils::generate_window_label(&config.url, &config.title);

            // 如果窗口已存在，保留其状态和位置信息
            let status = if let Some(existing) = self.windows.get(&label) {
                existing.status.clone()
            } else {
                // 新窗口默认为后台状态
                WindowStatus::Background
            };

            let position = if let Some(existing) = self.windows.get(&label) {
                existing.position
            } else {
                None
            };

            let loaded = if let Some(existing) = self.windows.get(&label) {
                existing.loaded
            } else {
                false
            };

            // 创建新的WindowInfo
            let window_info = WindowInfo {
                label: label.clone(),
                title: config.title.clone(),
                url: config.url.clone(),
                status,
                loaded,
                position,
                icon: config.icon.clone(),
                shortcut: config.shortcut.clone(),
                proxy_id: config.proxy_id,
            };

            // 保存到哈希表
            self.windows.insert(label, window_info);
        }

        // 确保active_window指向有效窗口
        if let Some(active_label) = &self.active_window {
            if !self.windows.contains_key(active_label) {
                // 如果当前激活窗口不存在，设置第一个窗口为激活
                self.active_window = self.windows.keys().next().map(|k| k.clone());
            }
        } else if !self.windows.is_empty() {
            // 如果没有激活窗口但有窗口，设置第一个为激活
            self.active_window = self.windows.keys().next().map(|k| k.clone());
        }
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

    /// 更新窗口位置
    pub fn update_window_manager_position(
        &mut self,
        label: &str,
        position: WindowPosition,
    ) -> bool {
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

            // 判断是否为快速窗口 (非控制窗口)
            let is_quick_window = label != "control";

            // 更新位置信息
            window.position = Some(position.clone());

            // 如果是快速窗口，同步更新共享位置
            if is_quick_window {
                self.update_quick_common_position(position);
            }

            return true;
        }

        false
    }

    /// 检查是否允许窗口进行更新
    pub fn can_update(&mut self, label: &str) -> bool {
        let current_time = std::time::Instant::now();

        // 如果锁定时间已过或没有正在更新的窗口，则可以更新
        if !self.is_updating
            || current_time
                .duration_since(self.last_update_time)
                .as_millis()
                > self.update_lock_duration as u128
        {
            // 重置更新状态
            self.is_updating = true;
            self.updating_source = Some(label.to_string());
            self.last_update_time = current_time;
            return true;
        }

        // 如果是同一窗口继续更新，允许更新
        if let Some(source) = &self.updating_source {
            if source == label {
                self.last_update_time = current_time;
                return true;
            }
        }

        // 其他情况不允许更新
        false
    }

    /// 清除当前激活窗口状态
    pub fn clear_active_window(&mut self) {
        // 将当前激活窗口保存为前一个激活窗口
        if let Some(active_label) = &self.active_window {
            self.previous_active_window = Some(active_label.clone());
        }
        // 清除当前激活窗口
        self.active_window = None;
    }
}

/// 初始化窗口管理器
pub fn init_window_manager<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建窗口管理器实例
    let window_manager = WindowManager::new();
    let window_manager_state = super::models::WindowManagerState(std::sync::Arc::new(
        std::sync::Mutex::new(window_manager),
    ));

    // 注册为应用状态
    app.manage(window_manager_state);

    Ok(())
}
