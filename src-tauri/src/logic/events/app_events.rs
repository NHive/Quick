// file_path: src/logic/events/app_events.rs
use log::{error, info, warn};
use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, Runtime, WindowEvent};

use crate::logic::window_manager::operations::{
    position_control_window_below_quick, sync_positions_after_control_moved,
};
use crate::logic::window_manager::utils;

use crate::logic::window_manager::focus_state::WindowFocusState;
use crate::logic::window_manager::manager::WindowManager;

// 应用程序事件主处理函数
pub fn handle_app_events<R: Runtime>(app_handle: &AppHandle<R>, event: RunEvent) {
    if let RunEvent::WindowEvent { label, event, .. } = event {
        let app_handle_arc = std::sync::Arc::new(app_handle.clone());

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
    app_handle: &std::sync::Arc<AppHandle<R>>,
    label: &str,
    _position: PhysicalPosition<i32>,
) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    WindowFocusState::update_move_time();
    WindowFocusState::set_dragging(true);

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
                WindowFocusState::set_current_quick_window(Some(label_clone));
            }
            _ => {} // 忽略其他窗口
        }

        // 延迟一段时间后重置拖动状态
        // 这是为了在用户停止拖动后，给一个缓冲时间再接受焦点事件
        tokio::time::sleep(Duration::from_millis(300)).await;

        if WindowFocusState::time_since_last_move() >= Duration::from_millis(200) {
            WindowFocusState::set_dragging(false);
        }
    });
}

// 处理窗口大小调整事件
fn handle_window_resized<R: Runtime>(
    app_handle: &std::sync::Arc<AppHandle<R>>,
    label: &str,
    _size: PhysicalSize<u32>,
) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    // 更新移动时间和拖动状态
    WindowFocusState::update_move_time();

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
fn update_window_manager_position<R: Runtime>(
    app_handle: &std::sync::Arc<AppHandle<R>>,
    label: &str,
) {
    // 使用utils中的函数获取规范化的窗口位置
    match utils::get_window_position_and_size(app_handle, label) {
        Ok(window_position) => {
            // 更新位置信息
            if WindowManager::update_window_manager_position(label, window_position) {
                info!("已更新窗口 {} 的位置信息", label);
            }
        }
        Err(e) => {
            error!("获取窗口 {} 的位置和大小时出错: {}", label, e);
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
    if !WindowFocusState::is_latest_focus_restore_id(focus_restore_id)
        || WindowFocusState::is_dragging()
    {
        info!(
            "跳过焦点恢复 (ID: {})：不是最新请求或正在拖动",
            focus_restore_id
        );
        return;
    }

    if let Some(quick_window_label) = WindowFocusState::get_current_quick_window() {
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
fn handle_window_focused<R: Runtime>(
    app_handle: &std::sync::Arc<AppHandle<R>>,
    label: &str,
    focused: bool,
) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        // 创建焦点恢复ID（用于防抖）
        let focus_restore_id = if label_clone == "control" && focused {
            WindowFocusState::next_focus_restore_id()
        } else {
            0 // 非控制窗口焦点事件不需要ID
        };

        // 如果正在拖动并且是控制窗口获得焦点，跳过处理
        if WindowFocusState::is_dragging() && label_clone == "control" && focused {
            info!("跳过控制窗口焦点处理：正在拖动窗口");
            return;
        }

        // 如果是快速窗口，注册它
        if label_clone.starts_with("quick_") {
            WindowFocusState::register_quick_window(label_clone.clone());
        }

        // 根据窗口类型更新焦点状态
        match label_clone.as_str() {
            "control" => WindowFocusState::set_control_focused(focused),
            label if label.starts_with("quick_") => {
                WindowFocusState::set_quick_window_focused(focused);
                // 如果快速窗口获得焦点，更新当前活动窗口
                if focused {
                    WindowFocusState::set_current_quick_window(Some(label.to_string()));
                }
            }
            _ => {} // 忽略其他窗口
        }

        // 更新隐藏标志
        WindowFocusState::set_hide_pending(!focused);

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
            let should_hide = WindowFocusState::time_since_last_focus_change() >= Duration::from_millis(100)
                && WindowFocusState::is_hide_pending() // 等待隐藏
                && WindowFocusState::all_windows_unfocused() // 所有窗口都失去焦点
                && !WindowFocusState::is_pinned() // 未被固定
                && !WindowFocusState::is_dragging(); // 不在拖动中

            if should_hide {
                // 隐藏所有管理的窗口
                hide_all_managed_windows(&app_handle_clone).await;

                // 重置隐藏等待标志和显示状态
                WindowFocusState::set_hide_pending(false);
                WindowFocusState::set_showing_quick_window(false);
            }
        }
    });
}

// 当所有窗口都失去焦点时隐藏所有管理的窗口
pub async fn hide_all_managed_windows<R: Runtime>(app_handle: &AppHandle<R>) {
    info!("隐藏所有管理的窗口");

    // 隐藏控制窗口
    if let Some(window) = app_handle.get_webview_window("control") {
        if let Err(e) = window.hide() {
            warn!("无法隐藏控制窗口: {}", e);
        }
    }

    // 隐藏当前活动的快速窗口
    if let Some(label) = WindowFocusState::get_current_quick_window() {
        if let Some(window) = app_handle.get_webview_window(&label) {
            if let Err(e) = window.hide() {
                warn!("无法隐藏窗口 {}: {}", label, e);
            }
        }
    }

    // 清除当前活动的快速窗口
    WindowFocusState::set_current_quick_window(None);
    WindowFocusState::set_showing_quick_window(false);
    WindowManager::clear_active_window();
}
