use log::info;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// 将原来的大结构体拆分成多个独立的状态组件

// 焦点状态组件
struct FocusState {
    control_focused: bool,
    quick_windows_focused: bool,
    last_focus_change: Instant,
}

// 窗口标签组件
struct WindowLabels {
    quick_window_labels: HashSet<String>,
    quick_window_current: Option<String>,
    is_showing_quick_window: bool,
}

// 窗口状态组件
struct WindowState {
    hide_pending: bool,
    is_pinned: bool,
}

// 拖动状态组件
struct DragState {
    is_dragging: bool,
    last_move_time: Instant,
}

// 焦点恢复ID组件
struct FocusRestoreState {
    control_focus_restore_id: u64,
}

// 初始化各个状态组件的全局实例
static FOCUS: Lazy<Mutex<FocusState>> = Lazy::new(|| {
    info!("初始化焦点状态组件");
    Mutex::new(FocusState {
        control_focused: false,
        quick_windows_focused: false,
        last_focus_change: Instant::now(),
    })
});

static LABELS: Lazy<Mutex<WindowLabels>> = Lazy::new(|| {
    info!("初始化窗口标签组件");
    Mutex::new(WindowLabels {
        quick_window_labels: HashSet::new(),
        quick_window_current: None,
        is_showing_quick_window: false,
    })
});

static WINDOW_STATE: Lazy<Mutex<WindowState>> = Lazy::new(|| {
    info!("初始化窗口状态组件");
    Mutex::new(WindowState {
        hide_pending: false,
        is_pinned: false,
    })
});

static DRAG: Lazy<Mutex<DragState>> = Lazy::new(|| {
    info!("初始化拖动状态组件");
    Mutex::new(DragState {
        is_dragging: false,
        last_move_time: Instant::now(),
    })
});

static FOCUS_RESTORE: Lazy<Mutex<FocusRestoreState>> = Lazy::new(|| {
    info!("初始化焦点恢复ID组件");
    Mutex::new(FocusRestoreState {
        control_focus_restore_id: 0,
    })
});

// 公共API封装
pub struct WindowFocusState;

impl WindowFocusState {
    // 注册快速窗口标签
    pub fn register_quick_window(label: String) {
        if let Ok(mut labels) = LABELS.lock() {
            labels.quick_window_labels.insert(label);
        }
    }

    // 更新控制窗口的焦点状态
    pub fn set_control_focused(focused: bool) {
        if let Ok(mut focus) = FOCUS.lock() {
            focus.control_focused = focused;
            focus.last_focus_change = Instant::now();
        }
    }

    // 更新快速窗口的焦点状态
    pub fn set_quick_window_focused(focused: bool) {
        if let Ok(mut focus) = FOCUS.lock() {
            focus.quick_windows_focused = focused;
            focus.last_focus_change = Instant::now();
        }
    }

    // 设置隐藏等待标志
    pub fn set_hide_pending(pending: bool) {
        if let Ok(mut state) = WINDOW_STATE.lock() {
            state.hide_pending = pending;
        }
    }

    // 检查所有窗口是否都失去焦点
    pub fn all_windows_unfocused() -> bool {
        if let Ok(focus) = FOCUS.lock() {
            !focus.control_focused && !focus.quick_windows_focused
        } else {
            false // 默认安全值
        }
    }

    // 检查是否有隐藏操作等待执行
    pub fn is_hide_pending() -> bool {
        if let Ok(state) = WINDOW_STATE.lock() {
            state.hide_pending
        } else {
            false // 默认安全值
        }
    }

    // 计算自上次焦点变化经过的时间
    pub fn time_since_last_focus_change() -> Duration {
        if let Ok(focus) = FOCUS.lock() {
            Instant::now().duration_since(focus.last_focus_change)
        } else {
            Duration::from_secs(0) // 默认安全值
        }
    }

    // 计算自上次窗口移动经过的时间
    pub fn time_since_last_move() -> Duration {
        if let Ok(drag) = DRAG.lock() {
            Instant::now().duration_since(drag.last_move_time)
        } else {
            Duration::from_secs(0) // 默认安全值
        }
    }

    // 设置窗口是否处于Pin状态
    pub fn set_pinned(pinned: bool) {
        if let Ok(mut state) = WINDOW_STATE.lock() {
            state.is_pinned = pinned;
        }
    }

    // 检查窗口是否处于Pin状态
    pub fn is_pinned() -> bool {
        if let Ok(state) = WINDOW_STATE.lock() {
            state.is_pinned
        } else {
            false // 默认安全值
        }
    }

    // 设置当前活动的快速窗口标签
    pub fn set_current_quick_window(label: Option<String>) {
        if let Ok(mut labels) = LABELS.lock() {
            labels.quick_window_current = label;
        }
    }

    // 获取当前活动的快速窗口标签
    pub fn get_current_quick_window() -> Option<String> {
        if let Ok(labels) = LABELS.lock() {
            labels.quick_window_current.clone()
        } else {
            None // 默认安全值
        }
    }

    // 设置是否正在显示快速窗口
    pub fn set_showing_quick_window(showing: bool) {
        if let Ok(mut labels) = LABELS.lock() {
            labels.is_showing_quick_window = showing;
        }
    }

    // 获取并增加焦点恢复ID
    pub fn next_focus_restore_id() -> u64 {
        if let Ok(mut restore) = FOCUS_RESTORE.lock() {
            restore.control_focus_restore_id += 1;
            restore.control_focus_restore_id
        } else {
            0 // 默认安全值
        }
    }

    // 检查焦点恢复ID是否匹配
    pub fn is_latest_focus_restore_id(id: u64) -> bool {
        if let Ok(restore) = FOCUS_RESTORE.lock() {
            id == restore.control_focus_restore_id
        } else {
            false // 默认安全值
        }
    }

    // 设置窗口拖动状态
    pub fn set_dragging(dragging: bool) {
        if let Ok(mut drag) = DRAG.lock() {
            drag.is_dragging = dragging;
            if dragging {
                drag.last_move_time = Instant::now();
            }
        }
    }

    // 更新窗口移动时间
    pub fn update_move_time() {
        if let Ok(mut drag) = DRAG.lock() {
            drag.last_move_time = Instant::now();
        }
    }

    // 检查是否正在拖动窗口
    pub fn is_dragging() -> bool {
        if let Ok(drag) = DRAG.lock() {
            drag.is_dragging
        } else {
            false // 默认安全值
        }
    }
}
