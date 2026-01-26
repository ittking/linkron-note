// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod mouse;
mod autostart;
mod filesystem;
mod terminal;
mod database;

#[cfg(any(windows, target_os = "macos"))]
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
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(any(windows, target_os = "macos"))]
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
            mouse::is_mouse_listener_running,
            autostart::set_autostart,
            autostart::is_autostart_enabled,
            filesystem::check_directory_exists,
            filesystem::create_directory,
            terminal::create_pty_session,
            terminal::write_to_pty,
            terminal::resize_pty,
            terminal::close_pty_session,
            terminal::get_current_directory,
            database::init_database,
            database::get_all_notes,
            database::get_note,
            database::create_note,
            database::update_note,
            database::delete_note,
            database::search_notes,
            database::migrate_from_json,
            database::get_all_tags,
            database::get_tags_with_stats,
            database::get_note_tags,
            database::add_tag_to_note,
            database::remove_tag_from_note,
            database::delete_tag,
            database::get_notes_by_tag,
            database::search_tags
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
