// file_path: src/window/control.rs
// use super::quick_window::show_previous_window;
use crate::CONTROL_WINDOW_LABEL;
use tauri::{self, Error, Manager, Runtime};

pub fn show_control_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Error> {
    let Some(control_window) = app.get_webview_window(CONTROL_WINDOW_LABEL) else {
        return Ok(());
    };

    // show_previous_window(app)?;
    control_window.show()?;
    control_window.set_focus()?;
    Ok(())
}
