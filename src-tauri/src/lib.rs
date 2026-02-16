// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod ai_generator;
mod autostart;
mod database;
mod file_reader;
mod filesystem;
mod model_provider;
mod note;
mod protocol;
mod tag;
mod todo;
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
        .setup(|app| window_manager::setup_window_manager(app))
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
            model_provider::load_provider_models,
            ai_generator::generate_regex
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
