use std::path::{Path, PathBuf};
use tauri::http::Request;
use tauri::http::Response;
use tauri::{AppHandle, Manager, UriSchemeContext};

/// 从配置文件读取工作目录
pub fn read_work_directory<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> Option<String> {
    let store_path = app_handle.path().app_data_dir().ok()?.join("settings.json");
    let content = std::fs::read_to_string(&store_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let work_dir = json.get("workDirectory")?.as_str()?;

    if work_dir.is_empty() {
        None
    } else {
        Some(work_dir.to_string())
    }
}

/// 获取基础目录
fn get_base_directory<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> PathBuf {
    read_work_directory(app_handle)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_local_dir()
                .map(|mut p| {
                    p.push("linkron");
                    p
                })
                .unwrap_or_else(|| PathBuf::from("."))
        })
}

/// 验证并规范化资源路径，防止路径遍历攻击
fn validate_and_normalize_path(base_dir: &Path, resource_path: &str) -> Option<PathBuf> {
    // 空路径检查
    if resource_path.is_empty() {
        return None;
    }

    // 检查路径遍历攻击
    if resource_path.contains("..") || resource_path.contains('\\') {
        return None;
    }

    // 检查是否包含非法路径组件
    for component in resource_path.split('/') {
        if component.is_empty() || component == "." || component.contains("..") {
            continue;
        }
        // 放宽限制：只要不包含明显的非法字符即可
        // 允许字母、数字、横线、下划线、点、空格和中文字符
        for c in component.chars() {
            if !c.is_alphanumeric() && c != '-' && c != '_' && c != '.' && c != ' ' {
                // 允许 Unicode 字符（包括中文）
                if !('\u{4e00}'..='\u{9fff}').contains(&c) {
                    return None;
                }
            }
        }
    }

    let full_path = base_dir.join(resource_path);

    // 先检查文件是否存在，避免 canonicalize() 对不存在文件返回 None
    if !full_path.exists() {
        return None;
    }

    // 规范化路径并确保它在基础目录内
    let normalized = full_path.canonicalize().ok()?;
    let base_normalized = base_dir.canonicalize().ok()?;

    // 确保解析后的路径以基础目录开头
    if !normalized.starts_with(&base_normalized) {
        return None;
    }

    Some(normalized)
}

/// 构建错误响应
fn build_error_response(status: u16, body: &'static str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(body.as_bytes().to_vec())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(b"Internal server error".to_vec())
                .unwrap()
        })
}

/// linkron:// 自定义协议处理器
/// URL 格式: linkron://localhost/resources/images/xxx.png
pub fn iterm_protocol_handler<R: tauri::Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let app = ctx.app_handle();
    let uri = request.uri();
    let uri_str = uri.to_string();

    // 从完整 URI 中提取资源路径
    // 可能的格式:
    // 1. linkron://localhost/resources/images/xxx.png
    // 2. linkron://resources/images/xxx.png
    // 3. http://linkron.localhost/resources/images/xxx.png (Tauri 2.x 格式)
    // 4. /resources/images/xxx.png (直接路径)
    let resource_path = if uri_str.contains("/resources/") {
        let parts: Vec<&str> = uri_str.split("/resources/").collect();
        if parts.len() >= 2 {
            // 移除可能的查询参数和协议前缀
            let path = parts[1].split('?').next().unwrap_or(parts[1]);
            path
        } else {
            return build_error_response(400, "Invalid request path");
        }
    } else if uri_str.starts_with("resources/") {
        // 直接以 resources/ 开头的情况
        let path = uri_str.strip_prefix("resources/").unwrap_or(&uri_str);
        path
    } else {
        return build_error_response(400, "Invalid request: must contain /resources/");
    };

    let base_dir = get_base_directory(app);
    let resources_dir = base_dir.join("resources");

    // 验证并规范化路径
    let file_path = match validate_and_normalize_path(&resources_dir, resource_path) {
        Some(p) => p,
        None => {
            return build_error_response(400, "Invalid request path");
        }
    };

    match std::fs::read(&file_path) {
        Ok(content) => {
            let mime_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            Response::builder()
                .header("Content-Type", mime_type)
                .header("Access-Control-Allow-Origin", "*")
                .header("Cache-Control", "public, max-age=31536000")
                .status(200)
                .body(content)
                .unwrap_or_else(|_| build_error_response(500, "Failed to build response"))
        }
        Err(_) => build_error_response(404, "File not found"),
    }
}
