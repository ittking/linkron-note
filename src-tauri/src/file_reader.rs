use std::fs;
use std::path::Path;

/// 读取文本文件（txt、md）
#[tauri::command]
pub fn read_text_file(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))
}

/// 读取文件并自动识别类型
#[tauri::command]
pub fn read_file_text(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "txt" | "md" | "markdown" => read_text_file(file_path),
        _ => Err(format!("不支持的文件类型: .{}", extension)),
    }
}

/// 获取文件元数据
#[derive(serde::Serialize)]
pub struct FileMetadata {
    pub name: String,
    pub size: u64,
    pub extension: String,
    pub mime_type: Option<String>,
}

#[tauri::command]
pub fn get_file_metadata(file_path: String) -> Result<FileMetadata, String> {
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    let metadata = fs::metadata(&path).map_err(|e| format!("获取文件元数据失败: {}", e))?;

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();

    let mime_type = mime_guess::from_path(&path).first().map(|m| m.to_string());

    Ok(FileMetadata {
        name,
        size: metadata.len(),
        extension,
        mime_type,
    })
}
