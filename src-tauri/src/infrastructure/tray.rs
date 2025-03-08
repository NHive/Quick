// file_path: src/infrastructure/tray.rs
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Error,
};

use crate::window::control;

// 托盘菜单
pub fn menu(app: &AppHandle) -> Result<TrayIcon, Error> {
    let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app).items(&[&quit_item]).build()?;

    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Quick - Newbee")
        .icon_as_template(true)
        .on_menu_event(move |_app, event| {
            if event.id.0.as_str() == "quit" {
                std::process::exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if let (MouseButton::Left, MouseButtonState::Up) = (button, button_state) {
                    let _ = tauri::async_runtime::block_on(control::show_control_window::<tauri::Wry>(
                        tray.app_handle(),
                    ));
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
