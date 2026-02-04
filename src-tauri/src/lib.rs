// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod autostart;
mod filesystem;
mod terminal;
mod database;
mod protocol;
mod file_reader;
mod web_scraper;

#[cfg(any(windows, target_os = "macos"))]
mod window_manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_os() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("iterm", protocol::iterm_protocol_handler)
        .setup(|app| window_manager::setup_window_manager(app))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_os,
            autostart::set_autostart,
            autostart::is_autostart_enabled,
            filesystem::check_directory_exists,
            filesystem::create_directory,
            filesystem::save_image,
            filesystem::save_file,
            filesystem::get_local_path_from_protocol,
            filesystem::delete_resource_by_url,
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
            database::get_notes_by_tags,
            database::search_tags,
            database::create_or_get_tag,
            database::rename_tag,
            file_reader::read_text_file,
            file_reader::read_file_text,
            file_reader::get_file_metadata,
            web_scraper::fetch_webpage_html
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
