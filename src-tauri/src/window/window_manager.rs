// file_path: src/window/window_manager.rs
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::utils::config::WebviewUrl;
use tauri::{AppHandle, Error, LogicalPosition, Manager, Runtime, WebviewWindowBuilder};
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

// =============== 数据结构定义 ===============

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
    pub position: Option<WindowPosition>, // 集成位置信息
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// =============== 消息系统定义 ===============

// 窗口管理消息，用于异步操作
#[derive(Debug)]
pub enum WindowManagerMessage {
    UpdatePosition {
        label: String,
        position: WindowPosition,
    },
    SyncPositions,
    WindowCreated {
        label: String,
    },
}

// =============== 统一的窗口管理器 ===============

pub struct WindowManager {
    windows: HashMap<String, WindowInfo>,
    active_window: Option<String>,
    previous_active_window: Option<String>,
    window_configs: Vec<WindowConfig>,
    control_position: Option<WindowPosition>,
    is_updating: bool,
    pub tx: mpsc::Sender<WindowManagerMessage>,
}

impl WindowManager {
    pub fn new(tx: mpsc::Sender<WindowManagerMessage>) -> Self {
        WindowManager {
            windows: HashMap::new(),
            active_window: None,
            previous_active_window: None,
            window_configs: Vec::new(),
            control_position: None,
            is_updating: false,
            tx,
        }
    }

    // 获取所有窗口信息
    pub fn get_windows(&self) -> Vec<WindowInfo> {
        self.windows.values().cloned().collect()
    }

    // 获取特定窗口信息
    pub fn get_window_info(&self, label: &str) -> Option<WindowInfo> {
        self.windows.get(label).cloned()
    }

    // 获取当前激活窗口
    pub fn get_active_window(&self) -> Option<WindowInfo> {
        match &self.active_window {
            Some(label) => self.windows.get(label).cloned(),
            None => None,
        }
    }

    // 获取上一个激活窗口
    pub fn get_previous_active_window(&self) -> Option<WindowInfo> {
        match &self.previous_active_window {
            Some(label) => self.windows.get(label).cloned(),
            None => None,
        }
    }

    // 设置窗口配置列表
    pub fn set_window_configs(&mut self, configs: Vec<WindowConfig>) {
        self.window_configs = configs;
    }

    // 获取窗口配置列表
    pub fn get_window_configs(&self) -> Vec<WindowConfig> {
        self.window_configs.clone()
    }

    // 添加新窗口
    pub fn add_window(&mut self, label: String, title: String, url: String) {
        // 将当前激活窗口更新为后台
        if let Some(active_label) = &self.active_window {
            // 保存为上一个激活窗口
            self.previous_active_window = Some(active_label.clone());

            if let Some(active_window) = self.windows.get_mut(active_label) {
                active_window.status = WindowStatus::Background;
            }
        }

        // 创建新窗口信息
        let window_info = WindowInfo {
            label: label.clone(),
            title,
            url,
            status: WindowStatus::Foreground,
            loaded: false,
            position: None,
        };

        self.windows.insert(label.clone(), window_info);
        self.active_window = Some(label);
    }

    // 删除窗口
    pub fn remove_window(&mut self, label: &str) {
        self.windows.remove(label);

        // 如果删除的是当前激活窗口，更新激活窗口
        if let Some(active_label) = &self.active_window {
            if active_label == label {
                // 尝试切换到上一个激活窗口
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

                // 如果没有上一个激活窗口或已不存在，选择任意一个窗口
                self.active_window = self.windows.keys().next().cloned();

                // 更新新的激活窗口状态
                if let Some(new_active) = &self.active_window {
                    if let Some(window) = self.windows.get_mut(new_active) {
                        window.status = WindowStatus::Foreground;
                    }
                }
            } else if let Some(prev_label) = &self.previous_active_window {
                // 如果删除的是上一个激活窗口，更新上一个激活窗口引用
                if prev_label == label {
                    self.previous_active_window = None;
                }
            }
        }
    }

    // 切换到指定窗口
    pub fn switch_to_window(&mut self, label: &str) -> bool {
        if !self.windows.contains_key(label) {
            return false;
        }

        // 如果已经是激活窗口，不做操作
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

    // 标记窗口已加载
    pub fn mark_window_loaded(&mut self, label: &str) -> bool {
        if let Some(window) = self.windows.get_mut(label) {
            window.loaded = true;

            // 发送窗口创建消息，以便后续处理位置
            let tx = self.tx.clone();
            let label_clone = label.to_string();
            tauri::async_runtime::spawn(async move {
                let _ = tx
                    .send(WindowManagerMessage::WindowCreated { label: label_clone })
                    .await;
            });

            return true;
        }
        false
    }

    // 更新控制窗口位置
    pub fn update_control_position(&mut self, position: WindowPosition) -> bool {
        // 避免递归更新
        if self.is_updating {
            return false;
        }

        // 检查位置是否有显著变化
        if let Some(existing) = &self.control_position {
            if (existing.x - position.x).abs() < 1.0
                && (existing.y - position.y).abs() < 1.0
                && (existing.width - position.width).abs() < 1.0
                && (existing.height - position.height).abs() < 1.0
            {
                return false;
            }
        }

        // 计算位移量
        let delta_x;
        let delta_y;

        if let Some(existing) = &self.control_position {
            delta_x = position.x - existing.x;
            delta_y = position.y - existing.y;

            // 位移太小不更新
            if delta_x.abs() < 1.0 && delta_y.abs() < 1.0 {
                return false;
            }
        } else {
            delta_x = 0.0;
            delta_y = 0.0;
        }

        // 更新控制窗口位置
        self.control_position = Some(position);

        // 如果有显著位移，异步更新所有窗口位置
        if delta_x.abs() >= 1.0 || delta_y.abs() >= 1.0 {
            let tx = self.tx.clone();
            tauri::async_runtime::spawn(async move {
                let _ = tx.send(WindowManagerMessage::SyncPositions).await;
            });
        }

        true
    }

    // 更新窗口位置
    pub fn update_window_position(&mut self, label: String, position: WindowPosition) -> bool {
        // 避免递归更新
        if self.is_updating {
            return false;
        }

        // 检查位置变化是否显著
        if let Some(window) = self.windows.get_mut(&label) {
            if let Some(existing) = &window.position {
                if (existing.x - position.x).abs() < 1.0
                    && (existing.y - position.y).abs() < 1.0
                    && (existing.width - position.width).abs() < 1.0
                    && (existing.height - position.height).abs() < 1.0
                {
                    return false;
                }
            }

            // 更新位置信息
            window.position = Some(position);
            return true;
        }

        false
    }

    // 获取窗口位置
    pub fn get_window_position(&self, label: &str) -> Option<WindowPosition> {
        if let Some(window) = self.windows.get(label) {
            window.position
        } else {
            None
        }
    }

    // 获取控制窗口位置
    pub fn get_control_position(&self) -> Option<WindowPosition> {
        self.control_position
    }
}

// =============== 全局窗口管理器状态 ===============

pub struct WindowManagerState(pub Arc<Mutex<WindowManager>>);

// =============== 工具函数 ===============

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

// 获取窗口位置和大小
pub fn get_window_position_and_size<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
) -> Result<WindowPosition, Error> {
    let window = app.get_webview_window(label).ok_or_else(|| {
        Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Window {} not found", label),
        ))
    })?;

    let position = window.outer_position()?;
    let size = window.inner_size()?;

    Ok(WindowPosition {
        x: position.x as f64,
        y: position.y as f64,
        width: size.width as f64,
        height: size.height as f64,
    })
}

// 初始化窗口管理器和消息处理系统
pub fn init_window_manager<R: Runtime>(app: &AppHandle<R>) -> mpsc::Sender<WindowManagerMessage> {
    // 创建消息通道
    let (tx, mut rx) = mpsc::channel::<WindowManagerMessage>(100);

    // 创建窗口管理器实例
    let window_manager = WindowManager::new(tx.clone());
    let window_manager_state = WindowManagerState(Arc::new(Mutex::new(window_manager)));

    // 注册为应用状态
    app.manage(window_manager_state);

    // 克隆AppHandle用于消息处理
    let app_handle = app.clone();

    // 启动消息处理循环
    tauri::async_runtime::spawn(async move {
        while let Some(msg) = rx.recv().await {
            process_manager_message(&app_handle, msg).await;
        }
    });

    tx
}

// 处理窗口管理器消息
async fn process_manager_message<R: Runtime>(app: &AppHandle<R>, msg: WindowManagerMessage) {
    match msg {
        WindowManagerMessage::UpdatePosition { label, position } => {
            // 处理位置更新消息
            if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
                if let Ok(mut manager) = window_manager_state.0.try_lock() {
                    // 标记更新状态，防止递归
                    manager.is_updating = true;

                    if label == "control" {
                        manager.update_control_position(position);
                    } else {
                        manager.update_window_position(label.clone(), position);
                    }

                    manager.is_updating = false;
                }
            }
        }
        WindowManagerMessage::SyncPositions => {
            // 同步所有窗口位置
            sync_window_positions(app);
        }
        WindowManagerMessage::WindowCreated { label } => {
            // 窗口创建后定位处理
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            position_window_left_of_control(app, &label, 10.0, 0.0);
        }
    }
}

// =============== 主要API函数 ===============

// 配置窗口列表
pub fn configure_windows<R: Runtime>(
    app: &AppHandle<R>,
    configs: Vec<WindowConfig>,
) -> Result<(), Error> {
    // 获取窗口管理器状态
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
            // 更新窗口配置
            window_manager.set_window_configs(configs);
            return Ok(());
        }
    }

    Err(Error::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to access window manager",
    )))
}

// 创建或切换到窗口
pub fn create_or_switch_window<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
    title: &str,
) -> Result<(), Error> {
    let label = generate_window_label(url);

    // 先检查窗口是否存在
    let window_exists = app.get_webview_window(&label).is_some();

    // 记录要更新的窗口信息
    let (active_changed, to_hide) = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
                let was_active = window_manager
                    .get_active_window()
                    .map(|w| w.label == label)
                    .unwrap_or(false);

                // 如果窗口不存在，添加到管理器
                if !window_exists {
                    window_manager.add_window(label.clone(), title.to_string(), url.to_string());
                } else if !was_active {
                    // 窗口存在但不是激活窗口，切换到它
                    window_manager.switch_to_window(&label);
                }

                // 获取需要隐藏的窗口
                let to_hide = window_manager
                    .get_windows()
                    .into_iter()
                    .filter(|w| {
                        w.label != label
                            && w.label != "control"
                            && w.status == WindowStatus::Background
                    })
                    .map(|w| w.label)
                    .collect::<Vec<_>>();

                (was_active, to_hide)
            } else {
                (false, Vec::new())
            }
        } else {
            (false, Vec::new())
        }
    };

    if window_exists {
        // 窗口已存在，仅切换到它
        if let Some(window) = app.get_webview_window(&label) {
            window.show()?;
            window.set_focus()?;

            // 隐藏背景窗口
            for hide_label in to_hide {
                if let Some(other_window) = app.get_webview_window(&hide_label) {
                    other_window.hide()?;
                }
            }
        }
    } else {
        // 创建新窗口
        let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title(title)
            .fullscreen(false)
            .inner_size(1280.0, 768.0)
            .center()
            .resizable(true)
            .visible(false) // 先隐藏再显示，避免闪烁
            .skip_taskbar(true)
            .decorations(false)
            .always_on_top(true);

        // 根据操作系统设置不同的窗口样式
        #[cfg(target_os = "macos")]
        {
            builder = builder.title_bar_style(TitleBarStyle::Overlay);
            builder = builder.hidden_title(true);
        }

        #[cfg(not(target_os = "macos"))]
        {
            builder = builder.decorations(false);
            builder = builder.transparent(true);
        }

        // 构建窗口
        let new_window = builder.build()?;

        // 标记窗口为已加载
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
                window_manager.mark_window_loaded(&label);
            }
        }

        // 显示新窗口
        new_window.show()?;
        new_window.set_focus()?;

        // 隐藏背景窗口
        for hide_label in to_hide {
            if let Some(other_window) = app.get_webview_window(&hide_label) {
                other_window.hide()?;
            }
        }
    }

    Ok(())
}

// 切换到特定窗口
pub fn switch_to_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 获取窗口信息和需要隐藏的窗口
    let (window_exists, create_from_config, config_info, to_hide) = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
                let window_exists = window_manager.get_window_info(label).is_some();

                // 检查是否有窗口配置
                let create_from_config = !window_exists;
                let config_info = if !window_exists {
                    let configs = window_manager.get_window_configs();
                    configs
                        .iter()
                        .find(|c| generate_window_label(&c.url) == label)
                        .cloned()
                } else {
                    None
                };

                // 如果窗口存在，切换到它
                if window_exists {
                    window_manager.switch_to_window(label);
                }

                // 获取需要隐藏的窗口
                let to_hide = window_manager
                    .get_windows()
                    .into_iter()
                    .filter(|w| {
                        w.label != label
                            && w.label != "control"
                            && w.status == WindowStatus::Background
                    })
                    .map(|w| w.label)
                    .collect::<Vec<_>>();

                (window_exists, create_from_config, config_info, to_hide)
            } else {
                (false, false, None, Vec::new())
            }
        } else {
            (false, false, None, Vec::new())
        }
    };

    if window_exists {
        // 窗口存在，显示并置于前台
        if let Some(window) = app.get_webview_window(label) {
            window.show()?;
            window.set_focus()?;

            // 隐藏其他窗口
            for hide_label in to_hide {
                if let Some(other_window) = app.get_webview_window(&hide_label) {
                    other_window.hide()?;
                }
            }
        }
    } else if create_from_config {
        // 从配置创建新窗口
        if let Some(config) = config_info {
            create_or_switch_window(app, &config.url, &config.title)?;
        }
    }

    Ok(())
}

// 获取所有窗口信息
pub fn get_all_windows<R: Runtime>(app: &AppHandle<R>) -> Vec<WindowInfo> {
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            return window_manager.get_windows();
        }
    }

    Vec::new()
}

// 获取活动窗口
pub fn get_active_window<R: Runtime>(app: &AppHandle<R>) -> Option<WindowInfo> {
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            return window_manager.get_active_window();
        }
    }

    None
}

// 获取上一个活动窗口
pub fn get_previous_active_window<R: Runtime>(app: &AppHandle<R>) -> Option<WindowInfo> {
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            return window_manager.get_previous_active_window();
        }
    }

    None
}

// 显示上一个活动窗口
pub fn show_previous_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), Error> {
    let previous_label = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(window_manager) = window_manager_state.0.try_lock() {
                window_manager
                    .get_previous_active_window()
                    .map(|info| info.label)
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(label) = previous_label {
        switch_to_window(app, &label)?;
    }

    Ok(())
}

// 将窗口定位在控制窗口左侧
pub fn position_window_left_of_control<R: Runtime>(
    app: &AppHandle<R>,
    window_label: &str,
    offset_x: f64,
    offset_y: f64,
) {
    // 获取控制窗口位置
    let control_position = match get_window_position_and_size(app, "control") {
        Ok(pos) => pos,
        Err(_) => return,
    };

    // 获取目标窗口大小
    let window_size = match get_window_position_and_size(app, window_label) {
        Ok(pos) => WindowPosition {
            x: 0.0,
            y: 0.0,
            width: pos.width,
            height: pos.height,
        },
        Err(_) => return,
    };

    // 计算新位置 - 在控制窗口左侧
    let window_x = control_position.x - window_size.width - offset_x;
    let window_y = control_position.y + offset_y;

    // 设置窗口位置
    if let Some(window) = app.get_webview_window(window_label) {
        let _ = window.set_position(LogicalPosition::new(window_x, window_y));

        // 更新窗口管理器中的位置
        let window_position = WindowPosition {
            x: window_x,
            y: window_y,
            width: window_size.width,
            height: window_size.height,
        };

        // 发送位置更新消息
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(window_manager) = window_manager_state.0.try_lock() {
                let tx = window_manager.tx.clone();
                let label = window_label.to_string();
                tauri::async_runtime::spawn(async move {
                    let _ = tx
                        .send(WindowManagerMessage::UpdatePosition {
                            label,
                            position: window_position,
                        })
                        .await;
                });
            }
        }
    }
}

// 同步所有窗口位置
pub fn sync_window_positions<R: Runtime>(app: &AppHandle<R>) {
    // 获取控制窗口和所有窗口的信息
    let (control_position, windows_info) = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(window_manager) = window_manager_state.0.try_lock() {
                let control_pos = window_manager.get_control_position();
                let windows = window_manager.get_windows();
                (control_pos, windows)
            } else {
                return;
            }
        } else {
            return;
        }
    };

    // 如果没有控制窗口位置，不进行同步
    let control_position = match control_position {
        Some(pos) => pos,
        None => return,
    };

    // 遍历所有非控制窗口
    for window_info in windows_info {
        if window_info.label == "control" || !window_info.loaded {
            continue;
        }

        // 获取该窗口的位置信息
        let window_position = match get_window_position_and_size(app, &window_info.label) {
            Ok(pos) => pos,
            Err(_) => continue,
        };

        // 根据控制窗口位置计算新位置
        // 保持窗口在控制窗口左侧的相对位置
        let window_x = control_position.x - window_position.width - 10.0;
        let window_y = control_position.y;

        // 设置窗口位置
        if let Some(window) = app.get_webview_window(&window_info.label) {
            let _ = window.set_position(LogicalPosition::new(window_x, window_y));

            // 更新窗口管理器中的位置信息
            let new_position = WindowPosition {
                x: window_x,
                y: window_y,
                width: window_position.width,
                height: window_position.height,
            };

            // 更新窗口管理器中的位置记录
            if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
                if let Ok(mut manager) = window_manager_state.0.try_lock() {
                    manager.update_window_position(window_info.label.clone(), new_position);
                }
            }
        }
    }
}
