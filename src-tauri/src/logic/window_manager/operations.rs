// 窗口操作的API实现

use tauri::utils::config::WebviewUrl;
use tauri::{AppHandle, Error, LogicalPosition, Manager, Runtime, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

use super::models::{WindowConfig, WindowInfo, WindowManagerState, WindowPosition, WindowStatus};
use super::utils::{generate_window_label, get_window_position_and_size};

/// 配置窗口列表
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
        "无法访问窗口管理器",
    )))
}

/// 创建或切换到窗口
pub fn create_or_switch_window<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
    title: &str,
) -> Result<(), Error> {
    let label = generate_window_label(url);

    // 首先检查窗口是否存在
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
                    // 窗口存在但不活跃，切换到它
                    window_manager.switch_to_window(&label);
                }

                // 获取要隐藏的窗口
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
        // 窗口已存在，只需切换到它
        if let Some(window) = app.get_webview_window(&label) {
            window.show()?;
            window.set_focus()?;

            // 将控制窗口定位在此快速窗口下方
            position_control_window_below_quick(app, &label)?;

            // 隐藏后台窗口
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
            .visible(false) // 先隐藏再显示以避免闪烁
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

        // 将控制窗口定位在此快速窗口下方
        tauri::async_runtime::spawn(async {
            // 小延迟确保窗口完全渲染
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        });

        position_control_window_below_quick(app, &label)?;

        // 隐藏后台窗口
        for hide_label in to_hide {
            if let Some(other_window) = app.get_webview_window(&hide_label) {
                other_window.hide()?;
            }
        }
    }

    Ok(())
}

/// 切换到特定窗口
pub fn switch_to_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 获取窗口信息和要隐藏的窗口
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

                // 获取要隐藏的窗口
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
        // 窗口存在，显示并带到前面
        if let Some(window) = app.get_webview_window(label) {
            window.show()?;
            window.set_focus()?;

            // 将控制窗口定位在此快速窗口下方
            position_control_window_below_quick(app, label)?;

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

/// 获取所有窗口信息
pub fn get_all_windows<R: Runtime>(app: &AppHandle<R>) -> Vec<WindowInfo> {
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            return window_manager.get_windows();
        }
    }

    Vec::new()
}

/// 获取活跃窗口
pub fn get_active_window<R: Runtime>(app: &AppHandle<R>) -> Option<WindowInfo> {
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            return window_manager.get_active_window();
        }
    }

    None
}

/// 获取前一个活跃窗口
pub fn get_previous_active_window<R: Runtime>(app: &AppHandle<R>) -> Option<WindowInfo> {
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(window_manager) = window_manager_state.0.try_lock() {
            return window_manager.get_previous_active_window();
        }
    }

    None
}

/// 显示前一个活跃窗口
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

/// 将控制窗口定位在快速窗口下方
pub fn position_control_window_below_quick<R: Runtime>(
    app: &AppHandle<R>,
    quick_window_label: &str,
) -> Result<(), Error> {
    // 获取快速窗口位置
    let quick_position = get_window_position_and_size(app, quick_window_label)?;

    // 获取控制窗口大小
    let control_position = match get_window_position_and_size(app, "control") {
        Ok(pos) => pos,
        Err(_) => return Ok(()), // 控制窗口可能尚不存在
    };

    // 计算新位置 - 在快速窗口下方居中
    // 控制窗口的中心与快速窗口的中心对齐
    let control_x = quick_position.x + (quick_position.width - control_position.width) / 2.0;
    let control_y = quick_position.y + quick_position.height + 10.0; // 10px间隙

    // 设置控制窗口位置
    if let Some(control_window) = app.get_webview_window("control") {
        let _ = control_window.set_position(LogicalPosition::new(control_x, control_y));

        // 更新窗口管理器位置
        let updated_position = WindowPosition {
            x: control_x,
            y: control_y,
            width: control_position.width,
            height: control_position.height,
        };

        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut manager) = window_manager_state.0.try_lock() {
                manager.update_control_position(updated_position);
            }
        }
    }

    Ok(())
}

/// 将快速窗口定位在控制窗口上方
pub fn position_quick_window_above_control<R: Runtime>(
    app: &AppHandle<R>,
    quick_window_label: &str,
) -> Result<(), Error> {
    // 获取控制窗口位置
    let control_position = match get_window_position_and_size(app, "control") {
        Ok(pos) => pos,
        Err(_) => return Ok(()), // 控制窗口可能尚不存在
    };

    // 获取快速窗口大小
    let quick_position = match get_window_position_and_size(app, quick_window_label) {
        Ok(pos) => pos,
        Err(_) => return Ok(()), // 快速窗口可能尚不存在
    };

    // 计算新位置 - 在控制窗口上方居中
    let quick_x = control_position.x + (control_position.width - quick_position.width) / 2.0;
    let quick_y = control_position.y - quick_position.height - 10.0; // 10px间隙

    // 设置快速窗口位置
    if let Some(quick_window) = app.get_webview_window(quick_window_label) {
        let _ = quick_window.set_position(LogicalPosition::new(quick_x, quick_y));

        // 更新窗口管理器位置
        let updated_position = WindowPosition {
            x: quick_x,
            y: quick_y,
            width: quick_position.width,
            height: quick_position.height,
        };

        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut manager) = window_manager_state.0.try_lock() {
                manager.update_window_position(quick_window_label, updated_position);
            }
        }
    }

    Ok(())
}

/// 控制窗口移动后同步位置
pub fn sync_positions_after_control_moved<R: Runtime>(app: &AppHandle<R>) -> Result<(), Error> {
    // 获取活跃快速窗口（一次只显示一个）
    let active_window = get_active_window(app);

    if let Some(quick_window) = active_window {
        if quick_window.label != "control" {
            position_quick_window_above_control(app, &quick_window.label)?;
        }
    }

    Ok(())
}
