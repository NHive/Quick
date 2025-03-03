use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::utils::config::WebviewUrl;
use tauri::{AppHandle, Error, LogicalPosition, Manager, Runtime, WebviewWindowBuilder};

use super::window_layout::WindowPositionTrackerState;
use crate::communication::events::app_events::{
    register_quick_window_label, sync_quick_windows_position,
};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum WindowStatus {
    Foreground,
    Background,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowInfo {
    pub label: String,
    pub title: String,
    pub url: String,
    pub status: WindowStatus,
    pub loaded: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    pub title: String,
    pub url: String,
}

pub struct WindowManager {
    windows: HashMap<String, WindowInfo>,
    active_window: Option<String>,
    previous_active_window: Option<String>, // 存储上一个激活的窗口标签
    window_configs: Vec<WindowConfig>,
}

impl WindowManager {
    pub fn new() -> Self {
        WindowManager {
            windows: HashMap::new(),
            active_window: None,
            previous_active_window: None, // 初始化为None
            window_configs: Vec::new(),
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

    // 获取上一个激活的窗口信息
    pub fn get_previous_active_window(&self) -> Option<WindowInfo> {
        match &self.previous_active_window {
            Some(label) => self.windows.get(label).cloned(),
            None => None,
        }
    }

    pub fn set_window_configs(&mut self, configs: Vec<WindowConfig>) {
        self.window_configs = configs;
    }

    pub fn get_window_configs(&self) -> Vec<WindowConfig> {
        self.window_configs.clone()
    }

    pub fn add_window(&mut self, label: String, title: String, url: String) {
        let window_info = WindowInfo {
            label: label.clone(),
            title,
            url,
            status: WindowStatus::Foreground,
            loaded: false,
        };

        // 将当前激活窗口状态更新为后台
        if let Some(active_label) = &self.active_window {
            // 保存当前激活窗口为上一个激活窗口
            self.previous_active_window = Some(active_label.clone());

            if let Some(active_window) = self.windows.get_mut(active_label) {
                active_window.status = WindowStatus::Background;
            }
        }

        self.windows.insert(label.clone(), window_info);
        self.active_window = Some(label);
    }

    pub fn remove_window(&mut self, label: &str) {
        self.windows.remove(label);

        // 如果删除的是当前激活窗口，更新激活窗口
        if let Some(active_label) = &self.active_window {
            if active_label == label {
                // 如果删除的是当前激活窗口，尝试切换到上一个激活窗口
                if let Some(prev_label) = &self.previous_active_window {
                    if self.windows.contains_key(prev_label) {
                        self.active_window = Some(prev_label.clone());

                        // 更新新的激活窗口状态
                        if let Some(window) = self.windows.get_mut(prev_label) {
                            window.status = WindowStatus::Foreground;
                        }
                        return;
                    }
                }

                // 如果没有上一个激活窗口或已不存在，则选择任意一个窗口
                self.active_window = self.windows.keys().next().cloned();

                // 更新新的激活窗口状态
                if let Some(new_active) = &self.active_window {
                    if let Some(window) = self.windows.get_mut(new_active) {
                        window.status = WindowStatus::Foreground;
                    }
                }
            } else if let Some(prev_label) = &self.previous_active_window {
                // 如果删除的是上一个激活窗口，需要更新上一个激活窗口引用
                if prev_label == label {
                    self.previous_active_window = None;
                }
            }
        }
    }

    pub fn switch_to_window(&mut self, label: &str) -> bool {
        if !self.windows.contains_key(label) {
            return false;
        }

        // 如果切换到的窗口已经是激活窗口，不做任何操作
        if let Some(active_label) = &self.active_window {
            if active_label == label {
                return true;
            }

            // 保存当前激活窗口为上一个激活窗口
            self.previous_active_window = Some(active_label.clone());

            // 更新上一个激活窗口状态
            if let Some(active_window) = self.windows.get_mut(active_label) {
                active_window.status = WindowStatus::Background;
            }
        }

        // 更新新的激活窗口状态
        if let Some(window) = self.windows.get_mut(label) {
            window.status = WindowStatus::Foreground;
            window.loaded = true;
        }

        self.active_window = Some(label.to_string());
        true
    }

    pub fn mark_window_loaded(&mut self, label: &str) -> bool {
        if let Some(window) = self.windows.get_mut(label) {
            window.loaded = true;
            return true;
        }
        false
    }
}

// 全局访问的状态类型
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

// 配置窗口列表
pub fn configure_windows<R: Runtime>(
    app: &AppHandle<R>,
    configs: Vec<WindowConfig>,
) -> Result<(), Error> {
    // 获取窗口管理器状态
    let window_manager_state = app.state::<WindowManagerState>();
    let mut window_manager = window_manager_state.0.lock().unwrap();

    // 更新窗口配置
    window_manager.set_window_configs(configs);

    Ok(())
}

// 创建或切换到窗口函数
pub fn create_or_switch_window<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
    title: &str,
) -> Result<(), Error> {
    let label = generate_window_label(url);

    // 先获取所有必要信息，然后释放锁，再执行操作
    let (window_exists, window_loaded, windows_to_update) = {
        // 获取窗口管理器状态
        let window_manager_state = app.state::<WindowManagerState>();
        let mut window_manager = window_manager_state.0.lock().unwrap();

        // 检查窗口是否已存在
        let window_info = window_manager.get_window_info(&label);
        let window_exists = window_info.is_some();
        let window_loaded = window_info.map(|info| info.loaded).unwrap_or(false);

        // 如果窗口存在，切换到它
        if window_exists {
            window_manager.switch_to_window(&label);
        } else {
            // 窗口不存在，将其添加到管理器
            window_manager.add_window(label.clone(), title.to_string(), url.to_string());
        }

        // 获取所有需要更新的窗口
        (window_exists, window_loaded, window_manager.get_windows())
    }; // 锁在这里释放

    if window_exists {
        // 窗口存在，切换到它
        if let Some(window) = app.get_webview_window(&label) {
            window.show()?;
            window.set_focus()?;

            // 在不持有锁的情况下更新其他窗口状态
            for other_info in windows_to_update {
                if other_info.label != label && other_info.label != "control" {
                    if let Some(other_window) = app.get_webview_window(&other_info.label) {
                        if other_info.status == WindowStatus::Background {
                            other_window.hide()?;
                        }
                    }
                }
            }
        }
    } else {
        // 窗口不存在或未加载，创建新窗口
        let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title(title)
            .fullscreen(false)
            .inner_size(1280.0, 768.0)
            .center()
            .resizable(true)
            .visible(false)
            .skip_taskbar(true)
            .decorations(false)
            .always_on_top(true);

        // 注册快速窗口
        register_quick_window_label(app, label.clone());

        // 根据操作系统设置不同的窗口样式
        #[cfg(target_os = "macos")]
        {
            builder = builder.title_bar_style(TitleBarStyle::Overlay);
            builder = builder.hidden_title(true);
        }
        // 在Windows和Linux上移除标题栏
        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
            builder = builder.transparent(true);
        }

        let new_window = builder.build()?;

        // 标记窗口为已加载
        {
            let window_manager_state = app.state::<WindowManagerState>();
            let mut window_manager = window_manager_state.0.lock().unwrap();
            window_manager.mark_window_loaded(&label);
        }

        // 安全地设置新窗口位置
        {
            set_new_window_position_safely(app, &label)?;
        }

        // 显示新窗口
        new_window.show()?;
        new_window.set_focus()?;

        // 更新其他窗口状态
        for other_info in windows_to_update {
            if other_info.label != label && other_info.label != "control" {
                if let Some(other_window) = app.get_webview_window(&other_info.label) {
                    if other_info.status == WindowStatus::Background {
                        other_window.hide()?;
                    }
                }
            }
        }
    }

    sync_quick_windows_position(app);

    Ok(())
}

// 获取所有窗口信息
pub fn get_all_windows<R: Runtime>(app: &AppHandle<R>) -> Vec<WindowInfo> {
    let window_manager_state = app.state::<WindowManagerState>();
    let window_manager = window_manager_state.0.lock().unwrap();
    window_manager.get_windows()
}

// 获取激活窗口
pub fn get_active_window<R: Runtime>(app: &AppHandle<R>) -> Option<WindowInfo> {
    let window_manager_state = app.state::<WindowManagerState>();
    let window_manager = window_manager_state.0.lock().unwrap();
    window_manager.get_active_window()
}

// 获取上一个激活的窗口
pub fn get_previous_active_window<R: Runtime>(app: &AppHandle<R>) -> Option<WindowInfo> {
    let window_manager_state = app.state::<WindowManagerState>();
    let window_manager = window_manager_state.0.lock().unwrap();
    window_manager.get_previous_active_window()
}

// 显示上一个激活的窗口
pub fn show_previous_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), Error> {
    // 获取上一个激活窗口的标签
    let previous_label = {
        let window_manager_state = app.state::<WindowManagerState>();
        let window_manager = window_manager_state.0.lock().unwrap();

        // 如果没有上一个激活窗口，直接返回
        match window_manager.get_previous_active_window() {
            Some(info) => info.label,
            None => return Ok(()), // 没有上一个窗口，直接返回成功
        }
    }; // 这里释放锁

    // 调用switch_to_window切换到上一个窗口
    switch_to_window(app, &previous_label)
}

// 切换到指定窗口
pub fn switch_to_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    let (should_create, window_exists, window_info, windows_to_update) = {
        // 获取窗口管理器状态
        let window_manager_state = app.state::<WindowManagerState>();
        let mut window_manager = window_manager_state.0.lock().unwrap();

        // 检查窗口是否存在
        let window_exists = window_manager.get_window_info(label).is_some();
        let window_info = if window_exists {
            window_manager.get_window_info(label)
        } else {
            // 检查是否有该窗口的配置
            let configs = window_manager.get_window_configs();
            let config = configs
                .iter()
                .find(|c| generate_window_label(&c.url) == label);

            if let Some(config) = config {
                // 将窗口添加到管理器，但不创建Tauri窗口
                window_manager.add_window(
                    label.to_string(),
                    config.title.clone(),
                    config.url.clone(),
                );
                window_manager.get_window_info(label)
            } else {
                None
            }
        };

        // 如果窗口存在，切换到它
        if window_exists {
            window_manager.switch_to_window(label);
        }

        // 获取所有需要更新的窗口
        let windows_to_update = window_manager.get_windows();
        let should_create = !window_exists && window_info.is_some();

        (should_create, window_exists, window_info, windows_to_update)
    }; // 锁在这里释放

    // 如果窗口在Tauri中存在，切换到它
    if window_exists {
        if let Some(window) = app.get_webview_window(label) {
            window.show()?;
            window.set_focus()?;

            // 隐藏其他窗口
            for other_info in windows_to_update {
                if other_info.label != label && other_info.label != "control" {
                    if let Some(other_window) = app.get_webview_window(&other_info.label) {
                        if other_info.status == WindowStatus::Background {
                            other_window.hide()?;
                        }
                    }
                }
            }
        }
    } else if should_create {
        // 从配置创建窗口
        if let Some(info) = window_info {
            create_or_switch_window(app, &info.url, &info.title)?;
        }
    }

    Ok(())
}

pub fn set_new_window_position_safely<R: Runtime>(
    app_handle: &AppHandle<R>,
    window_label: &str,
) -> Result<(), tauri::Error> {
    // 获取控制窗口位置信息
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
    let window_x = control_position.x as f64;
    let window_y = control_position.y as f64; // 与控制窗口顶部对齐

    // 设置窗口位置
    if let Some(window) = app_handle.get_webview_window(window_label) {
        window.set_position(LogicalPosition::new(window_x, window_y))?;

        // 获取窗口的实际大小
        let window_size = window.inner_size()?;

        // 在所有其他操作之后更新跟踪器中的位置信息
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
