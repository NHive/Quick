// file_path: src/logic/platform/macos/utils.rs
use core_foundation::base::TCFType;
use core_foundation_sys::dictionary::CFDictionaryRef;
use core_foundation_sys::string::kCFStringEncodingUTF8;
use std::ffi::CString;
use std::ptr;

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
}

// 只检查权限
#[allow(unused)]
pub fn check_accessibility_permissions() -> bool {
    check_accessibility_permissions_with_prompt(false)
}

// 申请权限
#[allow(unused)]
pub fn request_accessibility_permissions() -> bool {
    check_accessibility_permissions_with_prompt(true)
}

// 内部函数，统一处理检查和申请
fn check_accessibility_permissions_with_prompt(show_prompt: bool) -> bool {
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionaryCreate;
    use core_foundation::string::CFStringCreateWithCString;

    let key = CString::new("AXTrustedCheckOptionPrompt").unwrap();
    let value = if show_prompt {
        CFBoolean::true_value().as_CFType()
    } else {
        CFBoolean::false_value().as_CFType()
    };

    let keys =
        [unsafe { CFStringCreateWithCString(ptr::null(), key.as_ptr(), kCFStringEncodingUTF8) }];
    let values = [value];

    let options = unsafe {
        CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr() as *const *const _,
            values.as_ptr() as *const *const _,
            keys.len() as _,
            &core_foundation_sys::dictionary::kCFTypeDictionaryKeyCallBacks,
            &core_foundation_sys::dictionary::kCFTypeDictionaryValueCallBacks,
        )
    };

    unsafe { AXIsProcessTrustedWithOptions(options) }
}

