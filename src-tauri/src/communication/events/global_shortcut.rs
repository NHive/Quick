// file_path: src/communication/events/global_shortcut.rs
use tauri::{self, App};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent, ShortcutState,
};

use crate::window::control;

pub fn global_shortcuts_handle(app: &tauri::AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    let open_control_window = Shortcut::new(Some(Modifiers::ALT), Code::KeyC);
    if shortcut == &open_control_window && event.state == ShortcutState::Released {
        let _ = control::toggle_control_window::<tauri::Wry>(app);
    }
}

/// 注册全局快捷键
pub fn register_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut()
        .register(open_control_window_shortcut())?;
    Ok(())
}

/// 打开控制窗口的快捷键
fn open_control_window_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT), Code::KeyC)
}
