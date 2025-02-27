use sha2::{Digest, Sha256};

use tauri::utils::config::WebviewUrl;
use tauri::{AppHandle, Error, Manager, Runtime, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

pub fn generate_window_label(url: &str) -> String {
    // 计算 URL 的 SHA256 哈希值
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();

    // 将哈希值转换为十六进制字符串
    let hex_string = format!("{:x}", result);

    // 将十六进制字符串转换为字母字符串
    let mut label = String::new();
    for c in hex_string.chars() {
        if c.is_ascii_digit() {
            // 如果是数字，转换为对应的字母（0-9 -> a-j）
            label.push((c as u8 - b'0' + b'a') as char);
        } else {
            // 如果已经是字母，直接添加
            label.push(c);
        }
    }

    // 截取前 20 个字符作为标签（可选）
    label.chars().take(20).collect()
}

pub fn create_quick_window<R: Runtime>(app: &AppHandle<R>, url: &str) -> Result<(), Error> {
    // 生成窗口标签
    let label = generate_window_label(url);

    // 检查窗口是否已经存在
    if let Some(setting_window) = app.get_webview_window(&label) {
        // 如果窗口存在，将其显示到最前
        setting_window.show()?;
        setting_window.set_focus()?;

        Ok(())
    } else {
        // 如果窗口不存在，创建新窗口
        let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title("")
            .fullscreen(false)
            .inner_size(1280.0, 768.0)
            .center() // 设置窗口居中
            .resizable(true)
            .visible(false)
            .skip_taskbar(false)
            .always_on_top(true);
        // .menu(Menu::new());

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

        let new_window = builder.build().unwrap();

        new_window.show()?;

        Ok(())
    }
}
