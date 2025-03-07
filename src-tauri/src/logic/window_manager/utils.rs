// file_path: src/logic/window_manager/utils.rs
// 窗口管理相关的工具函数

use log::debug;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Error, Manager, Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
use tauri::LogicalPosition;
#[cfg(target_os = "macos")]
use tauri::LogicalSize;

#[cfg(not(target_os = "macos"))]
use log::warn;
#[cfg(not(target_os = "macos"))]
use tauri::PhysicalPosition;
#[cfg(not(target_os = "macos"))]
use tauri::PhysicalSize;

use super::models::WindowPosition;

/// 生成窗口标签
/// 基于URL生成唯一的窗口标识符
pub fn generate_window_label(url: &str, title: &str) -> String {
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

    format!(
        "quick_{}_{}",
        title,
        label.chars().take(20).collect::<String>()
    )
}

/// 获取窗口位置和大小
pub fn get_window_position_and_size<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
) -> Result<WindowPosition, Error> {
    let window = app.get_webview_window(label).ok_or_else(|| {
        Error::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("窗口 {} 未找到", label),
        ))
    })?;

    #[cfg(target_os = "macos")]
    {
        // macOS 使用逻辑坐标
        let position = window
            .outer_position()?
            .to_logical::<f64>(window.scale_factor()?);

        let size = window
            .inner_size()?
            .to_logical::<f64>(window.scale_factor()?);

        Ok(WindowPosition {
            x: f64::from(position.x),
            y: f64::from(position.y),
            width: f64::from(size.width),
            height: f64::from(size.height),
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows 和其他系统使用物理坐标
        let position = window.outer_position()?.to_logical(window.scale_factor()?);
        let size = window.inner_size()?.to_logical(window.scale_factor()?);

        Ok(WindowPosition {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        })
    }
}

/// 设置窗口位置，处理不同平台的差异
pub fn set_window_position<R: Runtime>(
    window: &WebviewWindow<R>,
    x: f64,
    y: f64,
) -> Result<(), Error> {
    debug!("设置窗口位置: x={}, y={}", x, y);

    #[cfg(target_os = "macos")]
    {
        window.set_position(LogicalPosition::new(x, y))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(scale_factor) = window.scale_factor() {
            let physical_x = x * scale_factor;
            let physical_y = y * scale_factor;

            debug!(
                "设置窗口物理位置: physical_x={}, physical_y={}, 缩放因子: {}",
                physical_x, physical_y, scale_factor
            );

            window.set_position(PhysicalPosition::new(physical_x as i32, physical_y as i32))?;
        } else {
            // 降级方案，如果无法获取缩放因子
            warn!("无法获取缩放因子，使用未缩放坐标");
            window.set_position(PhysicalPosition::new(x as i32, y as i32))?;
        }
    }

    Ok(())
}

/// 设置窗口大小，处理不同平台的差异
pub fn set_window_size<R: Runtime>(
    window: &WebviewWindow<R>,
    width: f64,
    height: f64,
) -> Result<(), Error> {
    debug!("设置窗口大小: width={}, height={}", width, height);

    #[cfg(target_os = "macos")]
    {
        window.set_size(LogicalSize::new(width, height))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(scale_factor) = window.scale_factor() {
            let physical_width = width * scale_factor;
            let physical_height = height * scale_factor;

            debug!(
                "设置窗口物理大小: physical_width={}, physical_height={}, 缩放因子: {}",
                physical_width, physical_height, scale_factor
            );

            window.set_size(PhysicalSize::new(
                physical_width as u32,
                physical_height as u32,
            ))?;
        } else {
            // 降级方案，如果无法获取缩放因子
            warn!("无法获取缩放因子，使用未缩放尺寸");
            window.set_size(PhysicalSize::new(width as u32, height as u32))?;
        }
    }

    Ok(())
}

/// 同时设置窗口位置和大小
pub fn set_window_position_and_size<R: Runtime>(
    window: &WebviewWindow<R>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), Error> {
    set_window_position(window, x, y)?;
    set_window_size(window, width, height)?;
    Ok(())
}

/// 应用位置信息到窗口
pub fn apply_position_to_window<R: Runtime>(
    window: &WebviewWindow<R>,
    position: &super::models::WindowPosition,
) -> Result<(), Error> {
    debug!(
        "应用位置到窗口: x={}, y={}, width={}, height={}",
        position.x, position.y, position.width, position.height
    );

    set_window_position_and_size(
        window,
        position.x,
        position.y,
        position.width,
        position.height,
    )
}
