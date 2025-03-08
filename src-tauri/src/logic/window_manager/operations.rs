// file_path: src/logic/window_manager/operations.rs
// 窗口操作的API实现

use log::{debug, info};
use std::sync::{Arc, RwLock};
use tauri::utils::config::WebviewUrl;
use tauri::{AppHandle, Error, Manager, Runtime, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

use super::models::{WindowConfig, WindowInfo, WindowManagerState, WindowStatus};
use super::utils::{
    apply_position_to_window, generate_window_label, get_quick_common_position,
    get_window_position_and_size, set_window_position,
};
use crate::communication::events::app_events::WindowFocusState;

/// 配置窗口列表
pub fn configure_windows<R: Runtime>(
    app: &AppHandle<R>,
    configs: Vec<WindowConfig>,
) -> Result<(), Error> {
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
            window_manager.set_window_configs(configs);
            return Ok(());
        }
    }

    Err(Error::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        "无法访问窗口管理器",
    )))
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

/// 显示前一个活跃窗口,如果前一个活跃窗口不存在，则显示已注册的第一个窗口
pub fn show_previous_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), Error> {
    // 获取前一个活跃窗口的标签
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

    // 如果前一个活跃窗口不存在，则获取第一个非控制窗口
    let window_label = if previous_label.is_none() {
        // 获取所有窗口
        let windows = get_all_windows(app);

        // 找到第一个非控制窗口
        let first_window = windows
            .into_iter()
            .find(|w| w.label != "control")
            .map(|w| w.label);

        info!(
            "前一个活跃窗口不存在，使用第一个可用窗口: {:?}",
            first_window
        );
        first_window
    } else {
        info!("显示前一个活跃窗口: {:?}", previous_label);
        previous_label
    };

    // 如果有可用窗口，切换到该窗口
    if let Some(label) = window_label {
        switch_to_window(app, &label)?;
        position_control_window_below_quick(app, &label)?;
    }

    Ok(())
}

/// 创建或获取窗口
pub fn get_or_create_window<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
) -> Result<tauri::WebviewWindow<R>, Error> {
    // 检查窗口是否已存在
    if let Some(window) = app.get_webview_window(label) {
        return Ok(window);
    }

    // 获取窗口信息并创建
    let window_info = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(window_manager) = window_manager_state.0.try_lock() {
                window_manager.get_window_info(label)
            } else {
                None
            }
        } else {
            None
        }
    };

    let (url, title) = if let Some(info) = window_info {
        (info.url, info.title)
    } else {
        return Err(Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("窗口 {} 的配置信息未找到", label),
        )));
    };

    // 创建新窗口
    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title(&title)
        .fullscreen(false)
        .inner_size(1200.0, 800.0)
        .resizable(true)
        .visible(false)
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
        builder = builder.transparent(false);
    }

    let window = builder.build()?;

    // 标记窗口为已加载
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
            window_manager.mark_window_loaded(label);
        }
    }

    Ok(window)
}

/// 将控制窗口定位在快速窗口下方
pub fn position_control_window_below_quick<R: Runtime>(
    app: &AppHandle<R>,
    quick_window_label: &str,
) -> Result<(), Error> {
    // 获取控制窗口
    let control_window = match app.get_webview_window("control") {
        Some(window) => window,
        None => return Ok(()), // 控制窗口可能尚不存在
    };

    let position = get_quick_common_position(app);

    let quick_position = if let Some(position) = position {
        position
    } else {
        get_window_position_and_size(app, quick_window_label)?
    };

    // 获取控制窗口大小
    let control_position = match get_window_position_and_size(app, "control") {
        Ok(pos) => pos,
        Err(_) => return Ok(()), // 控制窗口可能尚不存在
    };

    // 计算新位置 - 在快速窗口下方居中
    let control_x = quick_position.x + (quick_position.width - control_position.width) / 2.0;
    let control_y = quick_position.y + quick_position.height + 5.0; // 间隙

    debug!("控制窗口位置计算: control_x={}, control_y={}, 基于快速窗口: x={}, y={}, width={}, height={}",
        control_x, control_y, quick_position.x, quick_position.y, quick_position.width, quick_position.height);

    // 更新窗口管理器中的位置信息，检查是否被允许更新
    let can_update = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
                window_manager.can_update(quick_window_label)
            } else {
                false
            }
        } else {
            false
        }
    };

    // 如果允许更新，则设置控制窗口位置
    if can_update {
        set_window_position(&control_window, control_x, control_y)?;
    }

    // 显示控制窗口
    control_window.show()?;

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
    let quick_y = control_position.y - quick_position.height - 5.0; // 间隙

    // 设置快速窗口位置
    if let Some(quick_window) = app.get_webview_window(quick_window_label) {
        set_window_position(&quick_window, quick_x, quick_y)?;
    }

    Ok(())
}

/// 控制窗口移动后同步位置
pub fn sync_positions_after_control_moved<R: Runtime>(app: &AppHandle<R>) -> Result<(), Error> {
    // 获取活跃快速窗口（一次只显示一个）
    let active_window = get_active_window(app);

    // 检查是否允许更新
    let can_update = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
                window_manager.can_update("control")
            } else {
                false
            }
        } else {
            false
        }
    };

    if can_update {
        if let Some(quick_window) = active_window {
            if quick_window.label != "control" {
                // 同步快速窗口位置
                position_quick_window_above_control(app, &quick_window.label)?;
            }
        }
    }

    Ok(())
}

/// 清空窗口缓存
pub fn clear_window_cache<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    let window = get_or_create_window(app, label)?;
    window.clear_all_browsing_data()?;
    Ok(())
}

/// 关闭指定窗口
pub fn close_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 首先检查窗口是否存在
    let window = app.get_webview_window(label).ok_or_else(|| {
        Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("窗口 {} 未找到", label),
        ))
    })?;

    // 关闭窗口
    window.close()?;

    // 如果关闭的是活跃窗口，清空窗口管理器中的活跃窗口状态
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
            if let Some(mut active_window) = window_manager.get_active_window() {
                if active_window.label == label {
                    active_window.loaded = false; // 如果窗口关闭，标记为未加载
                    window_manager.clear_active_window();
                }
            }
        }
    }

    Ok(())
}

/// 隐藏指定窗口
pub fn hide_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 首先检查窗口是否存在
    let window = app.get_webview_window(label).ok_or_else(|| {
        Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("窗口 {} 未找到", label),
        ))
    })?;

    // 隐藏窗口
    window.hide()?;

    // 更新窗口管理器中的窗口状态
    if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
        if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
            if let Some(window) = window_manager.get_window_info(label) {
                if window.status == WindowStatus::Foreground {
                    // 如果隐藏的是前台窗口，则尝试切换到前一个活跃窗口
                    if let Some(previous_window) = window_manager.get_previous_active_window() {
                        window_manager.switch_to_window(&previous_window.label);
                    } else {
                        window_manager.clear_active_window();
                    }
                }
            }
        }
    }

    Ok(())
}

/// 切换到指定窗口
pub fn switch_to_window<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), Error> {
    // 获取窗口信息、要隐藏的窗口和共享位置
    let (window_info, to_hide, quick_common_position) = {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
                let window_info = window_manager.get_window_info(label);

                // 如果窗口存在，切换到它
                if window_info.is_some() {
                    window_manager.switch_to_window(label);
                }

                // 获取要隐藏的窗口
                let to_hide = window_manager
                    .get_previous_active_window()
                    .filter(|w| w.label != label)
                    .map(|w| w.label);

                // 获取快速窗口共享位置
                let quick_common_position = window_manager.get_quick_common_position();

                (window_info, to_hide, quick_common_position)
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        }
    };

    if window_info.is_some() {
        // 获取或创建窗口
        let window = get_or_create_window(app, label)?;

        log::info!("切换到窗口 {}", label);

        // 如果有共享位置且不是控制窗口，应用共享位置
        if label != "control" {
            if let Some(position) = &quick_common_position {
                info!(
                    "应用共享位置到窗口 {}: x={}, y={}, width={}, height={}",
                    label, position.x, position.y, position.width, position.height
                );

                apply_position_to_window(&window, position)?;
            }
        }

        // 将控制窗口定位在此快速窗口下方
        position_control_window_below_quick(app, label)?;

        // 隐藏其他窗口
        if let Some(hide_label) = to_hide {
            if let Some(other_window) = app.get_webview_window(&hide_label) {
                other_window.hide()?;
            }
        }

        window.show()?;
        window.set_focus()?;

        if let Some(focus_state_arc) = app.try_state::<Arc<RwLock<WindowFocusState>>>() {
            if let Ok(mut focus_state) = focus_state_arc.write() {
                focus_state.set_current_quick_window(Some(label.to_string()));
                focus_state.set_showing_quick_window(true);
            }
        }
    } else {
        // 窗口不存在于管理器中，无法切换
        return Err(Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("窗口 {} 不存在", label),
        )));
    }

    Ok(())
}

// 为了兼容性实现
pub fn create_or_switch_window<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
    title: &str,
) -> Result<(), Error> {
    // 从URL生成窗口标签
    let label = generate_window_label(url, title);

    // 如果管理器中没有此窗口，添加配置
    {
        if let Some(window_manager_state) = app.try_state::<WindowManagerState>() {
            if let Ok(mut window_manager) = window_manager_state.0.try_lock() {
                if window_manager.get_window_info(&label).is_none() {
                    window_manager.add_window(&label, title, url);
                }
            }
        }
    }

    // 切换到窗口
    switch_to_window(app, &label)
}
