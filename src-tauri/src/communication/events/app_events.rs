use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, Runtime, WindowEvent};

use crate::logic::tools::window_utils::WindowUtils;
use crate::window::window_layout::{WindowLayoutManager, WindowPositionTrackerState};

pub struct WindowFocusState {
    pub control_focused: Mutex<bool>,
    pub quick_windows_focused: Mutex<bool>,
    quick_window_labels: Mutex<HashSet<String>>,
    last_focus_change: Mutex<Instant>, // 添加焦点切换时间追踪
    hide_pending: Mutex<bool>,         // 添加挂起的隐藏操作标记
}

impl Default for WindowFocusState {
    fn default() -> Self {
        WindowFocusState {
            control_focused: Mutex::new(false),
            quick_windows_focused: Mutex::new(false),
            quick_window_labels: Mutex::new(HashSet::new()),
            last_focus_change: Mutex::new(Instant::now()),
            hide_pending: Mutex::new(false),
        }
    }
}

// 注册 quick 窗口 label
pub fn register_quick_window_label<R: Runtime>(app_handle: &AppHandle<R>, label: String) {
    let Some(focus_state) = app_handle.try_state::<WindowFocusState>() else {
        eprintln!("WindowFocusState not initialized!");
        return;
    };

    let mut labels = focus_state.quick_window_labels.lock().unwrap();
    labels.insert(label);
}

// 处理应用事件
pub fn handle_app_events<R: Runtime>(app_handle: &AppHandle<R>, event: RunEvent) {
    let app_handle_clone = Arc::new(app_handle.clone());
    match event {
        RunEvent::WindowEvent { label, event, .. } => match event {
            WindowEvent::Moved(position) => handle_move_event(&app_handle_clone, &label, position),
            WindowEvent::Resized(size) => handle_resize_event(&app_handle_clone, &label, size),
            WindowEvent::Focused(focused) => handle_focus_event(&app_handle_clone, &label, focused),
            _ => {}
        },
        _ => {}
    }
}

// 处理移动事件
fn handle_move_event<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    position: PhysicalPosition<i32>,
) {
    let app_handle = Arc::clone(app_handle);
    let label = label.to_string();

    std::thread::Builder::new()
        .name("window-moved".into())
        .spawn(move || {
            if label == "control" {
                if let Some(window) = app_handle.get_webview_window("control") {
                    if let Ok(logical_position) =
                        WindowUtils::physical_to_logical(&window, position)
                    {
                        if let Ok(size) = WindowUtils::get_window_size(&app_handle, "control") {
                            if let Some(state) =
                                app_handle.try_state::<WindowPositionTrackerState>()
                            {
                                let mut tracker = match state.0.try_lock() {
                                    Ok(t) => t,
                                    Err(_) => return,
                                };
                                let need_sync = tracker.update_control_position(
                                    logical_position.x,
                                    logical_position.y,
                                    size.width,
                                    size.height,
                                );
                                if need_sync {
                                    let _ = WindowLayoutManager::sync_window_positions(&app_handle);
                                }
                            }
                        }
                    }
                }
            } else if let Some(window) = app_handle.get_webview_window(&label) {
                if let Ok(logical_position) = WindowUtils::physical_to_logical(&window, position) {
                    if let Ok(size) = WindowUtils::get_window_size(&app_handle, &label) {
                        if let Some(state) = app_handle.try_state::<WindowPositionTrackerState>() {
                            let mut tracker = match state.0.try_lock() {
                                Ok(t) => t,
                                Err(_) => return,
                            };
                            tracker.update_window_position(
                                label.clone(),
                                logical_position.x,
                                logical_position.y,
                                size.width,
                                size.height,
                            );
                        }
                    }
                }
            }
        })
        .unwrap();
}

// 处理调整大小事件
fn handle_resize_event<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    size: PhysicalSize<u32>,
) {
    let app_handle = Arc::clone(app_handle);
    let label = label.to_string();

    std::thread::Builder::new()
        .name("window-resized".into())
        .spawn(move || {
            let window = match app_handle.get_webview_window(&label) {
                Some(w) => w,
                None => return,
            };

            if let Ok(logical_size) = WindowUtils::physical_to_logical_size(&window, size) {
                if let Ok(position) = WindowUtils::get_window_position(&app_handle, &label) {
                    if let Some(state) = app_handle.try_state::<WindowPositionTrackerState>() {
                        let mut tracker = match state.0.try_lock() {
                            Ok(t) => t,
                            Err(_) => return,
                        };
                        if label == "control" {
                            tracker.update_control_position(
                                position.x,
                                position.y,
                                logical_size.width,
                                logical_size.height,
                            );
                        } else {
                            tracker.update_window_position(
                                label,
                                position.x,
                                position.y,
                                logical_size.width,
                                logical_size.height,
                            );
                        }
                    }
                }
            }
        })
        .unwrap();
}

// 处理焦点事件
fn handle_focus_event<R: Runtime>(app_handle: &Arc<AppHandle<R>>, label: &str, focused: bool) {
    let app_handle = Arc::clone(app_handle);
    let label = label.to_string();

    std::thread::Builder::new()
        .name("window-focused".into())
        .spawn(move || {
            let Some(focus_state) = app_handle.try_state::<WindowFocusState>() else {
                eprintln!("WindowFocusState not initialized!");
                return;
            };

            // 更新最后焦点变化时间
            {
                let mut last_change = focus_state.last_focus_change.lock().unwrap();
                *last_change = Instant::now();
            }

            // 更新窗口焦点状态
            if label == "control" {
                if let Ok(mut control_focused) = focus_state.control_focused.lock() {
                    *control_focused = focused;
                }
            } else if label.starts_with("quick_") {
                if let Ok(mut quick_focused) = focus_state.quick_windows_focused.lock() {
                    *quick_focused = focused;
                }
            }

            if !focused {
                // 设置延迟隐藏
                {
                    let mut hide_pending = focus_state.hide_pending.lock().unwrap();
                    *hide_pending = true;
                }

                // 创建一个新的应用句柄克隆用于延迟执行
                let app_handle_clone = app_handle.clone();

                // 延迟执行隐藏操作，给窗口切换留出时间
                std::thread::Builder::new()
                    .name("window-focus-delay".into())
                    .spawn(move || {
                        // 等待一个短暂的时间，给焦点切换预留时间
                        std::thread::sleep(Duration::from_millis(150));

                        let Some(focus_state) = app_handle_clone.try_state::<WindowFocusState>()
                        else {
                            return;
                        };

                        // 检查是否有新的焦点事件发生
                        let now = Instant::now();
                        let should_check = {
                            let last_change = focus_state.last_focus_change.lock().unwrap();
                            now.duration_since(*last_change) >= Duration::from_millis(100)
                        };

                        if should_check {
                            let hide_pending = {
                                let pending = focus_state.hide_pending.lock().unwrap();
                                *pending
                            };

                            if hide_pending {
                                let control_focused = match focus_state.control_focused.lock() {
                                    Ok(f) => *f,
                                    Err(_) => return,
                                };
                                let quick_focused = match focus_state.quick_windows_focused.lock() {
                                    Ok(f) => *f,
                                    Err(_) => return,
                                };

                                // 只有在两个窗口都没有焦点的情况下才隐藏
                                if !control_focused && !quick_focused {
                                    // 重置挂起标记
                                    {
                                        let mut pending = focus_state.hide_pending.lock().unwrap();
                                        *pending = false;
                                    }
                                    hide_control_and_quick_windows(&app_handle_clone);
                                }
                            }
                        }
                    })
                    .unwrap();
            } else {
                // 如果窗口获得焦点，取消挂起的隐藏操作
                let mut hide_pending = focus_state.hide_pending.lock().unwrap();
                *hide_pending = false;
            }
        })
        .unwrap();
}

// 隐藏所有相关窗口
fn hide_control_and_quick_windows<R: Runtime>(app_handle: &AppHandle<R>) {
    if let Some(window) = app_handle.get_webview_window("control") {
        let _ = window.hide();
    }

    let Some(focus_state) = app_handle.try_state::<WindowFocusState>() else {
        eprintln!("WindowFocusState not initialized!");
        return;
    };

    let labels = match focus_state.quick_window_labels.lock() {
        Ok(l) => l,
        Err(_) => return,
    };

    for label in labels.iter() {
        if let Some(window) = app_handle.get_webview_window(label) {
            let _ = window.hide();
        }
    }
}
