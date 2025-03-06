// file_path: src/communication/events/app_events.rs
use log::{error, info, warn};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, Runtime, WindowEvent};

use crate::logic::window_manager::operations::{
    position_control_window_below_quick, sync_positions_after_control_moved,
};

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

    // 获取所有快速窗口标签
    pub fn get_quick_window_labels(&self) -> &HashSet<String> {
        &self.quick_window_labels
    }

    // 检查是否有隐藏操作等待执行
    pub fn is_hide_pending(&self) -> bool {
        self.hide_pending
    }

    // 计算自上次焦点变化经过的时间
    pub fn time_since_last_focus_change(&self) -> Duration {
        Instant::now().duration_since(self.last_focus_change)
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

    // 检查是否正在显示快速窗口
    pub fn is_showing_quick_window(&self) -> bool {
        self.is_showing_quick_window
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

// 将焦点还给快速窗口
async fn return_focus_to_quick_window<R: Runtime>(app_handle: &AppHandle<R>) {
    if let Some(focus_state_arc) = app_handle.try_state::<Arc<RwLock<WindowFocusState>>>() {
        let current_quick_window = {
            match focus_state_arc.read() {
                Ok(state) => state.get_current_quick_window().clone(),
                Err(e) => {
                    error!("获取当前快速窗口标签时无法锁定焦点状态: {}", e);
                    return;
                }
            }
        };

        if let Some(quick_window_label) = current_quick_window {
            if let Some(quick_window) = app_handle.get_webview_window(&quick_window_label) {
                info!("将焦点还给快速窗口: {}", quick_window_label);
                if let Err(e) = quick_window.set_focus() {
                    warn!("无法将焦点设置到快速窗口 {}: {}", quick_window_label, e);
                }
            } else {
                warn!("找不到快速窗口: {}", quick_window_label);
            }
        } else {
            info!("没有当前活动的快速窗口，无法还焦点");
        }
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
                    focus_state.set_quick_window_focused(focused)
                }
                _ => {} // 忽略其他窗口
            }

            // 更新隐藏标志
            focus_state.set_hide_pending(!focused);
        }

        // 处理控制窗口获取焦点的情况
        if label_clone == "control" && focused {
            // 延迟100ms后将焦点还给快速窗口
            let app_handle_for_focus = app_handle_clone.clone();
            tokio::time::sleep(Duration::from_millis(100)).await;

            // 再次检查控制窗口是否仍有焦点（防止用户在100ms内已切换焦点）
            let still_focused = {
                match focus_state_arc.read() {
                    Ok(state) => state.control_focused,
                    Err(e) => {
                        error!("检查控制窗口焦点状态时无法锁定焦点状态: {}", e);
                        false
                    }
                }
            };

            if still_focused {
                return_focus_to_quick_window(&app_handle_for_focus).await;
            }
        }

        // 如果窗口失去焦点，延时检查是否应该隐藏所有窗口
        if !focused {
            // 延迟一小段时间再检查，避免焦点切换冲突
            tokio::time::sleep(Duration::from_millis(150)).await;

            // 检查是否所有窗口都失去焦点且等待隐藏，并且未被固定
            let should_hide = {
                match focus_state_arc.write() {
                    Ok(state) => {
                        state.time_since_last_focus_change() >= Duration::from_millis(100)
                            && state.is_hide_pending() // 等待隐藏
                            && state.all_windows_unfocused() // 所有窗口都失去焦点
                            && !state.is_pinned() // 未被固定
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
