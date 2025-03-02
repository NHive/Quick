use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::utils::config::WebviewUrl;
use tauri::{
    AppHandle, Error, LogicalPosition, Manager, Runtime, State, WebviewWindow, WebviewWindowBuilder,
};

use super::window_layout::WindowPositionTrackerState;
use crate::communication::events::app_events::register_quick_window_label;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[derive(Debug, Clone)]
pub enum WindowStatus {
    Foreground,
    Background,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub label: String,
    pub title: String,
    pub url: String,
    pub status: WindowStatus,
}

pub struct WindowManager {
    windows: HashMap<String, WindowInfo>,
    active_window: Option<String>,
}

impl WindowManager {
    pub fn new() -> Self {
        WindowManager {
            windows: HashMap::new(),
            active_window: None,
        }
    }

    pub fn get_windows(&self) -> Vec<WindowInfo> {
        self.windows.values().cloned().collect()
    }

    pub fn get_window_info(&self, label: &str) -> Option<WindowInfo> {
        self.windows.get(label).cloned()
    }

    pub fn get_active_window(&self) -> Option<WindowInfo> {
        match &self.active_window {
            Some(label) => self.windows.get(label).cloned(),
            None => None,
        }
    }

    pub fn add_window(&mut self, label: String, title: String, url: String) {
        let window_info = WindowInfo {
            label: label.clone(),
            title,
            url,
            status: WindowStatus::Foreground,
        };

        // Update previous active window status to background
        if let Some(active_label) = &self.active_window {
            if let Some(active_window) = self.windows.get_mut(active_label) {
                active_window.status = WindowStatus::Background;
            }
        }

        self.windows.insert(label.clone(), window_info);
        self.active_window = Some(label);
    }

    pub fn remove_window(&mut self, label: &str) {
        self.windows.remove(label);

        //如果删除的窗口处于活动状态，则更新活动窗口
        if let Some(active_label) = &self.active_window {
            if active_label == label {
                self.active_window = self.windows.keys().next().cloned();

                // 更新新的活动窗口状态
                if let Some(new_active) = &self.active_window {
                    if let Some(window) = self.windows.get_mut(new_active) {
                        window.status = WindowStatus::Foreground;
                    }
                }
            }
        }
    }

    pub fn switch_to_window(&mut self, label: &str) -> bool {
        if !self.windows.contains_key(label) {
            return false;
        }

        // 更新之前的活动窗口状态
        if let Some(active_label) = &self.active_window {
            if let Some(active_window) = self.windows.get_mut(active_label) {
                active_window.status = WindowStatus::Background;
            }
        }

        // 更新新的活动窗口状态
        if let Some(window) = self.windows.get_mut(label) {
            window.status = WindowStatus::Foreground;
        }

        self.active_window = Some(label.to_string());
        true
    }

    pub fn hide_window(&mut self, label: &str) -> bool {
        if !self.windows.contains_key(label) {
            return false;
        }

        // 更新窗口状态
        if let Some(window) = self.windows.get_mut(label) {
            window.status = WindowStatus::Background;
        }

        // 如果隐藏活动窗口，将活动窗口切换到其他窗口
        if let Some(active_label) = &self.active_window {
            if active_label == label {
                let next_window = self
                    .windows
                    .iter()
                    .filter(|(key, info)| {
                        **key != label && matches!(info.status, WindowStatus::Background)
                    })
                    .map(|(key, _)| key.clone())
                    .next();

                if let Some(next_label) = next_window {
                    if let Some(next_window) = self.windows.get_mut(&next_label) {
                        next_window.status = WindowStatus::Foreground;
                    }
                    self.active_window = Some(next_label);
                } else {
                    self.active_window = None;
                }
            }
        }

        true
    }
}

// 全局状态类型
pub struct WindowManagerState(pub Arc<Mutex<WindowManager>>);

// 生成窗口标签
pub fn generate_window_label(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();

    let hex_string = format!("{:x}", result);

    let mut label = String::new();
    for c in hex_string.chars() {
        if c.is_ascii_digit() {
            label.push((c as u8 - b'0' + b'a') as char);
        } else {
            label.push(c);
        }
    }

    format!("quick_{}", label.chars().take(20).collect::<String>())
}

// 修改 create_or_switch_window 函数来避免死锁问题
pub fn create_or_switch_window<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
    title: &str,
) -> Result<(), Error> {
    let label = generate_window_label(url);

    // 我们需要确保锁的获取顺序始终一致
    // 先获取所有需要的信息，再释放锁，然后执行操作

    let window_exists;
    let windows_to_update = {
        // 获取窗口管理器状态
        let window_manager_state = app.state::<WindowManagerState>();
        let mut window_manager = window_manager_state.0.lock().unwrap();

        // 检查窗口是否已经存在
        window_exists = window_manager.get_window_info(&label).is_some();

        // 如果窗口存在，切换到该窗口
        if window_exists {
            window_manager.switch_to_window(&label);
        } else {
            // 窗口不存在，将其添加到管理器
            window_manager.add_window(label.clone(), title.to_string(), url.to_string());
        }

        // 获取所有需要更新的窗口信息
        window_manager.get_windows()
    }; // 在这里锁被释放

    if window_exists {
        // 窗口已存在，切换到该窗口
        if let Some(window) = app.get_webview_window(&label) {
            window.show()?;
            window.set_focus()?;

            // 更新其他窗口状态，但不在持有锁的情况下
            for other_info in windows_to_update {
                if other_info.label != label && other_info.label != "control" {
                    if let Some(other_window) = app.get_webview_window(&other_info.label) {
                        match other_info.status {
                            WindowStatus::Hidden => other_window.hide()?,
                            WindowStatus::Background => other_window.show()?,
                            _ => {}
                        }
                    }
                }
            }
        }
    } else {
        // 窗口不存在，创建新窗口
        let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title(title)
            .fullscreen(false)
            .inner_size(1280.0, 768.0)
            .center()
            .resizable(true)
            .visible(false)
            .skip_taskbar(false)
            .always_on_top(true);

        // 注册quick窗口
        register_quick_window_label(app, label.clone());

        // 根据操作系统设置不同的窗口样式
        #[cfg(target_os = "macos")]
        {
            builder = builder.title_bar_style(TitleBarStyle::Overlay);
        }
        // windows以及Linux下去掉标题栏
        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
            builder = builder.transparent(true);
        }

        let new_window = builder.build()?;

        // 注册窗口关闭事件
        let app_handle = app.clone();
        let window_label = label.clone();
        new_window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = app_handle.try_state::<WindowManagerState>() {
                    let mut manager = state.0.lock().unwrap();
                    manager.remove_window(&window_label);
                }
            }
        });

        // 设置新窗口位置，但在单独的块中以确保锁的正确释放
        {
            // 这里的代码独立处理窗口位置，避免锁定顺序问题
            set_new_window_position_safely(app, &label)?;
        }

        // 显示新窗口
        new_window.show()?;
        new_window.set_focus()?;

        // 更新其他窗口状态
        for other_info in windows_to_update {
            if other_info.label != label && other_info.label != "control" {
                if let Some(other_window) = app.get_webview_window(&other_info.label) {
                    match other_info.status {
                        WindowStatus::Hidden => other_window.hide()?,
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

// 关闭窗口
pub fn close_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 获取窗口管理器状态
    let window_manager_state = app.state::<WindowManagerState>();
    let mut window_manager = window_manager_state.0.lock().unwrap();

    // 从Tauri获取窗口实例并关闭
    if let Some(window) = app.get_webview_window(label) {
        window.close()?;

        // 从管理器中移除窗口
        window_manager.remove_window(label);

        // 如果有下一个活动窗口，将其显示
        if let Some(active_info) = window_manager.get_active_window() {
            if let Some(active_window) = app.get_webview_window(&active_info.label) {
                active_window.show()?;
                active_window.set_focus()?;
            }
        }
    }

    Ok(())
}

// 隐藏窗口
pub fn hide_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 获取窗口管理器状态
    let window_manager_state = app.state::<WindowManagerState>();
    let mut window_manager = window_manager_state.0.lock().unwrap();

    // 更新窗口状态为隐藏
    window_manager.hide_window(label);

    // 从Tauri获取窗口实例并隐藏
    if let Some(window) = app.get_webview_window(label) {
        window.hide()?;

        // 如果有下一个活动窗口，将其显示
        if let Some(active_info) = window_manager.get_active_window() {
            if let Some(active_window) = app.get_webview_window(&active_info.label) {
                active_window.show()?;
                active_window.set_focus()?;
            }
        }
    }

    Ok(())
}

// 将窗口置顶
pub fn bring_to_front<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 获取窗口管理器状态
    let window_manager_state = app.state::<WindowManagerState>();
    let mut window_manager = window_manager_state.0.lock().unwrap();

    // 切换到指定窗口
    if window_manager.switch_to_window(label) {
        // 从Tauri获取窗口实例并显示
        if let Some(window) = app.get_webview_window(label) {
            window.show()?;
            window.set_focus()?;

            // 更新其他窗口状态
            for other_info in window_manager.get_windows() {
                if other_info.label != label {
                    if let Some(other_window) = app.get_webview_window(&other_info.label) {
                        match other_info.status {
                            WindowStatus::Background => {
                                // 后台窗口保持可见，但不聚焦
                                other_window.hide()?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// 获取所有窗口信息
pub fn get_all_windows<R: Runtime>(app: &AppHandle<R>) -> Vec<WindowInfo> {
    let window_manager_state = app.state::<WindowManagerState>();
    let window_manager = window_manager_state.0.lock().unwrap();
    window_manager.get_windows()
}

// 获取当前活动窗口
pub fn get_active_window<R: Runtime>(app: &AppHandle<R>) -> Option<WindowInfo> {
    let window_manager_state = app.state::<WindowManagerState>();
    let window_manager = window_manager_state.0.lock().unwrap();
    window_manager.get_active_window()
}

// 安全地设置新窗口位置的辅助函数，避免死锁
pub fn set_new_window_position_safely<R: Runtime>(
    app_handle: &AppHandle<R>,
    window_label: &str,
) -> Result<(), tauri::Error> {
    // 获取控制窗口的位置信息
    let control_position;
    let control_size;

    if let Some(control_window) = app_handle.get_webview_window("control") {
        control_position = control_window.outer_position()?;
        control_size = control_window.inner_size()?;
    } else {
        // 控制窗口不存在，无法设置相对位置
        return Ok(());
    }

    // 计算新窗口位置 - 在控制窗口左侧
    let window_x = control_position.x as f64; // 窗口宽度 + 间隔
    let window_y = control_position.y as f64; // 与控制窗口顶部对齐

    // 设置窗口位置
    if let Some(window) = app_handle.get_webview_window(window_label) {
        window.set_position(LogicalPosition::new(window_x, window_y))?;

        // 获取窗口的实际大小
        let window_size = window.inner_size()?;

        // 更新跟踪器中的位置信息 - 在所有其他操作完成后
        if let Some(tracker_state) = app_handle.try_state::<WindowPositionTrackerState>() {
            let mut tracker = tracker_state.0.lock().unwrap();

            // 更新控制窗口位置
            tracker.update_control_position(
                control_position.x as f64,
                control_position.y as f64,
                control_size.width as f64,
                control_size.height as f64,
            );

            // 更新新窗口位置
            tracker.update_window_position(
                window_label.to_string(),
                window_x,
                window_y,
                window_size.width as f64,
                window_size.height as f64,
            );
        }
    }

    Ok(())
}
