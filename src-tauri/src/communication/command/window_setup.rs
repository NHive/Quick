// file_path: src/communication/command/window_setup.rs
use crate::logic::window_manager::focus_state::WindowFocusState;

#[tauri::command]
pub async fn get_window_pin() -> Result<bool, String> {
    let is_pinned = WindowFocusState::is_pinned();
    Ok(is_pinned)
}

#[tauri::command]
pub async fn set_window_pin(pin: bool) -> Result<(), String> {
    WindowFocusState::set_pinned(pin);
    Ok(())
}
