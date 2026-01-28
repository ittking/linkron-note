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

    // 返回完整 URL
    Ok(format!("http://iterm.localhost/resources/images/{}", unique_file_name))
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

pub fn get_resource_url(relative_path: String) -> String {
    // 规范化路径：移除开头的斜杠，将反斜杠替换为正斜杠
    let normalized = relative_path
        .trim_start_matches('/')
        .replace('\\', "/");

    // 统一使用 http://iterm.localhost/ 格式
    format!("http://iterm.localhost/resources/{}", normalized)
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

    // 返回完整 URL
    let resource_path = if file_type == "image" {
        format!("images/{}", unique_file_name)
    } else {
        format!("files/{}", unique_file_name)
    };

    // 生成完整 URL
    let full_url = format!("http://iterm.localhost/resources/{}", resource_path);
    Ok(full_url)
}

/// 删除资源文件（公共方法）
/// 
/// 参数:
/// - url: 资源 URL (如: http://local.iterm/resources/images/1234567890-1234.png)
/// - work_directory: 工作目录（可选）
/// 
/// 返回:
/// - Ok(()): 删除成功
/// - Err(String): 错误信息
#[tauri::command]
pub fn delete_resource_by_url(url: String, work_directory: Option<String>) -> Result<(), String> {
    use std::path::PathBuf;
    
    // 检查是否是本地资源 URL
    if !url.starts_with("http://iterm.localhost/resources/") {
        return Ok(()); // 外部 URL，跳过删除
    }
    
    // 提取资源路径：移除 http://iterm.localhost/resources/ 前缀
    let resource_path = url.trim_start_matches("http://iterm.localhost/resources/");
    
    // 确定基础目录
    let base_dir = if let Some(work_dir) = work_directory {
        PathBuf::from(&work_dir)
    } else {
        let mut path = dirs::data_local_dir().ok_or("Failed to get data directory")?;
        path.push("iterm");
        path
    };
    
    // 构建完整路径
    let full_path = base_dir.join(resource_path);
    
    // 检查文件是否存在
    if !full_path.exists() {
        return Ok(()); // 文件不存在，视为删除成功
    }
    
    // 删除文件
    std::fs::remove_file(&full_path)
        .map_err(|e| format!("Failed to delete file {}: {}", full_path.display(), e))?;
    
    Ok(())
}