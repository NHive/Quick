use log::error;

use mouse_position::mouse_position::Mouse;
use tauri::{Runtime, WebviewWindow};

pub fn show_and_position_window<R: Runtime>(main_window: &WebviewWindow<R>) {
    // 获取鼠标位置
    let (x, y) = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => (x, y),
        _ => {
            eprintln!("获取鼠标位置时出错");
            return;
        }
    };

    // 获取显示器信息
    let monitors = match main_window.available_monitors() {
        Ok(monitors) => monitors,
        Err(_) => {
            eprintln!("无法获取显示器信息");
            return;
        }
    };

    // 找到鼠标所在的显示器
    let monitor = monitors.iter().find(|monitor| {
        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        let logic_monitor_position: tauri::PhysicalPosition<i32> = *monitor_position;
        let logic_monitor_size: tauri::PhysicalSize<u32> = *monitor_size;

        x >= logic_monitor_position.x
            && y >= logic_monitor_position.y
            && x < logic_monitor_position.x + logic_monitor_size.width as i32
            && y < logic_monitor_position.y + logic_monitor_size.height as i32
    });

    let monitor = match monitor {
        Some(monitor) => monitor,
        None => {
            eprintln!("无法找到鼠标所在的显示器");
            return;
        }
    };

    // 获取窗口尺寸
    let window_size = match main_window.outer_size() {
        Ok(size) => size,
        Err(_) => {
            eprintln!("无法获取窗口尺寸");
            return;
        }
    };

    // 计算新窗口位置
    let new_x = x
        .max(monitor.position().x)
        .min(monitor.position().x + monitor.size().width as i32 - window_size.width as i32);
    let new_y = y
        .max(monitor.position().y)
        .min(monitor.position().y + monitor.size().height as i32 - window_size.height as i32);

    let physical_position = tauri::PhysicalPosition::new(new_x, new_y);

    // 设置窗口位置并显示
    if let Err(_) = main_window.set_position(tauri::Position::Physical(physical_position)) {
        eprintln!("无法设置窗口位置");
    }

    if let Err(_) = main_window.show() {
        eprintln!("无法显示窗口");
    }

    if let Err(_) = main_window.set_focus() {
        eprintln!("无法获取窗口焦点");
    }
}

pub fn show_window<R: Runtime>(main_window: &WebviewWindow<R>) {
    if let Err(e) = main_window.show() {
        error!("无法显示窗口: {}", e);
    }
}
