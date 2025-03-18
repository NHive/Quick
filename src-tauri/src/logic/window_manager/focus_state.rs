// file_path: src/logic/window_manager/focus_state.rs
use log::info;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::time::{Duration, Instant};

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
        LABELS.lock().quick_window_labels.insert(label);
    }

    // 更新控制窗口的焦点状态
    pub fn set_control_focused(focused: bool) {
        let mut focus = FOCUS.lock();
        focus.control_focused = focused;
        focus.last_focus_change = Instant::now();
    }

    // 更新快速窗口的焦点状态
    pub fn set_quick_window_focused(focused: bool) {
        let mut focus = FOCUS.lock();
        focus.quick_windows_focused = focused;
        focus.last_focus_change = Instant::now();
    }

    // 设置隐藏等待标志
    pub fn set_hide_pending(pending: bool) {
        WINDOW_STATE.lock().hide_pending = pending;
    }

    // 检查所有窗口是否都失去焦点
    pub fn all_windows_unfocused() -> bool {
        let focus = FOCUS.lock();
        !focus.control_focused && !focus.quick_windows_focused
    }

    // 检查是否有隐藏操作等待执行
    pub fn is_hide_pending() -> bool {
        WINDOW_STATE.lock().hide_pending
    }

    // 计算自上次焦点变化经过的时间
    pub fn time_since_last_focus_change() -> Duration {
        Instant::now().duration_since(FOCUS.lock().last_focus_change)
    }

    // 计算自上次窗口移动经过的时间
    pub fn time_since_last_move() -> Duration {
        Instant::now().duration_since(DRAG.lock().last_move_time)
    }

    // 设置窗口是否处于Pin状态
    pub fn set_pinned(pinned: bool) {
        WINDOW_STATE.lock().is_pinned = pinned;
    }

    // 检查窗口是否处于Pin状态
    pub fn is_pinned() -> bool {
        WINDOW_STATE.lock().is_pinned
    }

    // 设置当前活动的快速窗口标签
    pub fn set_current_quick_window(label: Option<String>) {
        LABELS.lock().quick_window_current = label;
    }

    // 获取当前活动的快速窗口标签
    pub fn get_current_quick_window() -> Option<String> {
        LABELS.lock().quick_window_current.clone()
    }

    // 设置是否正在显示快速窗口
    pub fn set_showing_quick_window(showing: bool) {
        LABELS.lock().is_showing_quick_window = showing;
    }

    // 获取并增加焦点恢复ID
    pub fn next_focus_restore_id() -> u64 {
        let mut restore = FOCUS_RESTORE.lock();
        restore.control_focus_restore_id += 1;
        restore.control_focus_restore_id
    }

    // 检查焦点恢复ID是否匹配
    pub fn is_latest_focus_restore_id(id: u64) -> bool {
        id == FOCUS_RESTORE.lock().control_focus_restore_id
    }

    // 设置窗口拖动状态
    pub fn set_dragging(dragging: bool) {
        let mut drag = DRAG.lock();
        drag.is_dragging = dragging;
        if dragging {
            drag.last_move_time = Instant::now();
        }
    }

    // 更新窗口移动时间
    pub fn update_move_time() {
        DRAG.lock().last_move_time = Instant::now();
    }

    // 检查是否正在拖动窗口
    pub fn is_dragging() -> bool {
        DRAG.lock().is_dragging
    }
}
