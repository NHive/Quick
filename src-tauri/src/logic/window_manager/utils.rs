// 窗口管理相关的工具函数

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Error, Manager, Runtime};

use super::models::WindowPosition;

/// 生成窗口标签
/// 基于URL生成唯一的窗口标识符
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

    let position = window.outer_position()?;
    let size = window.inner_size()?;

    Ok(WindowPosition {
        x: position.x as f64,
        y: position.y as f64,
        width: size.width as f64,
        height: size.height as f64,
    })
}
