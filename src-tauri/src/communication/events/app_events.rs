// file_path: src/communication/events/app_events.rs
use log::{error, info, warn};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, Runtime, WindowEvent};

use crate::logic::window_manager::operations::{
    position_control_window_below_quick, sync_positions_after_control_moved,
};
use crate::logic::window_manager::utils; // 确保导入utils模块

use crate::logic::window_manager::models::WindowManagerState;

// 窗口焦点状态追踪结构体
pub struct WindowFocusState {
    control_focused: bool,                // 控制窗口是否拥有焦点
    quick_windows_focused: bool,          // 快速窗口是否拥有焦点
    quick_window_labels: HashSet<String>, // 已注册的快速窗口标签集合
    last_focus_change: Instant,           // 上次焦点变化时间点
    hide_pending: bool,                   // 是否等待隐藏
    is_pinned: bool,                      // 窗口是否处于Pin状态
    quick_window_current: Option<String>, // 当前活动的快速窗口标签
    is_showing_quick_window: bool,        // 是否正在显示快速窗口
    control_focus_restore_id: u64,        // 控制窗口焦点防抖ID
    is_dragging: bool,                    // 是否正在拖动窗口
    last_move_time: Instant,              // 上次窗口移动时间
}

impl WindowFocusState {
    pub fn new() -> Self {
        info!("创建新的窗口焦点状态跟踪器");
        WindowFocusState {
            control_focused: false,
            quick_windows_focused: false,
            quick_window_labels: HashSet::new(),
            last_focus_change: Instant::now(),
            hide_pending: false,
            is_pinned: false,
            quick_window_current: None,
            is_showing_quick_window: false,
            control_focus_restore_id: 0,
            is_dragging: false,
            last_move_time: Instant::now(),
        }
    }

    // 注册快速窗口标签
    pub fn register_quick_window(&mut self, label: String) {
        self.quick_window_labels.insert(label);
    }

    // 更新控制窗口的焦点状态
    pub fn set_control_focused(&mut self, focused: bool) {
        self.control_focused = focused;
        self.last_focus_change = Instant::now();
    }

    // 更新快速窗口的焦点状态
    pub fn set_quick_window_focused(&mut self, focused: bool) {
        self.quick_windows_focused = focused;
        self.last_focus_change = Instant::now();
    }

    // 设置隐藏等待标志
    pub fn set_hide_pending(&mut self, pending: bool) {
        self.hide_pending = pending;
    }

    // 检查所有窗口是否都失去焦点
    pub fn all_windows_unfocused(&self) -> bool {
        !self.control_focused && !self.quick_windows_focused
    }

    // 检查是否有隐藏操作等待执行
    pub fn is_hide_pending(&self) -> bool {
        self.hide_pending
    }

    // 计算自上次焦点变化经过的时间
    pub fn time_since_last_focus_change(&self) -> Duration {
        Instant::now().duration_since(self.last_focus_change)
    }

    // 计算自上次窗口移动经过的时间
    pub fn time_since_last_move(&self) -> Duration {
        Instant::now().duration_since(self.last_move_time)
    }

    // 设置窗口是否处于Pin状态
    pub fn set_pinned(&mut self, pinned: bool) {
        self.is_pinned = pinned;
    }

    // 检查窗口是否处于Pin状态
    pub fn is_pinned(&self) -> bool {
        self.is_pinned
    }

    // 设置当前活动的快速窗口标签
    pub fn set_current_quick_window(&mut self, label: Option<String>) {
        self.quick_window_current = label;
    }

    // 获取当前活动的快速窗口标签
    pub fn get_current_quick_window(&self) -> &Option<String> {
        &self.quick_window_current
    }

    // 设置是否正在显示快速窗口
    pub fn set_showing_quick_window(&mut self, showing: bool) {
        self.is_showing_quick_window = showing;
    }

    // 获取并增加焦点恢复ID
    pub fn next_focus_restore_id(&mut self) -> u64 {
        self.control_focus_restore_id += 1;
        self.control_focus_restore_id
    }

    // 检查焦点恢复ID是否匹配
    pub fn is_latest_focus_restore_id(&self, id: u64) -> bool {
        id == self.control_focus_restore_id
    }

    // 设置窗口拖动状态
    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
        if dragging {
            self.last_move_time = Instant::now();
        }
    }

    // 更新窗口移动时间
    pub fn update_move_time(&mut self) {
        self.last_move_time = Instant::now();
    }

    // 检查是否正在拖动窗口
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
}

// 应用程序事件主处理函数
pub fn handle_app_events<R: Runtime>(app_handle: &AppHandle<R>, event: RunEvent) {
    if let RunEvent::WindowEvent { label, event, .. } = event {
        let app_handle_arc = Arc::new(app_handle.clone());

        match event {
            WindowEvent::Moved(position) => {
                handle_window_moved(&app_handle_arc, &label, position);
            }
            WindowEvent::Resized(size) => {
                handle_window_resized(&app_handle_arc, &label, size);
            }
            WindowEvent::Focused(focused) => {
                handle_window_focused(&app_handle_arc, &label, focused);
            }
            _ => {}
        }
    }
}

// 处理窗口移动事件
fn handle_window_moved<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    _position: PhysicalPosition<i32>,
) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    // 更新移动时间和拖动状态
    if let Some(focus_state_arc) = app_handle.try_state::<Arc<RwLock<WindowFocusState>>>() {
        if let Ok(mut focus_state) = focus_state_arc.write() {
            focus_state.update_move_time();
            focus_state.set_dragging(true);
        }
    }

    // 如果是快速窗口，更新WindowManagerState
    if label.starts_with("quick_") {
        update_window_manager_position(app_handle, label);
    }

    tauri::async_runtime::spawn(async move {
        match label_clone.as_str() {
            "control" => {
                // 当控制窗口移动时，同步活动快速窗口的位置
                if let Err(e) = sync_positions_after_control_moved(&app_handle_clone) {
                    warn!("同步控制窗口位置失败: {}", e);
                }
            }
            label if label.starts_with("quick_") => {
                // 当快速窗口移动时，更新控制窗口位置
                if let Err(e) = position_control_window_below_quick(&app_handle_clone, &label_clone)
                {
                    warn!("定位控制窗口失败: {}", e);
                }

                // 更新当前活动的快速窗口
                if let Some(focus_state_arc) =
                    app_handle_clone.try_state::<Arc<RwLock<WindowFocusState>>>()
                {
                    if let Ok(mut focus_state) = focus_state_arc.write() {
                        focus_state.set_current_quick_window(Some(label_clone.clone()));
                    }
                }
            }
            _ => {} // 忽略其他窗口
        }

        // 延迟一段时间后重置拖动状态
        // 这是为了在用户停止拖动后，给一个缓冲时间再接受焦点事件
        tokio::time::sleep(Duration::from_millis(300)).await;

        if let Some(focus_state_arc) = app_handle_clone.try_state::<Arc<RwLock<WindowFocusState>>>()
        {
            if let Ok(mut focus_state) = focus_state_arc.write() {
                // 只有当距离上次移动已经超过200ms才重置拖动状态
                if focus_state.time_since_last_move() >= Duration::from_millis(200) {
                    focus_state.set_dragging(false);
                }
            }
        }
    });
}

// 处理窗口大小调整事件
fn handle_window_resized<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    _size: PhysicalSize<u32>,
) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    // 更新移动时间和拖动状态
    if let Some(focus_state_arc) = app_handle.try_state::<Arc<RwLock<WindowFocusState>>>() {
        if let Ok(mut focus_state) = focus_state_arc.write() {
            focus_state.update_move_time();
        }
    }

    // 如果是快速窗口，更新WindowManagerState
    if label.starts_with("quick_") {
        update_window_manager_position(app_handle, label);
    }

    tauri::async_runtime::spawn(async move {
        match label_clone.as_str() {
            "control" => {
                // 当控制窗口大小改变时，同步活动快速窗口位置
                if let Err(e) = sync_positions_after_control_moved(&app_handle_clone) {
                    warn!("调整大小后同步位置失败: {}", e);
                }
            }
            label if label.starts_with("quick_") => {
                // 当快速窗口大小改变时，更新控制窗口位置
                if let Err(e) = position_control_window_below_quick(&app_handle_clone, &label_clone)
                {
                    warn!("调整大小后更新控制窗口位置失败: {}", e);
                }
            }
            _ => {} // 忽略其他窗口
        }
    });
}

// 更新WindowManagerState中的窗口位置信息 - 使用utils中的函数
fn update_window_manager_position<R: Runtime>(app_handle: &Arc<AppHandle<R>>, label: &str) {
    if let Some(window_manager_state) = app_handle.try_state::<WindowManagerState>() {
        if let Ok(mut window_manager) = window_manager_state.0.lock() {
            // 使用utils中的函数获取规范化的窗口位置
            match utils::get_window_position_and_size(app_handle, label) {
                Ok(window_position) => {
                    // 更新位置信息
                    if window_manager.update_window_manager_position(label, window_position) {
                        info!("已更新窗口 {} 的位置信息", label);
                    }
                }
                Err(e) => {
                    error!("获取窗口 {} 的位置和大小时出错: {}", label, e);
                }
            }
        } else {
            error!("无法锁定窗口管理器状态");
        }
    }
}

// 将焦点还给快速窗口（带防抖功能）
async fn return_focus_to_quick_window<R: Runtime>(
    app_handle: &AppHandle<R>,
    focus_restore_id: u64,
) {
    // 延迟300ms执行（防抖延迟）
    tokio::time::sleep(Duration::from_millis(300)).await;

    // 检查是否是最新的焦点恢复请求和拖动状态
    let (should_restore, current_quick_window) = {
        if let Some(focus_state_arc) = app_handle.try_state::<Arc<RwLock<WindowFocusState>>>() {
            match focus_state_arc.read() {
                Ok(state) => {
                    let is_latest = state.is_latest_focus_restore_id(focus_restore_id);
                    let is_dragging = state.is_dragging();
                    let quick_window = state.get_current_quick_window().clone();

                    // 如果正在拖动或不是最新ID，则不恢复焦点
                    (!is_dragging && is_latest, quick_window)
                }
                Err(e) => {
                    error!("获取焦点状态时出错: {}", e);
                    return;
                }
            }
        } else {
            return;
        }
    };

    if !should_restore {
        info!(
            "跳过焦点恢复 (ID: {})：不是最新请求或正在拖动",
            focus_restore_id
        );
        return;
    }

    if let Some(quick_window_label) = current_quick_window {
        if let Some(quick_window) = app_handle.get_webview_window(&quick_window_label) {
            info!(
                "将焦点还给快速窗口: {} (ID: {})",
                quick_window_label, focus_restore_id
            );
            if let Err(e) = quick_window.set_focus() {
                warn!("无法将焦点设置到快速窗口 {}: {}", quick_window_label, e);
            }
        } else {
            warn!("找不到快速窗口: {}", quick_window_label);
        }
    } else {
        info!(
            "没有当前活动的快速窗口，无法还焦点 (ID: {})",
            focus_restore_id
        );
    }
}

// 处理窗口焦点事件
fn handle_window_focused<R: Runtime>(app_handle: &Arc<AppHandle<R>>, label: &str, focused: bool) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        // 获取焦点状态
        let focus_state_arc = match app_handle_clone.try_state::<Arc<RwLock<WindowFocusState>>>() {
            Some(state) => state,
            None => {
                error!("无法获取窗口焦点状态");
                return;
            }
        };

        // 创建焦点恢复ID（用于防抖）
        let focus_restore_id = if label_clone == "control" && focused {
            match focus_state_arc.write() {
                Ok(mut state) => state.next_focus_restore_id(),
                Err(e) => {
                    error!("生成焦点恢复ID时出错: {}", e);
                    return;
                }
            }
        } else {
            0 // 非控制窗口焦点事件不需要ID
        };

        // 是否正在拖动
        let is_dragging = {
            match focus_state_arc.read() {
                Ok(state) => state.is_dragging(),
                Err(e) => {
                    error!("检查拖动状态时出错: {}", e);
                    false
                }
            }
        };

        // 如果正在拖动并且是控制窗口获得焦点，跳过处理
        if is_dragging && label_clone == "control" && focused {
            info!("跳过控制窗口焦点处理：正在拖动窗口");
            return;
        }

        // 更新焦点状态
        {
            let mut focus_state = match focus_state_arc.write() {
                Ok(state) => state,
                Err(e) => {
                    error!("无法锁定焦点状态: {}", e);
                    return;
                }
            };

            // 如果是快速窗口，注册它
            if label_clone.starts_with("quick_") {
                focus_state.register_quick_window(label_clone.clone());
            }

            // 根据窗口类型更新焦点状态
            match label_clone.as_str() {
                "control" => focus_state.set_control_focused(focused),
                label if label.starts_with("quick_") => {
                    focus_state.set_quick_window_focused(focused);
                    // 如果快速窗口获得焦点，更新当前活动窗口
                    if focused {
                        focus_state.set_current_quick_window(Some(label.to_string()));
                    }
                }
                _ => {} // 忽略其他窗口
            }

            // 更新隐藏标志
            focus_state.set_hide_pending(!focused);
        }

        // 处理控制窗口获取焦点的情况
        if label_clone == "control" && focused {
            // 使用防抖机制还原焦点
            return_focus_to_quick_window(&app_handle_clone, focus_restore_id).await;
        }

        // 如果窗口失去焦点，延时检查是否应该隐藏所有窗口
        if !focused {
            // 延迟一小段时间再检查，避免焦点切换冲突
            tokio::time::sleep(Duration::from_millis(150)).await;

            // 检查是否所有窗口都失去焦点且等待隐藏，并且未被固定且不在拖动中
            let should_hide = {
                match focus_state_arc.write() {
                    Ok(state) => {
                        state.time_since_last_focus_change() >= Duration::from_millis(100)
                            && state.is_hide_pending() // 等待隐藏
                            && state.all_windows_unfocused() // 所有窗口都失去焦点
                            && !state.is_pinned() // 未被固定
                            && !state.is_dragging() // 不在拖动中
                    }
                    Err(e) => {
                        error!("检查隐藏条件时无法锁定焦点状态: {}", e);
                        false
                    }
                }
            };

            if should_hide {
                // 隐藏所有管理的窗口
                hide_all_managed_windows(&app_handle_clone).await;

                // 重置隐藏等待标志和显示状态
                if let Ok(mut focus_state) = focus_state_arc.write() {
                    focus_state.set_hide_pending(false);
                    focus_state.set_showing_quick_window(false);
                }
            }
        }
    });
}

// 当所有窗口都失去焦点时隐藏所有管理的窗口
async fn hide_all_managed_windows<R: Runtime>(app_handle: &AppHandle<R>) {
    info!("隐藏所有管理的窗口");

    // 隐藏控制窗口
    if let Some(window) = app_handle.get_webview_window("control") {
        if let Err(e) = window.hide() {
            warn!("无法隐藏控制窗口: {}", e);
        }
    }

    // 隐藏所有快速窗口
    if let Some(focus_state_arc) = app_handle.try_state::<Arc<RwLock<WindowFocusState>>>() {
        let quick_window_labels = {
            match focus_state_arc.read() {
                Ok(state) => state.get_current_quick_window().clone(),
                Err(e) => {
                    error!("获取快速窗口标签时无法锁定焦点状态: {}", e);
                    return;
                }
            }
        };

        if let Some(label) = quick_window_labels {
            if let Some(window) = app_handle.get_webview_window(&label) {
                if let Err(e) = window.hide() {
                    warn!("无法隐藏窗口 {}: {}", label, e);
                }
            }
        }

        // 清除当前活动的快速窗口
        if let Ok(mut focus_state) = focus_state_arc.write() {
            focus_state.set_current_quick_window(None);
            focus_state.set_showing_quick_window(false);
        }
    }

    // 清除活动窗口状态
    if let Some(window_manager_state) = app_handle.try_state::<WindowManagerState>() {
        if let Ok(mut window_manager) = window_manager_state.0.lock() {
            window_manager.clear_active_window();
        } else {
            error!("无法锁定窗口管理器以清除活动窗口状态");
        }
    }
}
