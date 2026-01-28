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

/// 保存图片文件
/// 返回相对路径
#[tauri::command]
pub async fn save_image(file_data: Vec<u8>, file_name: String, work_directory: Option<String>) -> Result<String, String> {
    use std::path::PathBuf;

    // 确定图片保存目录
    let resources_dir = if let Some(work_dir) = work_directory {
        // 使用工作空间目录
        let mut path = PathBuf::from(&work_dir);
        path.push("resources");
        path
    } else {
        // 使用应用数据目录
        let mut path = dirs::data_local_dir().ok_or("Failed to get data directory")?;
        path.push("iterm");
        path.push("resources");
        path
    };

    // 创建 resources 目录
    std::fs::create_dir_all(&resources_dir)
        .map_err(|e| format!("Failed to create resources directory: {}", e))?;

    // 从原始文件名获取扩展名
    let file_path_buf = PathBuf::from(&file_name);
    let ext = file_path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    // 使用时间戳 + 随机数生成唯一文件名，避免编码问题
    let timestamp = chrono::Utc::now().timestamp_millis();
    let random: u32 = rand::random();
    let unique_file_name = format!("{}-{}.{}", timestamp, random, ext);

    // 构建完整文件路径
    let file_path = resources_dir.join(&unique_file_name);

    // 写入文件
    std::fs::write(&file_path, &file_data)
        .map_err(|e| format!("Failed to write image file: {}", e))?;

    // 返回相对路径
    Ok(format!("resources/{}", unique_file_name))
}

/// 获取图片的完整路径
#[tauri::command]
pub async fn get_image_path(relative_path: String, work_directory: Option<String>) -> Result<String, String> {
    use std::path::PathBuf;
    
    // 确定基础目录
    let base_dir = if let Some(work_dir) = work_directory {
        PathBuf::from(&work_dir)
    } else {
        let mut path = dirs::data_local_dir().ok_or("Failed to get data directory")?;
        path.push("iterm");
        path
    };
    
    // 构建完整路径
    let full_path = base_dir.join(&relative_path);
    
    // 检查文件是否存在
    if !full_path.exists() {
        return Err(format!("Image file not found: {}", relative_path));
    }
    
    // 返回完整路径
    full_path.to_str()
        .map(|s| s.to_string())
        .ok_or("Failed to convert path to string".to_string())
}

/// 获取资源的 iterm:// 协议 URL
/// 在 Windows 上使用 http://iterm.localhost/ 格式
/// 在其他平台使用 iterm:// 格式
#[tauri::command]
pub fn get_resource_url(relative_path: String) -> String {
    // 规范化路径：移除开头的斜杠，将反斜杠替换为正斜杠
    let normalized = relative_path
        .trim_start_matches('/')
        .replace('\\', "/");

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 http://iterm.localhost/ 格式
        format!("http://iterm.localhost/{}", normalized)
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 其他平台: 使用 iterm:// 格式
        format!("iterm://{}", normalized)
    }
}

/// 保存文件（通用接口，支持图片和附件）
/// 返回相对路径
#[tauri::command]
pub async fn save_file(
    file_data: Vec<u8>,
    file_name: String,
    file_type: String,
    work_directory: Option<String>
) -> Result<String, String> {
    use std::path::PathBuf;

    // 确定基础目录
    let base_dir = if let Some(work_dir) = work_directory {
        PathBuf::from(&work_dir)
    } else {
        let mut path = dirs::data_local_dir().ok_or("Failed to get data directory")?;
        path.push("iterm");
        path
    };

    // 根据文件类型确定子目录
    let resources_dir = base_dir.join("resources");
    let target_dir = if file_type == "image" {
        resources_dir.join("images")
    } else {
        resources_dir.join("files")
    };

    // 创建目标目录
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // 从原始文件名获取扩展名
    let file_path_buf = PathBuf::from(&file_name);
    let ext = file_path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    // 使用时间戳 + 随机数生成唯一文件名，避免编码问题
    let timestamp = chrono::Utc::now().timestamp_millis();
    let random: u32 = rand::random();
    let unique_file_name = format!("{}-{}.{}", timestamp, random, ext);

    // 构建完整文件路径
    let file_path = target_dir.join(&unique_file_name);

    // 写入文件
    std::fs::write(&file_path, &file_data)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // 返回相对路径
    let relative_path = if file_type == "image" {
        format!("resources/images/{}", unique_file_name)
    } else {
        format!("resources/files/{}", unique_file_name)
    };

    Ok(relative_path)
}

/// 将 iterm:// 协议 URL 转换为本地文件路径
/// 
/// 参数:
/// - protocol_url: iterm:// 协议 URL (如: http://iterm.localhost/resources/files/1234567890-1234.bin)
/// - work_directory: 工作目录（可选）
/// 
/// 返回:
/// - Ok(String): 本地文件路径
/// - Err(String): 错误信息
#[tauri::command]
pub fn get_local_path_from_protocol(protocol_url: String, work_directory: Option<String>) -> Result<String, String> {
    use std::path::PathBuf;
    
    // 解析协议 URL
    let path = if protocol_url.starts_with("http://iterm.localhost/") {
        // Windows 格式: http://iterm.localhost/resources/files/1234567890-1234.bin
        protocol_url.replace("http://iterm.localhost/", "")
    } else if protocol_url.starts_with("iterm://") {
        // 其他平台格式: iterm://resources/files/1234567890-1234.bin
        protocol_url.replace("iterm://", "")
    } else {
        return Err(format!("无效的协议 URL: {}", protocol_url));
    };
    
    // 确定基础目录
    let base_dir = if let Some(work_dir) = work_directory {
        PathBuf::from(&work_dir)
    } else {
        let mut path = dirs::data_local_dir().ok_or("Failed to get data directory")?;
        path.push("iterm");
        path
    };
    
    // 构建完整路径
    let full_path = base_dir.join(&path);
    
    // 检查文件是否存在
    if !full_path.exists() {
        return Err(format!("文件不存在: {}", full_path.display()));
    }
    
    // 返回本地文件路径
    full_path.to_str()
        .map(|s| s.to_string())
        .ok_or("Failed to convert path to string".to_string())
}