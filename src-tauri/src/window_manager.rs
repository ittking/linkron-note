// 窗口管理模块 - 实现跨虚拟桌面置顶
// 支持 Windows 和 macOS

use tauri::WebviewWindow;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOOLWINDOW};

/// 设置窗口跨所有虚拟桌面显示
pub fn set_window_on_all_desktops(window: &WebviewWindow) {
    #[cfg(windows)]
    {
        set_window_on_all_desktops_windows(window);
    }
    
    #[cfg(target_os = "macos")]
    {
        set_window_on_all_desktops_macos(window);
    }
    
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        println!("当前平台不支持跨虚拟桌面置顶");
    }
}

#[cfg(windows)]
fn set_window_on_all_desktops_windows(window: &WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        let hwnd = HWND(hwnd.0);
        
        // 获取当前扩展样式
        let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
        
        // 添加 WS_EX_TOOLWINDOW 样式
        let new_style = ex_style | WS_EX_TOOLWINDOW.0 as isize;
        
        // 设置新的扩展样式
        unsafe {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
        }
        
        println!("窗口已设置为跨所有虚拟桌面显示 (Windows)");
    }
}

#[cfg(target_os = "macos")]
fn set_window_on_all_desktops_macos(window: &WebviewWindow) {
    // macOS 使用 NSWindow 的 collectionBehavior
    // 设置 NSWindowCollectionBehaviorCanJoinAllSpaces 使窗口在所有空间显示
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehaviorCanJoinAllSpaces};
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    
    if let Ok(ns_window) = window.ns_window() {
        unsafe {
            let ns_window: *mut Object = ns_window as *mut Object;
            let _: () = msg_send![ns_window, setCollectionBehavior: NSWindowCollectionBehaviorCanJoinAllSpaces];
        }
        println!("窗口已设置为跨所有虚拟桌面显示 (macOS)");
    }
}