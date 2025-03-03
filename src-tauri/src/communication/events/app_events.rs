// file_path: src/communication/events/app_events.rs
use log::{error, info};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, RunEvent, Runtime, WindowEvent};

use crate::logic::window_manager::operations::{
    position_control_window_below_quick, sync_positions_after_control_moved,
};

use crate::logic::window_manager::models::{WindowConfig, WindowManagerState, WindowPosition};

// Simple state for tracking window focus
pub struct WindowFocusState {
    control_focused: bool,
    quick_windows_focused: bool,
    quick_window_labels: HashSet<String>,
    last_focus_change: Instant,
    hide_pending: bool,
}

impl WindowFocusState {
    pub fn new() -> Self {
        info!("Creating new WindowFocusState");
        WindowFocusState {
            control_focused: false,
            quick_windows_focused: false,
            quick_window_labels: HashSet::new(),
            last_focus_change: Instant::now(),
            hide_pending: false,
        }
    }

    // Register a quick window label
    pub fn register_quick_window(&mut self, label: String) {
        self.quick_window_labels.insert(label);
    }

    // Update focus state for control window
    pub fn set_control_focused(&mut self, focused: bool) {
        self.control_focused = focused;
        self.last_focus_change = Instant::now();
    }

    // Update focus state for quick windows
    pub fn set_quick_window_focused(&mut self, focused: bool) {
        self.quick_windows_focused = focused;
        self.last_focus_change = Instant::now();
    }

    // Set hide pending flag
    pub fn set_hide_pending(&mut self, pending: bool) {
        self.hide_pending = pending;
    }

    // Check if all windows are unfocused
    pub fn all_windows_unfocused(&self) -> bool {
        !self.control_focused && !self.quick_windows_focused
    }

    // Get quick window labels
    pub fn get_quick_window_labels(&self) -> &HashSet<String> {
        &self.quick_window_labels
    }

    // Check if hide is pending
    pub fn is_hide_pending(&self) -> bool {
        self.hide_pending
    }

    // Check if time since last focus change is enough
    pub fn time_since_last_focus_change(&self) -> Duration {
        Instant::now().duration_since(self.last_focus_change)
    }
}

// Main event handler for application events
pub fn handle_app_events<R: Runtime>(app_handle: &AppHandle<R>, event: RunEvent) {
    match event {
        RunEvent::WindowEvent { label, event, .. } => {
            let app_handle_arc = Arc::new(app_handle.clone());

            match event {
                WindowEvent::Moved(position) => {
                    handle_window_moved(&app_handle_arc, &label, position);
                }
                WindowEvent::Resized(size) => {
                    handle_window_resized(&app_handle_arc, &label, size);
                }
                WindowEvent::Focused(focused) => {
                    handle_window_focused(&app_handle_arc, &label, focused);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

// Initialize event system and state
pub fn init_event_system<R: Runtime>(app_handle: &AppHandle<R>) {
    info!("Initializing event system");
    // Create and register focus state
    let focus_state = WindowFocusState::new();
    app_handle.manage(Arc::new(Mutex::new(focus_state)));
}

// Handle window moved events
fn handle_window_moved<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    _position: PhysicalPosition<i32>,
) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        if label_clone == "control" {
            // When control window is moved, sync positions with active quick window
            let _ = sync_positions_after_control_moved(&app_handle_clone);
        } else if label_clone.starts_with("quick_") {
            // When quick window is moved, update control window position
            let _ = position_control_window_below_quick(&app_handle_clone, &label_clone);
        }
    });
}

// Handle window resized events
fn handle_window_resized<R: Runtime>(
    app_handle: &Arc<AppHandle<R>>,
    label: &str,
    _size: PhysicalSize<u32>,
) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        if label_clone == "control" {
            // When control window is resized, sync positions with active quick window
            let _ = sync_positions_after_control_moved(&app_handle_clone);
        } else if label_clone.starts_with("quick_") {
            // When quick window is resized, update control window position
            let _ = position_control_window_below_quick(&app_handle_clone, &label_clone);
        }
    });
}

// Handle window focused events
fn handle_window_focused<R: Runtime>(app_handle: &Arc<AppHandle<R>>, label: &str, focused: bool) {
    let app_handle_clone = app_handle.clone();
    let label_clone = label.to_string();

    tauri::async_runtime::spawn(async move {
        // Update focus state
        if let Some(focus_state_arc) = app_handle_clone.try_state::<Arc<Mutex<WindowFocusState>>>()
        {
            let mut focus_state_updated = false;

            {
                if let Ok(mut focus_state) = focus_state_arc.lock() {
                    // Register quick window if it's new
                    if label_clone.starts_with("quick_") {
                        focus_state.register_quick_window(label_clone.clone());
                    }

                    // Update focus state based on window type
                    if label_clone == "control" {
                        focus_state.set_control_focused(focused);
                    } else if label_clone.starts_with("quick_") {
                        focus_state.set_quick_window_focused(focused);
                    }

                    // Set hide pending flag if window loses focus
                    if !focused {
                        focus_state.set_hide_pending(true);
                    } else {
                        focus_state.set_hide_pending(false);
                    }

                    // Check if we need to verify window visibility status
                    focus_state_updated = !focused;
                }
            }

            // If a window has lost focus, schedule a check to see if all windows should be hidden
            if focus_state_updated {
                tokio::time::sleep(Duration::from_millis(150)).await;

                // Check if all windows are unfocused and hide is pending
                let should_hide = {
                    if let Ok(focus_state) = focus_state_arc.lock() {
                        focus_state.time_since_last_focus_change() >= Duration::from_millis(100)
                            && focus_state.is_hide_pending()
                            && focus_state.all_windows_unfocused()
                    } else {
                        false
                    }
                };

                if should_hide {
                    // Hide all managed windows
                    hide_all_managed_windows(&app_handle_clone).await;

                    // Reset hide pending flag
                    if let Ok(mut focus_state) = focus_state_arc.lock() {
                        focus_state.set_hide_pending(false);
                    }
                }
            }
        }
    });
}

// Hide all managed windows when they all lose focus
async fn hide_all_managed_windows<R: Runtime>(app_handle: &AppHandle<R>) {
    info!("Hiding all managed windows");

    // Hide control window
    if let Some(window) = app_handle.get_webview_window("control") {
        let _ = window.hide();
    }

    // Hide all quick windows
    if let Some(focus_state_arc) = app_handle.try_state::<Arc<Mutex<WindowFocusState>>>() {
        if let Ok(focus_state) = focus_state_arc.lock() {
            for label in focus_state.get_quick_window_labels() {
                if let Some(window) = app_handle.get_webview_window(label) {
                    let _ = window.hide();
                }
            }
        }
    }
}
