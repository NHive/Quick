use log::error;
use mouse_position::mouse_position::Mouse;
use tauri::{Runtime, WebviewWindow};

pub fn show_and_position_window<R: Runtime>(main_window: &WebviewWindow<R>) {
    // 定义保留的像素值，防止窗口被dock栏遮挡
    const BOTTOM_MARGIN: i32 = 80;

    // 获取鼠标位置
    let position = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => (x, y),
        _ => {
            error!("获取鼠标位置时出错");
            return;
        }
    };

    // 获取当前鼠标所在的显示器
    let monitor = match main_window.current_monitor() {
        Ok(Some(monitor)) => monitor,
        _ => {
            error!("无法获取当前显示器信息");
            return;
        }
    };

    let monitor_size = monitor.size(); // 获取显示器尺寸
    let scale_factor = monitor.scale_factor(); // 获取缩放因子

    let logic_monitor_size: tauri::LogicalSize<u32> = monitor_size.to_logical(scale_factor);

    // 获取窗口尺寸
    let window_size = match main_window.outer_size() {
        Ok(size) => size,
        Err(_) => {
            error!("无法获取窗口尺寸");
            return;
        }
    };

    let logic_window_size: tauri::LogicalSize<u32> = window_size.to_logical(scale_factor);

    // 计算窗口的新位置，确保窗口不会超出屏幕边界，并保留底部200像素
    let new_x = position
        .0
        .min(logic_monitor_size.width as i32 - logic_window_size.width as i32)
        .max(0);
    let new_y = position
        .1
        .min(logic_monitor_size.height as i32 - logic_window_size.height as i32 - BOTTOM_MARGIN)
        .max(0);

    // 将物理位置转换为逻辑位置
    let logical_position = tauri::LogicalPosition::new(new_x as f64, new_y as f64);

    // 将窗口移动到鼠标位置
    if let Err(e) = main_window.set_position(tauri::Position::Logical(logical_position)) {
        error!("无法设置窗口位置: {}", e);
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    // 显示窗口
    if let Err(e) = main_window.show() {
        error!("无法显示窗口: {}", e);
    }
}

pub fn show_window<R: Runtime>(main_window: &WebviewWindow<R>) {
    if let Err(e) = main_window.show() {
        error!("无法显示窗口: {}", e);
    }
}
