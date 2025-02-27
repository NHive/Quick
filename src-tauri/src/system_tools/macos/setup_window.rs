use objc::runtime::{Object, NO, YES};
use objc::{class, msg_send, sel, sel_impl};
use tauri::WebviewWindow;
use tauri::Wry;
use tauri_nspanel::WebviewWindowExt;

const NSWINDOW_STYLE_MASK_NONACTIVATING_PANEL: u32 = 1 << 7;
const NSWINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u32 = 1 << 0;
const NSWINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY: u32 = 1 << 8;

pub fn setup_macos_window(
    main_window: &WebviewWindow<Wry>,
    _key_window: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = main_window
        .to_panel()
        .map_err(|_| "Failed to hide window")?;

    let ns_window: *mut Object = main_window.ns_window().unwrap() as *mut _;

    unsafe {
        let _: () = msg_send![ns_window, setHasShadow: YES];
        let _: () = msg_send![ns_window, setOpaque: NO];
        let current_style_mask: u32 = msg_send![ns_window, styleMask];

        let new_style_mask = current_style_mask | NSWINDOW_STYLE_MASK_NONACTIVATING_PANEL;
        let _: () = msg_send![ns_window, setStyleMask: new_style_mask];

        let clear_color: *mut Object = msg_send![class!(NSColor), clearColor];
        let _: () = msg_send![ns_window, setBackgroundColor: clear_color];

        // 设置圆角
        let content_view: *mut Object = msg_send![ns_window, contentView];
        let _: () = msg_send![content_view, setWantsLayer: YES];
        let layer: *mut Object = msg_send![content_view, layer];
        let _: () = msg_send![layer, setCornerRadius: 15.0]; // 设置圆角半径
        let _: () = msg_send![layer, setMasksToBounds: YES];

        // 设置窗口行为
        let collection_behavior: u32 = NSWINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES
            | NSWINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY;
        let _: () = msg_send![ns_window, setCollectionBehavior: collection_behavior];
    }

    log::info!("macOS window setup completed");
    Ok(())
}
