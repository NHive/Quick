use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};

use crate::logic::tools::window_utils::WindowUtils;

// 导入之前的窗口管理器
use crate::window::quick_window::WindowManagerState;

// 窗口位置数据结构
#[derive(Debug, Clone, Copy)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// 窗口位置跟踪器
pub struct WindowPositionTracker {
    // 控制窗口位置
    pub control_window_position: WindowPosition,
    // quick窗口位置,存储每个窗口的完整位置信息
    pub window_positions: HashMap<String, WindowPosition>,
    // 是否更新中,防止递归更新
    pub is_updating: bool,
}

impl WindowPositionTracker {
    pub fn new() -> Self {
        WindowPositionTracker {
            control_window_position: WindowPosition {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            window_positions: HashMap::new(),
            is_updating: false,
        }
    }

    // 更新控制窗口位置
    pub fn update_control_position(&mut self, x: f64, y: f64, width: f64, height: f64) -> bool {
        // 如果正在更新中，避免递归更新
        if self.is_updating {
            return false;
        }

        // 防止相同位置的重复更新
        if (self.control_window_position.x - x).abs() < 1.0
            && (self.control_window_position.y - y).abs() < 1.0
            && (self.control_window_position.width - width).abs() < 1.0
            && (self.control_window_position.height - height).abs() < 1.0
        {
            return false;
        }

        let old_x = self.control_window_position.x;
        let old_y = self.control_window_position.y;

        self.control_window_position = WindowPosition {
            x,
            y,
            width,
            height,
        };

        // 计算位移量
        let delta_x = x - old_x;
        let delta_y = y - old_y;

        // 无位移时不更新其他窗口
        if delta_x.abs() < 1.0 && delta_y.abs() < 1.0 {
            return false;
        }

        // 更新所有窗口位置（跟随控制窗口移动）
        for (_, pos) in self.window_positions.iter_mut() {
            pos.x += delta_x;
            pos.y += delta_y;
        }

        true
    }

    // 更新窗口位置
    pub fn update_window_position(
        &mut self,
        label: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> bool {
        // 如果正在更新中，避免递归更新
        if self.is_updating {
            return false;
        }

        // 检查位置是否有显著变化
        if let Some(existing) = self.window_positions.get(&label) {
            if (existing.x - x).abs() < 1.0
                && (existing.y - y).abs() < 1.0
                && (existing.width - width).abs() < 1.0
                && (existing.height - height).abs() < 1.0
            {
                return false;
            }
        }

        self.window_positions.insert(
            label,
            WindowPosition {
                x,
                y,
                width,
                height,
            },
        );

        true
    }

    // 获取窗口位置
    pub fn get_window_position(&self, label: &str) -> Option<&WindowPosition> {
        self.window_positions.get(label)
    }

    // 移除窗口
    pub fn remove_window(&mut self, label: &str) {
        self.window_positions.remove(label);
    }
}

// 全局位置跟踪器状态
pub struct WindowPositionTrackerState(pub Arc<Mutex<WindowPositionTracker>>);

// 窗口布局管理器
pub struct WindowLayoutManager;

impl WindowLayoutManager {
    // 将网页窗口放置在控制窗口的左侧
    pub fn position_window_left_of_control<R: Runtime>(
        app_handle: &AppHandle<R>,
        window_label: &str,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<(), tauri::Error> {
        // 获取控制窗口
        let control_window = match app_handle.get_webview_window("control") {
            Some(window) => window,
            None => return Ok(()),
        };

        // 获取控制窗口的位置和尺寸
        let control_position = WindowUtils::get_window_position(app_handle, "control")?;
        let control_size = WindowUtils::get_window_size(app_handle, "control")?;

        // 获取目标窗口
        let window = match app_handle.get_webview_window(window_label) {
            Some(window) => window,
            None => return Ok(()),
        };

        // 获取目标窗口尺寸
        let window_size = WindowUtils::get_window_size(app_handle, window_label)?;

        // 计算新位置 - 在控制窗口左侧
        let window_x = control_position.x - window_size.width - offset_x;
        let window_y = control_position.y + offset_y;

        // 设置窗口位置
        let _ = WindowUtils::set_window_position(&window, window_x, window_y);

        // 更新跟踪器
        if let Some(tracker_state) = app_handle.try_state::<WindowPositionTrackerState>() {
            // 使用 try_lock 而不是 lock 避免死锁
            if let Ok(mut tracker) = tracker_state.0.try_lock() {
                // 打开更新标志
                tracker.is_updating = true;

                // 更新控制窗口位置信息
                tracker.update_control_position(
                    control_position.x,
                    control_position.y,
                    control_size.width,
                    control_size.height,
                );

                // 更新目标窗口位置信息
                tracker.update_window_position(
                    window_label.to_string(),
                    window_x,
                    window_y,
                    window_size.width,
                    window_size.height,
                );

                // 关闭更新标志
                tracker.is_updating = false;
            }
        }

        Ok(())
    }

    // 同步所有窗口位置
    pub fn sync_window_positions<R: Runtime>(
        app_handle: &AppHandle<R>,
    ) -> Result<(), tauri::Error> {
        // 使用 try_state 获取状态
        let tracker_state = match app_handle.try_state::<WindowPositionTrackerState>() {
            Some(state) => state,
            None => return Ok(()),
        };

        // 使用 try_lock 避免死锁
        let tracker = match tracker_state.0.try_lock() {
            Ok(t) => t,
            Err(_) => return Ok(()),
        };

        // 获取窗口管理器
        let window_manager_state = match app_handle.try_state::<WindowManagerState>() {
            Some(state) => state,
            None => return Ok(()),
        };

        // 使用 try_lock 避免死锁
        let window_manager = match window_manager_state.0.try_lock() {
            Ok(wm) => wm,
            Err(_) => return Ok(()),
        };

        // 同步所有受管理窗口的位置
        for window_info in window_manager.get_windows() {
            if window_info.label != "control" {
                if let Some(pos) = tracker.get_window_position(&window_info.label) {
                    if let Some(window) = app_handle.get_webview_window(&window_info.label) {
                        let _ = WindowUtils::set_window_position(&window, pos.x, pos.y);
                    }
                }
            }
        }

        Ok(())
    }
}

// 初始化窗口位置
pub fn initialize_window_position<R: Runtime>(
    app_handle: &AppHandle<R>,
    control_label: &str,
    window_label: &str,
    offset_x: f64,
    offset_y: f64,
) -> Result<(), tauri::Error> {
    WindowLayoutManager::position_window_left_of_control(
        app_handle,
        window_label,
        offset_x,
        offset_y,
    )
}

// 设置新窗口位置 - 在控制窗口左侧
pub fn set_new_window_position<R: Runtime>(
    app_handle: &AppHandle<R>,
    window_label: &str,
) -> Result<(), tauri::Error> {
    WindowLayoutManager::position_window_left_of_control(app_handle, window_label, 10.0, 0.0)
}
