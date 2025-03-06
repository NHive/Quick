// file_path: src/window/control.rs
// use super::quick_window::show_previous_window;
use tauri::{self, Error, Manager, Runtime};

use crate::logic::window_manager::operations;
use crate::CONTROL_WINDOW_LABEL;

pub fn show_control_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Error> {
    let Some(control_window) = app.get_webview_window(CONTROL_WINDOW_LABEL) else {
        return Ok(());
    };

    control_window.show()?;
    control_window.set_focus()?;
    operations::show_previous_window(app)?;
    Ok(())
}

pub fn hide_control_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Error> {
    let Some(control_window) = app.get_webview_window(CONTROL_WINDOW_LABEL) else {
        return Ok(());
    };
    control_window.hide()?;
    operations::hide_quick_window(app)?;
    Ok(())
}

pub fn toggle_control_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Error> {
    let Some(control_window) = app.get_webview_window(CONTROL_WINDOW_LABEL) else {
        return Ok(());
    };

    if control_window.is_visible()? {
        control_window.hide()?;
        operations::hide_quick_window(app)?;
    } else {
        control_window.show()?;
        control_window.set_focus()?;
        operations::show_previous_window(app)?;
    }

    Ok(())
}
