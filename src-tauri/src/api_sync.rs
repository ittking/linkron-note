use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 云同步配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiSyncConfig {
    pub repo_url: String, // 仓库URL (如: https://gitee.com/user/repo.git 或 user/repo)
    pub token: String,    // 访问 Token（用于 API 认证）
    #[serde(default = "default_branch")]
    pub branch: String, // 分支（默认 main）
}

fn default_branch() -> String {
    "main".to_string()
}

/// 同步结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub details: Option<SyncDetails>,
}

/// 同步详情
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncDetails {
    pub uploaded: usize,
    pub downloaded: usize,
    pub skipped: usize,
    pub total: usize,
    pub failed_files: Vec<String>,
}

/// Gitee API 文件信息响应
#[derive(Debug, Serialize, Deserialize)]
struct GiteeFileInfo {
    name: String,
    path: String,
    #[serde(rename = "type")]
    file_type: String,
    sha: String,
    size: i64,
    content: Option<String>,
    encoding: Option<String>,
}

/// 文件大小常量（Gitee API 限制）
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
fn parse_repo_url(repo_url: &str) -> Result<(String, String), String> {
    let url = repo_url.trim();

    // 如果是完整 URL
    if url.contains("://") {
        // 移除协议和可能的 .git 后缀
        let clean_url: String = url
            .replace("https://", "")
            .replace("http://", "")
            .trim_end_matches(".git")
            .to_string();

        // 提取 owner/repo
        let parts: Vec<&str> = clean_url.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2].to_string();
            let repo = parts[parts.len() - 1].to_string();
            return Ok((owner, repo));
        }
    } else if url.contains('/') {
        // 简短格式：owner/repo
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    Err("仓库地址格式无效，应为：用户名/仓库名 或完整的仓库URL".to_string())
}

/// 获取基础目录
fn get_base_directory(work_directory: &Option<String>) -> Result<PathBuf, String> {
    if let Some(work_dir) = work_directory {
        Ok(PathBuf::from(work_dir))
    } else {
        let mut path = dirs::data_local_dir().ok_or("Failed to get data directory")?;
        path.push("linkron");
        Ok(path)
    }
}

/// 构建API请求客户端
fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))
}

/// 验证同步配置
#[tauri::command]
pub async fn validate_sync_config(config: ApiSyncConfig) -> Result<SyncResult, String> {
    // 验证必填字段
    if config.repo_url.is_empty() {
        return Ok(SyncResult {
            success: false,
            message: "仓库地址不能为空".to_string(),
            details: None,
        });
    }
    if config.token.is_empty() {
        return Ok(SyncResult {
            success: false,
            message: "Token 不能为空".to_string(),
            details: None,
        });
    }

    // 解析仓库地址
    let (owner, repo) = parse_repo_url(&config.repo_url)?;

    let client = build_client()?;

    // 1. 先测试获取仓库信息
    let repo_url = format!("https://gitee.com/api/v5/repos/{}/{}", owner, repo);

    let repo_response = client
        .get(&repo_url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("User-Agent", "linkron")
        .send()
        .await;

    match repo_response {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "无法读取错误信息".to_string());

                if status.as_u16() == 401 {
                    return Ok(SyncResult {
                        success: false,
                        message: "Token 无效或权限不足".to_string(),
                        details: None,
                    });
                } else if status.as_u16() == 404 {
                    return Ok(SyncResult {
                        success: false,
                        message: "仓库不存在或无访问权限".to_string(),
                        details: None,
                    });
                } else {
                    return Ok(SyncResult {
                        success: false,
                        message: format!("连接失败: {} - {}", status, error_text),
                        details: None,
                    });
                }
            }
        }
        Err(e) => {
            return Ok(SyncResult {
                success: false,
                message: format!("网络请求失败: {}", e),
                details: None,
            });
        }
    }

    // 2. 验证分支是否存在
    let branches_url = format!("https://gitee.com/api/v5/repos/{}/{}/branches", owner, repo);

    let branches_response = client
        .get(&branches_url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("User-Agent", "linkron")
        .send()
        .await;

    let _branch_exists = match branches_response {
        Ok(resp) => {
            if resp.status().is_success() {
                // 解析分支列表，检查配置的分支是否存在
                if let Ok(branches_json) = resp.json::<serde_json::Value>().await {
                    if let Some(branches) = branches_json.as_array() {
                        // 检查分支是否存在
                        let found = branches.iter().any(|branch: &serde_json::Value| {
                            if let Some(branch_name) = branch.get("name") {
                                if let Some(name_str) = branch_name.as_str() {
                                    return name_str == config.branch;
                                }
                            }
                            false
                        });

                        if !found {
                            // 分支不存在，列出可用分支
                            let available_branches: Vec<String> = branches
                                .iter()
                                .filter_map(|branch: &serde_json::Value| {
                                    branch
                                        .get("name")
                                        .and_then(|n: &serde_json::Value| n.as_str())
                                        .map(|s| s.to_string())
                                })
                                .collect();

                            let branch_list = available_branches.join(", ");
                            return Ok(SyncResult {
                                success: false,
                                message: format!(
                                    "分支 '{}' 不存在。可用分支: {}",
                                    config.branch, branch_list
                                ),
                                details: None,
                            });
                        }
                    }
                    true
                } else {
                    false
                }
            } else {
                let status = resp.status();
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "无法读取错误信息".to_string());
                return Ok(SyncResult {
                    success: false,
                    message: format!("获取分支列表失败: {} - {}", status, error_text),
                    details: None,
                });
            }
        }
        Err(e) => {
            return Ok(SyncResult {
                success: false,
                message: format!("获取分支列表失败: {}", e),
                details: None,
            });
        }
    };

    // 3. 验证通过
    Ok(SyncResult {
        success: true,
        message: format!("连接成功，仓库访问正常，分支 '{}' 有效", config.branch),
        details: None,
    })
}

/// 读取本地文件内容并编码为base64
fn read_file_as_base64(path: &PathBuf) -> Result<String, String> {
    let content = fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let engine = base64::engine::general_purpose::STANDARD;
    Ok(engine.encode(content))
}

/// 创建或更新远程文件
async fn create_or_update_file(
    config: &ApiSyncConfig,
    path: &str,
    content: &str,
    is_binary: bool,
    sha: Option<String>,
) -> Result<(), String> {
    let (owner, repo) = parse_repo_url(&config.repo_url)?;
    let client = build_client()?;

    let url = format!(
        "https://gitee.com/api/v5/repos/{}/{}/contents/{}",
        owner, repo, url_encode_path(path)
    );

    let encoded_content = if is_binary {
        content.to_string()
    } else {
        // 文本内容也需要base64编码
        let engine = base64::engine::general_purpose::STANDARD;
        engine.encode(content)
    };

    // 检查内容大小（base64编码后）
    if encoded_content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大: {} 字节 (限制: {} 字节)",
            encoded_content.len(),
            MAX_FILE_SIZE
        ));
    }

    let mut request_body = json!({
        "content": encoded_content,
        "message": format!("Update {}", path),
        "branch": config.branch.clone(),
        "access_token": config.token.clone(),
    });

    // 新建用 POST，更新用 PUT（需提供 sha）
    let (method, action) = if let Some(s) = sha {
        request_body["sha"] = json!(s);
        ("PUT", "更新")
    } else {
        ("POST", "创建")
    };

    let response = client
        .request(
            reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
            &url,
        )
        .header("Authorization", format!("Bearer {}", config.token))
        .header("User-Agent", "linkron")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("{}文件失败: {}", action, e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误信息".to_string());
        return Err(format!("上传文件 {}: {} - {}", path, status, error_text));
    }

    Ok(())
}

/// 获取远程文件的 sha 值
async fn get_file_sha(config: &ApiSyncConfig, path: &str) -> Result<Option<String>, String> {
    let (owner, repo) = parse_repo_url(&config.repo_url)?;
    let client = build_client()?;

    let url = format!(
        "https://gitee.com/api/v5/repos/{}/{}/contents/{}?ref={}",
        owner, repo, url_encode_path(path), config.branch
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("User-Agent", "linkron")
        .send()
        .await
        .map_err(|e| format!("获取文件信息失败: {}", e))?;

    // 如果文件不存在，返回 None
    if response.status().as_u16() == 404 {
        return Ok(None);
    }

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误信息".to_string());
        return Err(format!(
            "获取文件信息 {}: {} - {}",
            path, status, error_text
        ));
    }

    let file_info: GiteeFileInfo = response
        .json()
        .await
        .map_err(|e| format!("解析文件信息失败: {}", e))?;

    Ok(Some(file_info.sha))
}

/// 下载远程文件到本地
async fn download_file(
    config: &ApiSyncConfig,
    path: &str,
    local_path: &PathBuf,
) -> Result<(), String> {
    let (owner, repo) = parse_repo_url(&config.repo_url)?;
    let client = build_client()?;

    let url = format!(
        "https://gitee.com/api/v5/repos/{}/{}/contents/{}?ref={}",
        owner, repo, url_encode_path(path), config.branch
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("User-Agent", "linkron")
        .send()
        .await
        .map_err(|e| format!("下载文件失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误信息".to_string());
        return Err(format!("下载文件 {}: {} - {}", path, status, error_text));
    }

    let file_info: GiteeFileInfo = response
        .json()
        .await
        .map_err(|e| format!("解析文件信息失败: {}", e))?;

    // 解码base64内容
    let engine = base64::engine::general_purpose::STANDARD;
    let content = engine
        .decode(file_info.content.ok_or("文件内容为空")?)
        .map_err(|e| format!("解码文件内容失败: {}", e))?;

    // 确保父目录存在
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 写入文件
    fs::write(local_path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}

/// 递归收集本地文件
fn collect_local_files(
    dir: &PathBuf,
    base_dir: &PathBuf,
) -> Result<Vec<(PathBuf, String)>, String> {
    let mut files = vec![];

    if !dir.exists() {
        return Ok(files);
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        // 跳过 .git 目录
        if path.file_name().map(|n| n == ".git").unwrap_or(false) {
            continue;
        }

        if path.is_dir() {
            // 递归处理子目录
            let sub_files = collect_local_files(&path, base_dir)?;
            files.extend(sub_files);
        } else {
            // 获取相对路径作为远程路径
            let relative_path = path
                .strip_prefix(base_dir)
                .map_err(|e| format!("获取相对路径失败: {}", e))?
                .to_str()
                .ok_or("路径包含无效字符")?
                .replace("\\", "/"); // Windows路径转Unix路径

            files.push((path, relative_path));
        }
    }

    Ok(files)
}

/// 收集远程文件（迭代方式）
async fn collect_remote_files_recursive(
    config: &ApiSyncConfig,
    path: &str,
) -> Result<Vec<String>, String> {
    let mut all_files = vec![];
    let mut dirs_to_process = vec![path.to_string()];

    while let Some(current_path) = dirs_to_process.pop() {
        let client = build_client()?;
        let (owner, repo) = parse_repo_url(&config.repo_url)?;

        let url = if current_path.is_empty() || current_path == "/" {
            format!(
                "https://gitee.com/api/v5/repos/{}/{}?ref={}",
                owner, repo, config.branch
            )
        } else {
            format!(
                "https://gitee.com/api/v5/repos/{}/{}/contents/{}?ref={}",
                owner, repo, url_encode_path(&current_path), config.branch
            )
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.token))
            .header("User-Agent", "linkron")
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            // 如果目录不存在，跳过
            if response.status().as_u16() == 404 {
                continue;
            }
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("获取远程文件列表失败: {} - {}", status, error_text));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        if let Some(items) = json.as_array() {
            for item in items {
                if let Some(file_type) = item.get("type").and_then(|t| t.as_str()) {
                    let file_path = item.get("path").and_then(|p| p.as_str()).unwrap_or("");

                    if file_type == "file" {
                        all_files.push(file_path.to_string());
                    } else if file_type == "dir" {
                        // 将子目录添加到待处理列表
                        dirs_to_process.push(file_path.to_string());
                    }
                }
            }
        }
    }

    Ok(all_files)
}

/// 同步清单文件（追踪已同步文件的 hash，实现增量同步）
const MANIFEST_FILE: &str = ".sync_manifest.json";

fn load_manifest(base_dir: &PathBuf) -> HashMap<String, String> {
    let path = base_dir.join(MANIFEST_FILE);
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_manifest(base_dir: &PathBuf, manifest: &HashMap<String, String>) {
    let path = base_dir.join(MANIFEST_FILE);
    if let Ok(json) = serde_json::to_string(manifest) {
        let _ = fs::write(path, json);
    }
}

fn compute_file_hash(local_path: &PathBuf) -> Result<String, String> {
    let content = fs::read(local_path).map_err(|e| format!("读取文件失败: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// 推送到云端
#[tauri::command]
pub async fn sync_to_remote(
    config: ApiSyncConfig,
    work_directory: Option<String>,
) -> Result<SyncResult, String> {
    let base_dir = get_base_directory(&work_directory)?;

    eprintln!("[DEBUG] 开始同步到云端");
    eprintln!("[DEBUG] 工作目录: {:?}", base_dir);

    // 确保目录存在
    if !base_dir.exists() {
        fs::create_dir_all(&base_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 收集本地文件
    let local_files = collect_local_files(&base_dir, &base_dir)?;
    eprintln!("[DEBUG] 本地文件数量: {}", local_files.len());

    // 加载同步清单（追踪文件 hash，实现增量同步）
    let mut manifest = load_manifest(&base_dir);

    let mut uploaded = 0;
    let mut skipped = 0;
    let mut unchanged = 0;
    let mut failed_files = vec![];

    // 上传每个文件
    for (local_path, remote_path) in local_files {
        eprintln!("[DEBUG] 处理文件: {}", remote_path);

        // 跳过清单文件本身
        if remote_path == MANIFEST_FILE {
            continue;
        }

        // 检查文件大小
        let file_size = fs::metadata(&local_path).map(|m| m.len()).unwrap_or(0);

        if file_size as usize > MAX_FILE_SIZE {
            eprintln!(
                "[DEBUG] 文件过大，跳过: {} ({} 字节)",
                remote_path, file_size
            );
            skipped += 1;
            failed_files.push(format!(
                "{} (文件过大: {} MB)",
                remote_path,
                file_size / 1024 / 1024
            ));
            continue;
        }

        // 计算本地文件 hash，与清单对比，未变更则跳过
        let local_hash = match compute_file_hash(&local_path) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[DEBUG] 计算hash失败: {} - {}", remote_path, e);
                failed_files.push(format!("{} (hash失败: {})", remote_path, e));
                continue;
            }
        };

        if let Some(cached_hash) = manifest.get(&remote_path) {
            if *cached_hash == local_hash {
                eprintln!("[DEBUG] 未变更，跳过: {}", remote_path);
                unchanged += 1;
                continue;
            }
        }

        // 获取远程文件的 sha（如果存在）
        let file_sha = get_file_sha(&config, &remote_path).await.unwrap_or(None);

        // 判断是否为二进制文件
        let is_binary = is_binary_file(&local_path);

        match read_file_as_base64(&local_path) {
            Ok(content) => {
                // 尝试创建或更新文件（提供 sha 用于更新）
                match create_or_update_file(&config, &remote_path, &content, is_binary, file_sha)
                    .await
                {
                    Ok(_) => {
                        uploaded += 1;
                        manifest.insert(remote_path.clone(), local_hash);
                        eprintln!("[DEBUG] 上传成功: {}", remote_path);
                    }
                    Err(e) => {
                        eprintln!("[DEBUG] 上传失败: {} - {}", remote_path, e);
                        failed_files.push(format!("{} ({})", remote_path, e));
                    }
                }
            }
            Err(e) => {
                eprintln!("[DEBUG] 读取文件失败: {} - {}", remote_path, e);
                failed_files.push(format!("{} (读取失败: {})", remote_path, e));
            }
        }
    }

    // 保存同步清单
    save_manifest(&base_dir, &manifest);

    eprintln!(
        "[DEBUG] 上传完成: 成功 {} 个，未变更 {} 个，跳过 {} 个",
        uploaded, unchanged, skipped
    );

    // 构建消息
    let total_skipped = unchanged + skipped;
    let message = if failed_files.is_empty() {
        format!(
            "同步成功，已上传 {} 个文件{}",
            uploaded,
            if total_skipped > 0 { format!("，已是最新 {} 个", total_skipped) } else { String::new() }
        )
    } else {
        format!(
            "同步完成: 成功 {} 个，未变更 {} 个，跳过 {} 个，失败 {} 个",
            uploaded, unchanged, skipped, failed_files.len()
        )
    };

    Ok(SyncResult {
        success: failed_files.is_empty(),
        message,
        details: Some(SyncDetails {
            uploaded,
            downloaded: 0,
            skipped,
            total: uploaded + skipped + failed_files.len(),
            failed_files: failed_files.clone(),
        }),
    })
}

/// 从云端拉取
#[tauri::command]
pub async fn sync_from_remote(
    config: ApiSyncConfig,
    work_directory: Option<String>,
) -> Result<SyncResult, String> {
    let base_dir = get_base_directory(&work_directory)?;

    eprintln!("[DEBUG] 开始从云端拉取");
    eprintln!("[DEBUG] 工作目录: {:?}", base_dir);

    // 确保目录存在
    if !base_dir.exists() {
        fs::create_dir_all(&base_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 获取远程文件列表
    let remote_files = collect_remote_files_recursive(&config, "").await?;
    eprintln!("[DEBUG] 远程文件数量: {}", remote_files.len());

    let mut downloaded = 0;

    // 下载每个文件
    for remote_path in remote_files {
        eprintln!("[DEBUG] 下载文件: {}", remote_path);

        let local_path = base_dir.join(remote_path.replace("/", std::path::MAIN_SEPARATOR_STR));

        match download_file(&config, &remote_path, &local_path).await {
            Ok(_) => {
                downloaded += 1;
                eprintln!("[DEBUG] 下载成功: {}", remote_path);
            }
            Err(e) => {
                eprintln!("[DEBUG] 下载失败: {} - {}", remote_path, e);
                // 继续下载其他文件
            }
        }
    }

    eprintln!("[DEBUG] 下载完成: {} 个文件", downloaded);

    Ok(SyncResult {
        success: true,
        message: format!("同步成功，已下载 {} 个文件", downloaded),
        details: Some(SyncDetails {
            uploaded: 0,
            downloaded,
            skipped: 0,
            total: downloaded,
            failed_files: vec![],
        }),
    })
}

/// URL 编码文件路径（用于 Gitee API）
fn url_encode_path(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            let mut encoded = String::new();
            for byte in seg.bytes() {
                match byte {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        encoded.push(byte as char);
                    }
                    _ => {
                        encoded.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
            encoded
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// 判断是否为二进制文件
fn is_binary_file(path: &PathBuf) -> bool {
    // 通过扩展名判断
    if let Some(ext) = path.extension() {
        let ext = ext.to_str().unwrap_or("").to_lowercase();
        matches!(
            ext.as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "bmp"
                | "ico"
                | "webp"
                | "pdf"
                | "zip"
                | "rar"
                | "7z"
                | "tar"
                | "gz"
                | "exe"
                | "dll"
                | "so"
                | "dylib"
        )
    } else {
        false
    }
}

/// 检查 Git 是否安装（保留兼容性，但不再使用）
#[tauri::command]
pub fn check_git_installed() -> Result<bool, String> {
    Ok(true) // API方式不需要Git
}
