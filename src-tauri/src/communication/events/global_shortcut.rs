use tauri::{self};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutEvent, ShortcutState};

use crate::window::control;

pub fn global_shortcuts_handle(app: &tauri::AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    let open_control_window = Shortcut::new(Some(Modifiers::ALT), Code::KeyC);
    if shortcut == &open_control_window && event.state == ShortcutState::Released {
        let _ = control::show_control_window::<tauri::Wry>(app);
    }
}
