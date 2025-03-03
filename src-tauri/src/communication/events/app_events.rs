// file_path: src/communication/events/app_events.rs
use log::{error, info};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, Runtime, WindowEvent};
use tokio::sync::mpsc;

use crate::window::window_manager::{
    get_window_position_and_size, WindowManagerMessage, WindowManagerState,
};

// 跟踪所有窗口的焦点状态
pub struct WindowFocusState {
    control_focused: Mutex<bool>,
    quick_windows_focused: Mutex<bool>,
    quick_window_labels: Mutex<HashSet<String>>,
    last_focus_change: Mutex<Instant>,
    hide_pending: Mutex<bool>,
    tx: mpsc::Sender<WindowManagerMessage>,
}

impl WindowFocusState {
    pub fn new(tx: mpsc::Sender<WindowManagerMessage>) -> Self {
        info!("创建新的 WindowFocusState");
        WindowFocusState {
            control_focused: Mutex::new(false),
            quick_windows_focused: Mutex::new(false),
            quick_window_labels: Mutex::new(HashSet::new()),
            last_focus_change: Mutex::new(Instant::now()),
            hide_pending: Mutex::new(false),
            tx,
        }
    }
}

// 注册一个新的快速窗口标签以进行跟踪
pub fn register_quick_window_label<R: Runtime>(app_handle: &AppHandle<R>, label: String) {
    info!("注册快速窗口标签: {}", label);
    if let Some(focus_state) = app_handle.try_state::<WindowFocusState>() {
        if let Ok(mut labels) = focus_state.quick_window_labels.lock() {
            labels.insert(label);
        }
    }
}

// 应用程序事件的主事件处理器
pub fn handle_app_events<R: Runtime>(app_handle: &AppHandle<R>, event: RunEvent) {
    match event {
        RunEvent::WindowEvent { label, event, .. } => {
            info!("处理窗口事件，标签: {:?}", label);
            info!("事件详情: {:?}", event);

            let app_handle_arc = Arc::new(app_handle.clone());

            match event {
                WindowEvent::Moved(position) => {
                    info!("窗口移动: {:?}", position);
                    handle_window_moved(&app_handle_arc, &label, position);
                }
                WindowEvent::Resized(size) => {
                    info!("窗口调整大小: {:?}", size);
                    handle_window_resized(&app_handle_arc, &label, size);
                }
                WindowEvent::Focused(focused) => {
                    info!("窗口焦点: {}", focused);
                    handle_window_focused(&app_handle_arc, &label, focused);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

// 初始化事件系统和状态
pub fn init_event_system<R: Runtime>(
    app_handle: &AppHandle<R>,
    tx: mpsc::Sender<WindowManagerMessage>,
) {
    info!("初始化事件系统");
    // 创建并注册焦点状态
    let focus_state = WindowFocusState::new(tx);
    app_handle.manage(focus_state);
}

// 处理窗口移动事件
fn handle_window_moved<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    position: PhysicalPosition<i32>,
) {
    info!("处理窗口移动，标签: {}", label);
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        if label_clone == "control" {
            info!("处理控制窗口移动");
            if let Ok(position) = get_window_position_and_size(&app_handle_clone, "control") {
                let tx_clone = {
                    if let Some(manager_state) = app_handle_clone.try_state::<WindowManagerState>()
                    {
                        let guard = manager_state.0.lock().unwrap();
                        guard.tx.clone()
                    } else {
                        error!("获取 WindowManagerState 失败");
                        return;
                    }
                };

                let _ = tx_clone
                    .send(WindowManagerMessage::UpdatePosition {
                        label: "control".to_string(),
                        position,
                    })
                    .await;

                let _ = tx_clone.send(WindowManagerMessage::SyncPositions).await;
            }
        } else {
            info!("处理其他窗口移动，标签: {}", label_clone);
            if let Ok(position) = get_window_position_and_size(&app_handle_clone, &label_clone) {
                let tx_clone = {
                    if let Some(manager_state) = app_handle_clone.try_state::<WindowManagerState>()
                    {
                        let guard = manager_state.0.lock().unwrap();
                        guard.tx.clone()
                    } else {
                        error!("获取 WindowManagerState 失败");
                        return;
                    }
                };

                let _ = tx_clone
                    .send(WindowManagerMessage::UpdatePosition {
                        label: label_clone,
                        position,
                    })
                    .await;
            }
        }
    });
}

// 处理窗口调整大小事件
fn handle_window_resized<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    size: PhysicalSize<u32>,
) {
    info!("处理窗口调整大小，标签: {}", label);
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        if let Ok(position) = get_window_position_and_size(&app_handle_clone, &label_clone) {
            let tx_clone = {
                if let Some(manager_state) = app_handle_clone.try_state::<WindowManagerState>() {
                    let guard = manager_state.0.lock().unwrap();
                    guard.tx.clone()
                } else {
                    error!("获取 WindowManagerState 失败");
                    return;
                }
            };

            let _ = tx_clone
                .send(WindowManagerMessage::UpdatePosition {
                    label: label_clone.clone(),
                    position,
                })
                .await;

            if label_clone == "control" {
                let _ = tx_clone.send(WindowManagerMessage::SyncPositions).await;
            }
        }
    });
}

// 处理窗口焦点事件
fn handle_window_focused<R: Runtime>(app_handle: &Arc<AppHandle<R>>, label: &str, focused: bool) {
    info!("处理窗口焦点，标签: {}, 焦点: {}", label, focused);
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        if let Some(focus_state) = app_handle_clone.try_state::<WindowFocusState>() {
            if let Ok(mut last_change) = focus_state.last_focus_change.lock() {
                *last_change = Instant::now();
            }

            if label_clone == "control" {
                if let Ok(mut control_focused) = focus_state.control_focused.lock() {
                    *control_focused = focused;
                }
            } else if label_clone.starts_with("quick_") {
                if let Ok(mut quick_focused) = focus_state.quick_windows_focused.lock() {
                    *quick_focused = focused;
                }
            }

            if !focused {
                if let Ok(mut hide_pending) = focus_state.hide_pending.lock() {
                    *hide_pending = true;
                }

                let app_handle_async = app_handle_clone.clone();

                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(150)).await;

                    if let Some(focus_state) = app_handle_async.try_state::<WindowFocusState>() {
                        let now = Instant::now();
                        let should_check = {
                            if let Ok(last_change) = focus_state.last_focus_change.lock() {
                                now.duration_since(*last_change) >= Duration::from_millis(100)
                            } else {
                                false
                            }
                        };

                        if should_check {
                            let hide_pending = match focus_state.hide_pending.lock() {
                                Ok(pending) => *pending,
                                Err(_) => false,
                            };

                            if hide_pending {
                                let control_focused = match focus_state.control_focused.lock() {
                                    Ok(f) => *f,
                                    Err(_) => false,
                                };

                                let quick_focused = match focus_state.quick_windows_focused.lock() {
                                    Ok(f) => *f,
                                    Err(_) => false,
                                };

                                if !control_focused && !quick_focused {
                                    if let Ok(mut pending) = focus_state.hide_pending.lock() {
                                        *pending = false;
                                    }

                                    hide_all_managed_windows(&app_handle_async).await;
                                }
                            }
                        }
                    }
                });
            } else {
                if let Ok(mut hide_pending) = focus_state.hide_pending.lock() {
                    *hide_pending = false;
                }
            }
        }
    });
}

// 当没有焦点时隐藏控制和快速窗口
async fn hide_all_managed_windows<R: Runtime>(app_handle: &AppHandle<R>) {
    info!("隐藏所有管理的窗口");
    if let Some(window) = app_handle.get_webview_window("control") {
        let _ = window.hide();
    }

    if let Some(focus_state) = app_handle.try_state::<WindowFocusState>() {
        if let Ok(labels) = focus_state.quick_window_labels.lock() {
            for label in labels.iter() {
                if let Some(window) = app_handle.get_webview_window(label) {
                    let _ = window.hide();
                }
            }
        }
    }
}

// 函数用于将焦点窗口置于前台或显示
pub fn show_or_focus_window<R: Runtime>(
    app_handle: &AppHandle<R>,
    label: &str,
) -> Result<(), tauri::Error> {
    info!("显示或聚焦窗口: {}", label);
    if let Some(window) = app_handle.get_webview_window(label) {
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

// 函数用于切换窗口的可见性
pub fn toggle_window_visibility<R: Runtime>(
    app_handle: &AppHandle<R>,
    label: &str,
) -> Result<(), tauri::Error> {
    info!("切换窗口可见性: {}", label);
    if let Some(window) = app_handle.get_webview_window(label) {
        if window.is_visible()? {
            window.hide()?;
        } else {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}

// 检查是否有任何管理的窗口处于焦点
pub fn is_any_window_focused<R: Runtime>(app_handle: &AppHandle<R>) -> bool {
    info!("检查是否有窗口处于焦点");
    if let Some(focus_state) = app_handle.try_state::<WindowFocusState>() {
        let control_focused = match focus_state.control_focused.lock() {
            Ok(f) => *f,
            Err(_) => false,
        };

        let quick_focused = match focus_state.quick_windows_focused.lock() {
            Ok(f) => *f,
            Err(_) => false,
        };

        return control_focused || quick_focused;
    }

    false
}
