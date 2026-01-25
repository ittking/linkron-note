// Windows 窗口管理模块 - 实现跨虚拟桌面置顶

use tauri::WebviewWindow;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOOLWINDOW};

/// 设置窗口跨所有虚拟桌面显示
/// 通过设置 WS_EX_TOOLWINDOW 样式，使窗口在所有虚拟桌面上都可见
pub fn set_window_on_all_desktops(window: &WebviewWindow) {
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
        
        println!("窗口已设置为跨所有虚拟桌面显示");
    }
}