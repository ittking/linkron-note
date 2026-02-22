// 窗口管理模块 - 实现跨虚拟桌面置顶
// 支持 Windows 和 macOS
#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use tauri::{App, Manager, WebviewWindow};
use serde::{Deserialize, Serialize};

/// 窗口大小
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

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
    }
}

#[cfg(target_os = "macos")]
fn set_window_on_all_desktops_macos(window: &WebviewWindow) {
    // macOS 使用 NSWindow 的 collectionBehavior
    // NSWindowCollectionBehaviorCanJoinAllSpaces = 1 << 0
    // NSWindowCollectionBehaviorFullScreenAuxiliary = 1 << 8
    // NSWindowCollectionBehaviorIgnoresCycle = 1 << 10
    // NSWindowLevel: kCGFloatingWindowLevel = 3
    use cocoa::base;
    use cocoa::foundation::NSUInteger;
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};

    if let Ok(ns_window) = window.ns_window() {
        let ns_window: *mut Object = ns_window as *mut Object;

        unsafe {
            // 设置窗口为弹出菜单级别（高于状态栏，NSPopUpMenuWindowLevel = 101）
            // 状态栏层级是 25 (NSStatusWindowLevel)
            // 菜单栏层级是 24 (NSMainMenuWindowLevel)
            let _: () = msg_send![ns_window, setLevel: 101];

            // 组合多个 collectionBehavior
            let mut behavior: NSUInteger = 0;
            // 允许窗口加入所有空间（跨桌面）
            behavior |= 1 << 0;
            // 全屏时作为辅助窗口
            behavior |= 1 << 8;
            // 忽略 Cmd+Tab 切换
            behavior |= 1 << 10;

            // 设置 collectionBehavior
            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];

            // 强制窗口保持在最前面
            let _: () = msg_send![ns_window, setHidesOnDeactivate: base::NO];
        }
    }
}

/// 获取当前操作系统类型
#[tauri::command]
pub fn get_os() -> String {
    #[cfg(target_os = "macos")]
    return "macos".to_string();

    #[cfg(windows)]
    return "windows".to_string();

    #[cfg(target_os = "linux")]
    return "linux".to_string();

    #[cfg(not(any(target_os = "macos", windows, target_os = "linux")))]
    return "unknown".to_string();
}

/// 初始化窗口管理器
/// 这是一个便捷函数，用于在 Tauri app setup 中调用
pub fn setup_window_manager(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(windows, target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window("main") {
            set_window_on_all_desktops(&window);
        }
    }

    Ok(())
}

/// 获取窗口大小
#[tauri::command]
pub fn get_window_size(window: WebviewWindow) -> Result<WindowSize, String> {
    let size = window.outer_size()
        .map_err(|e| format!("获取窗口大小失败: {}", e))?;

    Ok(WindowSize {
        width: size.width,
        height: size.height,
    })
}

/// 设置窗口大小
#[tauri::command]
pub fn set_window_size(window: WebviewWindow, size: WindowSize) -> Result<(), String> {
    // 先设置窗口可调整大小
    window.set_resizable(true)
        .map_err(|e| format!("设置窗口可调整大小失败: {}", e))?;

    // 设置窗口大小
    use tauri::LogicalSize;
    let logical_size = LogicalSize::new(size.width, size.height);

    window.set_size(logical_size)
        .map_err(|e| format!("设置窗口大小失败: {}", e))?;

    Ok(())
}
