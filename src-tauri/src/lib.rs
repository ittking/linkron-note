// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod mouse;

#[cfg(windows)]
mod window_manager;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Windows 平台下设置窗口跨虚拟桌面置顶
            #[cfg(windows)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window_manager::set_window_on_all_desktops(&window);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            mouse::start_mouse_listener,
            mouse::stop_mouse_listener,
            mouse::is_mouse_listener_running
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
