// file_path: src/logic/window_manager/utils.rs
// 窗口管理相关的工具函数

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Error, Manager, Runtime};

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
