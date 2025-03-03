// file_path: src/window/setting.rs
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

use tauri::utils::config::WebviewUrl;
use tauri::Error;
use tauri::{AppHandle, Manager, Runtime, WebviewWindowBuilder};

use crate::SETTING_WINDOW_LABEL;

pub fn create_settings_window<R: Runtime>(app: &AppHandle<R>, path: &str) -> Result<(), Error> {
    // 检查窗口是否已经存在
    if let Some(setting_window) = app.get_webview_window(SETTING_WINDOW_LABEL) {
        // 如果窗口存在，将其显示到最前
        setting_window.show()?;
        setting_window.set_focus()?;

        Ok(())
    } else {
        // 如果窗口不存在，创建新窗口
        let mut builder = WebviewWindowBuilder::new(
            app,
            SETTING_WINDOW_LABEL,
            WebviewUrl::App(format!("#/{}/{}", SETTING_WINDOW_LABEL, path).into()),
        )
        .title("")
        .fullscreen(false)
        .inner_size(750.0, 520.0)
        .center() // 设置窗口居中
        .resizable(false)
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
        new_window.set_focus()?;

        Ok(())
    }
}
