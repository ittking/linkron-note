// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api_sync;
mod autostart;
mod database;
mod file_reader;
mod filesystem;
mod note;
mod protocol;
mod tag;
mod todo;
mod tray;
mod web_scraper;
mod window_manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .register_uri_scheme_protocol("linkron", protocol::iterm_protocol_handler)
        .setup(|app| {
            window_manager::setup_window_manager(app)?;

            // 创建系统托盘/Dock菜单
            #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
            {
                let app_handle = app.handle();
                if let Err(e) = tray::create_system_menu(&app_handle) {
                    eprintln!("Failed to create system menu: {}", e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            autostart::set_autostart,
            autostart::is_autostart_enabled,
            filesystem::check_directory_exists,
            filesystem::create_directory,
            filesystem::save_image,
            filesystem::save_file,
            filesystem::get_local_path_from_protocol,
            filesystem::delete_resource_by_url,
            database::init_database,
            database::get_all_notes,
            database::get_note,
            database::create_note,
            database::update_note,
            database::delete_note,
            database::search_notes,
            database::migrate_from_json,
            database::parse_tags,
            database::sync_tags,
            database::get_all_tags,
            database::delete_tag,
            database::pin_tag,
            database::pin_note,
            database::get_notes_by_tags,
            database::count_notes_by_tags,
            database::count_notes,
            database::get_notes_heatmap,
            database::search_tags,
            database::create_todo,
            database::update_todo,
            database::delete_todo,
            database::get_todos_by_date,
            database::get_todos_by_month,
            database::get_reminders,
            database::count_todos,
            database::get_today_todos,
            file_reader::read_text_file,
            file_reader::read_file_text,
            file_reader::get_file_metadata,
            web_scraper::fetch_webpage_html,
            window_manager::get_os,
            api_sync::check_git_installed,
            api_sync::validate_sync_config,
            api_sync::sync_to_remote,
            api_sync::sync_from_remote,
            tray::show_main_window,
            tray::quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
