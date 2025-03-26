// file_path: src/logic/window_manager/manager.rs
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Runtime};

use super::models::*;
use super::operations::{show_quick_window, switch_to_window};
use super::utils;
use crate::infrastructure::setup;
use crate::logic::events::global_shortcut::SHORTCUT_MANAGER;

// 位置比较的误差容忍度
const POSITION_EPSILON: f64 = 1.0;
// 更新锁定默认时长(毫秒)
const DEFAULT_UPDATE_LOCK_DURATION: u64 = 50;

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
        update_lock_duration: DEFAULT_UPDATE_LOCK_DURATION,
    })
});

// 窗口位置相关辅助函数
fn is_position_similar(p1: &WindowPosition, p2: &WindowPosition) -> bool {
    (p1.x - p2.x).abs() < POSITION_EPSILON
        && (p1.y - p2.y).abs() < POSITION_EPSILON
        && (p1.width - p2.width).abs() < POSITION_EPSILON
        && (p1.height - p2.height).abs() < POSITION_EPSILON
}

// 公共API封装
pub struct WindowManager;

impl WindowManager {
    /// 获取所有窗口信息
    pub fn get_windows() -> Vec<WindowInfo> {
        let windows_store = WINDOWS.lock();
        windows_store.windows.values().cloned().collect()
    }

    /// 获取快速窗口共享位置
    pub fn get_quick_common_position() -> Option<WindowPosition> {
        let position_state = POSITION.lock();
        position_state.quick_common_position.clone()
    }

    /// 更新快速窗口共享位置
    pub fn update_quick_common_position(position: WindowPosition) -> bool {
        let mut position_state = POSITION.lock();

        // 检查位置是否有显著变化
        if let Some(existing) = &position_state.quick_common_position {
            if is_position_similar(existing, &position) {
                debug!("快速窗口位置变化不显著，忽略更新");
                return false;
            }
        }

        // 更新共享位置
        debug!("更新快速窗口共享位置: {:?}", position);
        position_state.quick_common_position = Some(position);
        true
    }

    /// 获取特定窗口的信息
    pub fn get_window_info(label: &str) -> Option<WindowInfo> {
        let windows_store = WINDOWS.lock();
        windows_store.windows.get(label).cloned()
    }

    /// 获取当前激活窗口的信息
    pub fn get_active_window() -> Option<WindowInfo> {
        let active_state = ACTIVE.lock();
        let label = active_state.active_window.as_ref()?.clone();

        // 释放第一个锁再获取第二个锁，避免死锁
        drop(active_state);

        let windows_store = WINDOWS.lock();
        windows_store.windows.get(&label).cloned()
    }

    /// 获取前一个激活窗口的信息
    pub fn get_previous_active_window() -> Option<WindowInfo> {
        let active_state = ACTIVE.lock();
        let label = active_state.previous_active_window.as_ref()?.clone();

        // 释放第一个锁再获取第二个锁，避免死锁
        drop(active_state);

        let windows_store = WINDOWS.lock();
        windows_store.windows.get(&label).cloned()
    }

    /// 获取默认窗口的信息
    pub fn get_default_window() -> Option<WindowInfo> {
        let windows_store = WINDOWS.lock();

        // 查找标记为默认的窗口
        for window in windows_store.windows.values() {
            if window.is_default {
                return Some(window.clone());
            }
        }

        // 如果没有找到默认窗口，返回None
        None
    }

    /// 设置窗口配置列表
    /// 将配置转换为WindowInfo并保存到windows哈希表中
    pub fn set_window_configs<R: Runtime>(app: &AppHandle<R>, configs: Vec<WindowConfig>) {
        // TODO: 暂时将快捷键注册放在这里

        let mut windows_store = WINDOWS.lock();
        let mut active_state = ACTIVE.lock();

        // 跟踪更新的窗口标签
        let mut updated_labels = Vec::with_capacity(configs.len());

        // 为每个配置创建一个WindowInfo
        for config in configs.iter() {
            // 生成窗口标签
            let label = utils::generate_window_label(&config.url, &config.title);
            updated_labels.push(label.clone());

            if let Some(shortcut) = &config.shortcut {
                // 注册窗口快捷键
                let app_handle_clone = app.clone();
                let label_clone = label.clone();
                log::info!("注册窗口快捷键: {} => {:?}", label, shortcut);
                match SHORTCUT_MANAGER
                    .write()
                    .unwrap()
                    .register_shortcut_from_string(&label, shortcut, &label, move |_| {
                        let window_label = label_clone.clone();
                        let app_handle = app_handle_clone.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = switch_to_window(&app_handle, &window_label).await {
                                log::error!("切换窗口错误: {}", e);
                            }
                        });
                    }) {
                    Ok(_) => {
                        debug!("注册窗口快捷键: {} => {:?}", label, shortcut);
                    }
                    Err(e) => {
                        warn!(
                            "注册窗口快捷键失败: {} => {:?}, 错误: {}",
                            label, shortcut, e
                        );
                    }
                }
            }
            // 如果窗口已存在，保留其状态和位置信息
            let window_info = if let Some(existing) = windows_store.windows.get(&label) {
                WindowInfo {
                    label: label.clone(),
                    title: config.title.clone(),
                    url: config.url.clone(),
                    status: existing.status.clone(),
                    loaded: existing.loaded,
                    position: existing.position,
                    icon: config.icon.clone(),
                    shortcut: config.shortcut.clone(),
                    proxy_id: config.proxy_id,
                    is_default: config.is_default,
                }
            } else {
                // 创建新的WindowInfo
                WindowInfo {
                    label: label.clone(),
                    title: config.title.clone(),
                    url: config.url.clone(),
                    status: WindowStatus::Background,
                    loaded: false,
                    position: None,
                    icon: config.icon.clone(),
                    shortcut: config.shortcut.clone(),
                    proxy_id: config.proxy_id,
                    is_default: config.is_default,
                }
            };

            // 保存到哈希表
            windows_store.windows.insert(label, window_info);
        }

        // 确保active_window指向有效窗口
        if let Some(active_label) = &active_state.active_window {
            if !windows_store.windows.contains_key(active_label) {
                // 如果当前激活窗口不存在，设置第一个窗口为激活
                active_state.active_window = updated_labels
                    .first()
                    .cloned()
                    .or_else(|| windows_store.windows.keys().next().cloned());

                debug!("激活窗口不存在，重置为: {:?}", active_state.active_window);
            }
        } else if !windows_store.windows.is_empty() {
            // 如果没有激活窗口但有窗口，设置第一个为激活
            active_state.active_window = updated_labels
                .first()
                .cloned()
                .or_else(|| windows_store.windows.keys().next().cloned());

            debug!("设置初始激活窗口: {:?}", active_state.active_window);
        }
    }

    /// 切换到指定窗口
    pub fn switch_to_window(label: &str) -> bool {
        let mut windows_store = WINDOWS.lock();

        if !windows_store.windows.contains_key(label) {
            warn!("尝试切换到不存在的窗口: {}", label);
            return false;
        }

        let mut active_state = ACTIVE.lock();

        // 如果已经是激活状态，不做任何操作
        if let Some(active_label) = &active_state.active_window {
            if active_label == label {
                debug!("窗口已经是激活状态: {}", label);
                return true;
            }

            // 克隆当前激活窗口标签，避免同时借用
            let active_label_clone = active_label.clone();

            // 保存当前激活窗口为前一个窗口
            active_state.previous_active_window = Some(active_label_clone.clone());

            // 更新前一个激活窗口的状态
            if let Some(active_window) = windows_store.windows.get_mut(&active_label_clone) {
                active_window.status = WindowStatus::Background;
                debug!("将窗口设置为后台: {}", active_label_clone);
            }
        }

        // 更新新激活窗口的状态
        if let Some(window) = windows_store.windows.get_mut(label) {
            window.status = WindowStatus::Foreground;
            window.loaded = true;
            debug!("将窗口设置为前台: {}", label);
        }

        active_state.active_window = Some(label.to_string());
        info!("切换到窗口: {}", label);
        true
    }

    /// 标记窗口已加载完成
    pub fn mark_window_loaded(label: &str) -> bool {
        let mut windows_store = WINDOWS.lock();

        if let Some(window) = windows_store.windows.get_mut(label) {
            if !window.loaded {
                window.loaded = true;
                debug!("标记窗口已加载: {}", label);
                return true;
            }
        } else {
            warn!("尝试标记不存在的窗口为已加载: {}", label);
        }
        false
    }

    /// 更新窗口位置
    pub fn update_window_manager_position(label: &str, position: WindowPosition) -> bool {
        // 获取窗口存储的锁
        let mut windows_store = WINDOWS.lock();

        // 检查窗口是否存在
        if !windows_store.windows.contains_key(label) {
            warn!("尝试更新不存在的窗口位置: {}", label);
            return false;
        }

        let window = windows_store.windows.get_mut(label).unwrap();

        // 检查位置变化是否显著
        if let Some(existing) = &window.position {
            if is_position_similar(existing, &position) {
                return false;
            }
        }

        // 判断是否为快速窗口 (非控制窗口)
        let is_quick_window = label != "control";

        // 更新位置信息
        debug!("更新窗口位置: {} => {:?}", label, position);
        window.position = Some(position.clone());

        // 临时释放锁以避免可能的死锁
        drop(windows_store);

        // 如果是快速窗口，同步更新共享位置
        if is_quick_window {
            WindowManager::update_quick_common_position(position);
        }

        true
    }

    /// 检查是否允许窗口进行更新
    pub fn can_update(label: &str) -> bool {
        let mut update_state = UPDATE.lock();
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(update_state.last_update_time);
        let lock_duration = Duration::from_millis(update_state.update_lock_duration);

        // 如果锁定时间已过或没有正在更新的窗口，则可以更新
        if !update_state.is_updating || elapsed > lock_duration {
            // 重置更新状态
            update_state.is_updating = true;
            update_state.updating_source = Some(label.to_string());
            update_state.last_update_time = current_time;
            debug!("允许窗口更新: {}", label);
            return true;
        }

        // 如果是同一窗口继续更新，允许更新
        if let Some(source) = &update_state.updating_source {
            if source == label {
                update_state.last_update_time = current_time;
                return true;
            }
        }

        debug!(
            "拒绝窗口更新请求: {}, 当前更新窗口: {:?}",
            label, update_state.updating_source
        );
        false
    }

    /// 清除当前激活窗口状态
    pub fn clear_active_window() {
        let mut active_state = ACTIVE.lock();

        // 将当前激活窗口保存为前一个激活窗口
        if let Some(active_label) = &active_state.active_window {
            let label_clone = active_label.clone();
            active_state.previous_active_window = Some(label_clone.clone());
            debug!("设置前一个激活窗口: {}", label_clone);
        }

        // 清除当前激活窗口
        active_state.active_window = None;
        info!("清除当前激活窗口");
    }
}
