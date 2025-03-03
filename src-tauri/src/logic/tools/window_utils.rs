// file_path: src/logic/tools/window_utils.rs
use tauri;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Runtime,
};

// 窗口工具类 - 专门处理窗口尺寸和位置转换
pub struct WindowUtils;

impl WindowUtils {
    // 获取窗口尺寸 (逻辑尺寸)
    pub fn get_window_size<R: Runtime>(
        app_handle: &AppHandle<R>,
        label: &str,
    ) -> Result<LogicalSize<f64>, tauri::Error> {
        let window = match app_handle.get_webview_window(label) {
            Some(window) => window,
            None => return Ok(LogicalSize::new(0.0, 0.0)),
        };

        let physical_size = match window.inner_size() {
            Ok(size) => size,
            Err(_) => return Ok(LogicalSize::new(0.0, 0.0)),
        };

        let scale_factor = match window.scale_factor() {
            Ok(factor) => factor,
            Err(_) => return Ok(LogicalSize::new(0.0, 0.0)),
        };

        // 将物理尺寸转换为逻辑尺寸
        Ok(LogicalSize::new(
            physical_size.width as f64 / scale_factor,
            physical_size.height as f64 / scale_factor,
        ))
    }

    // 获取窗口位置 (逻辑位置)
    pub fn get_window_position<R: Runtime>(
        app_handle: &AppHandle<R>,
        label: &str,
    ) -> Result<LogicalPosition<f64>, tauri::Error> {
        let window = match app_handle.get_webview_window(label) {
            Some(window) => window,
            None => return Ok(LogicalPosition::new(0.0, 0.0)),
        };

        let physical_position = match window.outer_position() {
            Ok(pos) => pos,
            Err(_) => return Ok(LogicalPosition::new(0.0, 0.0)),
        };

        let scale_factor = match window.scale_factor() {
            Ok(factor) => factor,
            Err(_) => return Ok(LogicalPosition::new(0.0, 0.0)),
        };

        // 将物理位置转换为逻辑位置
        Ok(LogicalPosition::new(
            physical_position.x as f64 / scale_factor,
            physical_position.y as f64 / scale_factor,
        ))
    }

    // 物理位置转逻辑位置
    pub fn physical_to_logical<R: Runtime>(
        window: &tauri::WebviewWindow<R>,
        position: PhysicalPosition<i32>,
    ) -> Result<LogicalPosition<f64>, tauri::Error> {
        let scale_factor = match window.scale_factor() {
            Ok(factor) => factor,
            Err(_) => return Ok(LogicalPosition::new(0.0, 0.0)),
        };

        Ok(LogicalPosition::new(
            position.x as f64 / scale_factor,
            position.y as f64 / scale_factor,
        ))
    }

    // 逻辑位置转物理位置
    pub fn logical_to_physical<R: Runtime>(
        window: &tauri::WebviewWindow<R>,
        position: LogicalPosition<f64>,
    ) -> Result<PhysicalPosition<i32>, tauri::Error> {
        let scale_factor = match window.scale_factor() {
            Ok(factor) => factor,
            Err(_) => return Ok(PhysicalPosition::new(0, 0)),
        };

        Ok(PhysicalPosition::new(
            (position.x * scale_factor) as i32,
            (position.y * scale_factor) as i32,
        ))
    }

    // 物理尺寸转逻辑尺寸
    pub fn physical_to_logical_size<R: Runtime>(
        window: &tauri::WebviewWindow<R>,
        size: PhysicalSize<u32>,
    ) -> Result<LogicalSize<f64>, tauri::Error> {
        let scale_factor = match window.scale_factor() {
            Ok(factor) => factor,
            Err(_) => return Ok(LogicalSize::new(0.0, 0.0)),
        };

        Ok(LogicalSize::new(
            size.width as f64 / scale_factor,
            size.height as f64 / scale_factor,
        ))
    }

    // 逻辑尺寸转物理尺寸
    pub fn logical_to_physical_size<R: Runtime>(
        window: &tauri::WebviewWindow<R>,
        size: LogicalSize<f64>,
    ) -> Result<PhysicalSize<u32>, tauri::Error> {
        let scale_factor = match window.scale_factor() {
            Ok(factor) => factor,
            Err(_) => return Ok(PhysicalSize::new(0, 0)),
        };

        Ok(PhysicalSize::new(
            (size.width * scale_factor) as u32,
            (size.height * scale_factor) as u32,
        ))
    }

    // 设置窗口位置 (考虑缩放因子)
    pub fn set_window_position<R: Runtime>(
        window: &tauri::WebviewWindow<R>,
        x: f64,
        y: f64,
    ) -> Result<(), tauri::Error> {
        let _ = window.set_position(LogicalPosition::new(x, y));
        Ok(())
    }
}
