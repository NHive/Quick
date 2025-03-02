use crate::CONTROL_WINDOW_LABEL;
use tauri::{self, Error, Manager, Runtime};

pub fn show_control_window<R: Runtime>(app: &tauri::AppHandle) -> Result<(), Error> {
    let Some(control_window) = app.get_webview_window(CONTROL_WINDOW_LABEL) else {
        return Ok(());
    };

    control_window.show()?;
    Ok(())
}
