use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Error, Manager,
};
use tokio::runtime::Runtime;

use crate::window::control;
use crate::CONTROL_WINDOW_LABEL;

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
        .on_menu_event(move |app, event| match event.id.0.as_str() {
            "quit" => {
                std::process::exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if let (MouseButton::Left, MouseButtonState::Up) = (button, button_state) {
                    if let Some(main_window) =
                        tray.app_handle().get_webview_window(CONTROL_WINDOW_LABEL)
                    {
                        main_window.show();
                    }
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
