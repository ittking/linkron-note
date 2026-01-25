#[tauri::command]
#[allow(dead_code)]
pub async fn check_directory_exists(path: String) -> Result<bool, String> {
    std::fs::metadata(&path)
        .map(|m| m.is_dir())
        .map_err(|e| format!("Failed to check directory: {}", e))
}

#[tauri::command]
#[allow(dead_code)]
pub async fn create_directory(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create directory: {}", e))
}