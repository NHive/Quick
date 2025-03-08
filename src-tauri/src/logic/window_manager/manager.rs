// 窗口管理器的核心实现 - 重构版
// 使用更细粒度的锁和 once_cell::sync::Lazy 实现单例

use log::info;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use super::models::*;
use super::utils;

// 窗口存储组件
struct WindowsStore {
    // 存储所有窗口信息的哈希表
    windows: HashMap<String, WindowInfo>,
}

// 激活状态组件
struct ActiveState {
    // 当前激活的窗口标签
    active_window: Option<String>,
    // 前一个激活的窗口标签
    previous_active_window: Option<String>,
}

// 位置状态组件
struct PositionState {
    // 快速窗口的共享位置
    quick_common_position: Option<WindowPosition>,
}

// 更新控制组件
struct UpdateState {
    // 是否正在更新状态(避免递归更新)
    is_updating: bool,
    // 最后一次更新的窗口标签
    updating_source: Option<String>,
    // 最后一次更新的时间戳
    last_update_time: Instant,
    // 更新锁定时间(毫秒)
    update_lock_duration: u64,
}

// 初始化各个状态组件的全局实例
static WINDOWS: Lazy<Mutex<WindowsStore>> = Lazy::new(|| {
    info!("初始化窗口存储组件");
    Mutex::new(WindowsStore {
        windows: HashMap::new(),
    })
});

static ACTIVE: Lazy<Mutex<ActiveState>> = Lazy::new(|| {
    info!("初始化激活状态组件");
    Mutex::new(ActiveState {
        active_window: None,
        previous_active_window: None,
    })
});

static POSITION: Lazy<Mutex<PositionState>> = Lazy::new(|| {
    info!("初始化位置状态组件");
    Mutex::new(PositionState {
        quick_common_position: None,
    })
});

static UPDATE: Lazy<Mutex<UpdateState>> = Lazy::new(|| {
    info!("初始化更新控制组件");
    Mutex::new(UpdateState {
        is_updating: false,
        updating_source: None,
        last_update_time: Instant::now(),
        update_lock_duration: 50,
    })
});

// 公共API封装
pub struct WindowManager;

impl WindowManager {
    /// 获取所有窗口信息
    pub fn get_windows() -> Vec<WindowInfo> {
        if let Ok(windows_store) = WINDOWS.lock() {
            windows_store.windows.values().cloned().collect()
        } else {
            Vec::new() // 默认安全值
        }
    }

    /// 获取快速窗口共享位置
    pub fn get_quick_common_position() -> Option<WindowPosition> {
        if let Ok(position_state) = POSITION.lock() {
            position_state.quick_common_position.clone()
        } else {
            None // 默认安全值
        }
    }

    /// 更新快速窗口共享位置
    pub fn update_quick_common_position(position: WindowPosition) -> bool {
        if let Ok(mut position_state) = POSITION.lock() {
            // 检查位置是否有显著变化
            if let Some(existing) = &position_state.quick_common_position {
                if (existing.x - position.x).abs() < 1.0
                    && (existing.y - position.y).abs() < 1.0
                    && (existing.width - position.width).abs() < 1.0
                    && (existing.height - position.height).abs() < 1.0
                {
                    return false;
                }
            }

            // 更新共享位置
            position_state.quick_common_position = Some(position);
            return true;
        }
        false
    }

    /// 获取特定窗口的信息
    pub fn get_window_info(label: &str) -> Option<WindowInfo> {
        if let Ok(windows_store) = WINDOWS.lock() {
            windows_store.windows.get(label).cloned()
        } else {
            None // 默认安全值
        }
    }

    /// 获取当前激活窗口的信息
    pub fn get_active_window() -> Option<WindowInfo> {
        if let (Ok(active_state), Ok(windows_store)) = (ACTIVE.lock(), WINDOWS.lock()) {
            match &active_state.active_window {
                Some(label) => windows_store.windows.get(label).cloned(),
                None => None,
            }
        } else {
            None // 默认安全值
        }
    }

    /// 获取前一个激活窗口的信息
    pub fn get_previous_active_window() -> Option<WindowInfo> {
        if let (Ok(active_state), Ok(windows_store)) = (ACTIVE.lock(), WINDOWS.lock()) {
            match &active_state.previous_active_window {
                Some(label) => windows_store.windows.get(label).cloned(),
                None => None,
            }
        } else {
            None // 默认安全值
        }
    }

    /// 设置窗口配置列表
    /// 将配置转换为WindowInfo并保存到windows哈希表中
    pub fn set_window_configs(configs: Vec<WindowConfig>) {
        // 获取窗口存储和激活状态的锁
        let windows_result = WINDOWS.lock();
        let active_result = ACTIVE.lock();

        if let (Ok(mut windows_store), Ok(mut active_state)) = (windows_result, active_result) {
            // 为每个配置创建一个WindowInfo
            for config in configs.iter() {
                let label = utils::generate_window_label(&config.url, &config.title);

                // 如果窗口已存在，保留其状态和位置信息
                let status = if let Some(existing) = windows_store.windows.get(&label) {
                    existing.status.clone()
                } else {
                    // 新窗口默认为后台状态
                    WindowStatus::Background
                };

                let position = if let Some(existing) = windows_store.windows.get(&label) {
                    existing.position
                } else {
                    None
                };

                let loaded = if let Some(existing) = windows_store.windows.get(&label) {
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
                windows_store.windows.insert(label, window_info);
            }

            // 确保active_window指向有效窗口
            if let Some(active_label) = &active_state.active_window {
                if !windows_store.windows.contains_key(active_label) {
                    // 如果当前激活窗口不存在，设置第一个窗口为激活
                    active_state.active_window =
                        windows_store.windows.keys().next().map(|k| k.clone());
                }
            } else if !windows_store.windows.is_empty() {
                // 如果没有激活窗口但有窗口，设置第一个为激活
                active_state.active_window = windows_store.windows.keys().next().map(|k| k.clone());
            }
        }
    }

    /// 切换到指定窗口
    pub fn switch_to_window(label: &str) -> bool {
        // 获取窗口存储和激活状态的锁
        let windows_result = WINDOWS.lock();
        let active_result = ACTIVE.lock();

        if let (Ok(mut windows_store), Ok(mut active_state)) = (windows_result, active_result) {
            if !windows_store.windows.contains_key(label) {
                return false;
            }

            // 如果已经是激活状态，不做任何操作
            if let Some(active_label) = &active_state.active_window {
                if active_label == label {
                    return true;
                }

                // 先克隆当前活跃标签，避免借用冲突
                let active_label_clone = active_label.clone();

                // 保存当前激活窗口为前一个窗口
                active_state.previous_active_window = Some(active_label_clone.clone());

                // 更新前一个激活窗口的状态
                if let Some(active_window) = windows_store.windows.get_mut(&active_label_clone) {
                    active_window.status = WindowStatus::Background;
                }
            }

            // 更新新激活窗口的状态
            if let Some(window) = windows_store.windows.get_mut(label) {
                window.status = WindowStatus::Foreground;
                window.loaded = true;
            }

            active_state.active_window = Some(label.to_string());
            return true;
        }

        false
    }

    /// 标记窗口已加载完成
    pub fn mark_window_loaded(label: &str) -> bool {
        if let Ok(mut windows_store) = WINDOWS.lock() {
            if let Some(window) = windows_store.windows.get_mut(label) {
                window.loaded = true;
                return true;
            }
        }
        false
    }

    /// 更新窗口位置
    pub fn update_window_manager_position(label: &str, position: WindowPosition) -> bool {
        // 获取窗口存储的锁
        if let Ok(mut windows_store) = WINDOWS.lock() {
            // 检查位置变化是否显著
            if let Some(window) = windows_store.windows.get_mut(label) {
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
                    drop(windows_store); // 释放窗口存储的锁，避免死锁
                    WindowManager::update_quick_common_position(position);
                }

                return true;
            }
        }

        false
    }

    /// 检查是否允许窗口进行更新
    pub fn can_update(label: &str) -> bool {
        if let Ok(mut update_state) = UPDATE.lock() {
            let current_time = Instant::now();

            // 如果锁定时间已过或没有正在更新的窗口，则可以更新
            if !update_state.is_updating
                || current_time
                    .duration_since(update_state.last_update_time)
                    .as_millis()
                    > update_state.update_lock_duration as u128
            {
                // 重置更新状态
                update_state.is_updating = true;
                update_state.updating_source = Some(label.to_string());
                update_state.last_update_time = current_time;
                return true;
            }

            // 如果是同一窗口继续更新，允许更新
            if let Some(source) = &update_state.updating_source {
                if source == label {
                    update_state.last_update_time = current_time;
                    return true;
                }
            }
        }

        // 其他情况不允许更新
        false
    }

    /// 清除当前激活窗口状态
    pub fn clear_active_window() {
        if let Ok(mut active_state) = ACTIVE.lock() {
            // 将当前激活窗口保存为前一个激活窗口
            if let Some(active_label) = &active_state.active_window {
                active_state.previous_active_window = Some(active_label.clone());
            }
            // 清除当前激活窗口
            active_state.active_window = None;
        }
    }
}
