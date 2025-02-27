use crate::error::MainError;
use active_win_pos_rs::get_active_window;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: Option<String>,
    pub process_path: Option<PathBuf>,
    pub app_name: Option<String>,
    pub window_id: Option<String>,
    pub process_id: Option<u64>,
    pub position: Option<WindowPosition>,
}

impl WindowInfo {
    pub fn new() -> Result<Self, MainError> {
        let data = get_active_window();
        match data {
            Ok(data) => Ok(WindowInfo {
                title: Some(data.title),
                process_path: Some(data.process_path),
                app_name: Some(data.app_name),
                window_id: Some(data.window_id),
                process_id: Some(data.process_id),
                position: Some(WindowPosition {
                    x: data.position.x,
                    y: data.position.y,
                    width: data.position.width,
                    height: data.position.height,
                }),
            }),
            Err(_) => Err(MainError::Other(anyhow::Error::msg(
                "Failed to get active window information",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_active_window() {
        let test_data = WindowInfo::new();
        println!("{:?}", test_data);
    }
}
