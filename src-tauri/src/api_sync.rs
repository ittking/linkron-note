use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 云同步配置（简化版）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiSyncConfig {
    pub repo_url: String,  // 仓库URL (如: https://github.com/user/repo 或 user/repo)
    pub token: String,     // 访问 Token
    pub branch: String,    // 分支（默认 main/master）
}

/// 解析后的仓库信息
#[derive(Debug, Clone)]
struct ParsedRepoInfo {
    pub platform: String,  // "github" 或 "gitee"
    pub username: String,  // 用户名
    pub repo_name: String, // 仓库名
}

/// 解析仓库URL
fn parse_repo_url(repo_url: &str) -> Result<ParsedRepoInfo, String> {
    let url = repo_url.trim();

    // 支持的格式：
    // 1. https://github.com/user/repo
    // 2. https://gitee.com/user/repo
    // 3. https://github.com/user/repo.git
    // 4. https://gitee.com/user/repo.git
    // 5. http://github.com/user/repo
    // 6. http://gitee.com/user/repo
    // 7. github.com/user/repo
    // 8. gitee.com/user/repo
    // 9. user/repo (默认为Gitee)

    // 如果是 user/repo 格式，默认为Gitee
    if !url.contains('/') || url.starts_with('/') {
        return Err("仓库地址格式无效，应为：用户名/仓库名 或完整的仓库URL".to_string());
    }

    // 尝试解析完整URL
    if url.contains("://") {
        // 完整URL格式，去除 .git 后缀
        let clean_url = if url.ends_with(".git") {
            &url[..url.len() - 4]
        } else {
            url
        };

        let url_lower = clean_url.to_lowercase();

        if url_lower.contains("github.com") {
            let re = Regex::new(r"github\.com/([^/]+)/([^/?]+)").unwrap();
            if let Some(caps) = re.captures(clean_url) {
                return Ok(ParsedRepoInfo {
                    platform: "github".to_string(),
                    username: caps.get(1).unwrap().as_str().to_string(),
                    repo_name: caps.get(2).unwrap().as_str().to_string(),
                });
            }
        } else if url_lower.contains("gitee.com") {
            let re = Regex::new(r"gitee\.com/([^/]+)/([^/?]+)").unwrap();
            if let Some(caps) = re.captures(clean_url) {
                return Ok(ParsedRepoInfo {
                    platform: "gitee".to_string(),
                    username: caps.get(1).unwrap().as_str().to_string(),
                    repo_name: caps.get(2).unwrap().as_str().to_string(),
                });
            }
        }

        return Err("无法解析仓库URL，请检查格式".to_string());
    }

    // 简短格式 user/repo，去除 .git 后缀
    let clean_url = if url.ends_with(".git") {
        &url[..url.len() - 4]
    } else {
        url
    };

    let parts: Vec<&str> = clean_url.split('/').collect();
    if parts.len() == 2 {
        return Ok(ParsedRepoInfo {
            platform: "gitee".to_string(), // 默认为Gitee
            username: parts[0].to_string(),
            repo_name: parts[1].to_string(),
        });
    }

    Err("仓库地址格式无效".to_string())
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
}

/// GitHub/Gitee API 文件信息响应
#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    name: String,
    path: String,
    sha: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    r#type: Option<String>,
}

/// GitHub/Gitee API 文件上传请求
#[derive(Debug, Serialize)]
struct UploadRequest {
    message: String,
    content: String,
    branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
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

    // 解析仓库URL
    let repo_info = parse_repo_url(&config.repo_url)?;

    // 构建验证请求
    let (base_url, accept_header) = if repo_info.platform == "github" {
        ("https://api.github.com", "application/vnd.github.v3+json")
    } else {
        ("https://gitee.com/api/v5", "application/json")
    };

    let url = format!(
        "{}/repos/{}/{}",
        base_url, repo_info.username, repo_info.repo_name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("token {}", config.token))
        .header("Accept", accept_header)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(SyncResult {
                    success: true,
                    message: "连接成功，仓库访问正常".to_string(),
                    details: None,
                })
            } else if resp.status() == 401 {
                Ok(SyncResult {
                    success: false,
                    message: "Token 无效或权限不足".to_string(),
                    details: None,
                })
            } else if resp.status() == 404 {
                Ok(SyncResult {
                    success: false,
                    message: "仓库不存在或无访问权限".to_string(),
                    details: None,
                })
            } else {
                let status = resp.status();
                Ok(SyncResult {
                    success: false,
                    message: format!("验证失败: HTTP {}", status),
                    details: None,
                })
            }
        }
        Err(e) => Ok(SyncResult {
            success: false,
            message: format!("网络请求失败: {}", e),
            details: None,
        }),
    }
}

/// 获取已保存的同步配置
#[tauri::command]
pub async fn get_sync_config(
    work_directory: Option<String>,
) -> Result<Option<ApiSyncConfig>, String> {
    let base_dir = get_base_directory(&work_directory)?;
    let config_path = base_dir.join("sync_config.json");

    if !config_path.exists() {
        return Ok(None);
    }

    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: ApiSyncConfig =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    Ok(Some(config))
}

/// 保存同步配置
#[tauri::command]
pub async fn save_sync_config(
    config: ApiSyncConfig,
    work_directory: Option<String>,
) -> Result<(), String> {
    let base_dir = get_base_directory(&work_directory)?;
    let config_path = base_dir.join("sync_config.json");

    let content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&config_path, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

/// 推送到云端
#[tauri::command]
pub async fn sync_to_remote(
    config: ApiSyncConfig,
    work_directory: Option<String>,
) -> Result<SyncResult, String> {
    let base_dir = get_base_directory(&work_directory)?;
    let client = reqwest::Client::new();

    // 解析仓库URL
    let repo_info = parse_repo_url(&config.repo_url)?;
    let (base_url, accept_header) = get_api_config(&repo_info.platform);

    // 统计信息
    let mut uploaded = 0usize;
    let mut skipped = 0usize;
    let mut total = 0usize;
    let mut errors = Vec::new();

    // 1. 同步 notes.db
    total += 1;
    let db_path = base_dir.join("notes.db");
    if db_path.exists() {
        match upload_file(
            &client,
            &repo_info,
            &config.token,
            &config.branch,
            &base_url,
            &accept_header,
            &db_path,
            "notes.db",
        )
        .await
        {
            Ok(true) => uploaded += 1,
            Ok(false) => skipped += 1,
            Err(e) => errors.push(format!("notes.db: {}", e)),
        }
    } else {
        errors.push("notes.db 文件不存在".to_string());
    }

    // 2. 同步 resources 文件夹
    let resources_path = base_dir.join("resources");
    if resources_path.exists() {
        let files = collect_files(&resources_path)?;
        total += files.len();

        for (local_path, remote_path) in files {
            match upload_file(
                &client,
                &repo_info,
                &config.token,
                &config.branch,
                &base_url,
                &accept_header,
                &local_path,
                &remote_path,
            )
            .await
            {
                Ok(true) => uploaded += 1,
                Ok(false) => skipped += 1,
                Err(e) => errors.push(format!("{}: {}", remote_path, e)),
            }
        }
    }

    // 3. 保存同步时间
    save_sync_time(&base_dir)?;

    // 构建结果
    let success = errors.is_empty();
    let message = if success {
        format!(
            "同步成功: 上传 {} 个，跳过 {} 个，共 {} 个文件",
            uploaded, skipped, total
        )
    } else {
        format!("同步完成但有错误: {}", errors.join("; "))
    };

    Ok(SyncResult {
        success,
        message,
        details: Some(SyncDetails {
            uploaded,
            downloaded: 0,
            skipped,
            total,
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
    let client = reqwest::Client::new();

    // 解析仓库URL
    let repo_info = parse_repo_url(&config.repo_url)?;
    let (base_url, accept_header) = get_api_config(&repo_info.platform);

    // 统计信息
    let mut downloaded = 0usize;
    let mut skipped = 0usize;
    let mut total = 1usize; // notes.db
    let mut errors = Vec::new();

    // 1. 下载 notes.db
    match download_file(
        &client,
        &repo_info,
        &config.token,
        &base_url,
        &accept_header,
        "notes.db",
        &base_dir,
    )
    .await
    {
        Ok(true) => downloaded += 1,
        Ok(false) => skipped += 1,
        Err(e) => errors.push(format!("notes.db: {}", e)),
    }

    // 2. 下载 resources 文件夹
    let resources_path = base_dir.join("resources");
    fs::create_dir_all(&resources_path).map_err(|e| format!("创建 resources 目录失败: {}", e))?;

    match download_directory(
        &client,
        &repo_info,
        &config.token,
        &base_url,
        &accept_header,
        "resources".to_string(),
        &resources_path,
        &mut total,
        &mut downloaded,
        &mut skipped,
        &mut errors,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => errors.push(format!("resources: {}", e)),
    }

    // 3. 保存同步时间
    save_sync_time(&base_dir)?;

    // 构建结果
    let success = errors.is_empty();
    let message = if success {
        format!(
            "同步成功: 下载 {} 个，跳过 {} 个，共 {} 个文件",
            downloaded, skipped, total
        )
    } else {
        format!("同步完成但有错误: {}", errors.join("; "))
    };

    Ok(SyncResult {
        success,
        message,
        details: Some(SyncDetails {
            uploaded: 0,
            downloaded,
            skipped,
            total,
        }),
    })
}

/// 获取 API 配置
fn get_api_config(platform: &str) -> (&'static str, &'static str) {
    if platform == "github" {
        ("https://api.github.com", "application/vnd.github.v3+json")
    } else {
        ("https://gitee.com/api/v5", "application/json")
    }
}

/// 上传单个文件（强制覆盖模式）
async fn upload_file(
    client: &reqwest::Client,
    repo_info: &ParsedRepoInfo,
    token: &str,
    branch: &str,
    base_url: &str,
    accept_header: &str,
    local_path: &Path,
    remote_path: &str,
) -> Result<bool, String> {
    // 读取文件内容
    let content = fs::read(local_path).map_err(|e| format!("读取文件失败: {}", e))?;

    // Base64 编码
    use base64::{engine::general_purpose::STANDARD, Engine};
    let encoded = STANDARD.encode(&content);

    // 检查文件大小
    let file_size = content.len();
    let max_size = if repo_info.platform == "github" {
        100 * 1024 * 1024 // 100MB
    } else {
        50 * 1024 * 1024 // 50MB
    };

    if file_size > max_size {
        return Err(format!(
            "文件过大: {} 字节 (最大 {} 字节)",
            file_size, max_size
        ));
    }

    // 构建上传请求 URL（Gitee 需要使用 access_token 参数）
    let url = if repo_info.platform == "gitee" {
        format!(
            "{}/repos/{}/{}/contents/{}?access_token={}",
            base_url, repo_info.username, repo_info.repo_name, remote_path, token
        )
    } else {
        format!(
            "{}/repos/{}/{}/contents/{}",
            base_url, repo_info.username, repo_info.repo_name, remote_path
        )
    };

    // 获取远程文件信息
    let remote_info = get_remote_file_info(client, repo_info, token, branch, base_url, accept_header, remote_path).await;

    match remote_info {
        Ok(Some(info)) => {
            // 文件存在，需要带 SHA 更新
            let upload_req = UploadRequest {
                message: format!("Update {}", remote_path),
                content: encoded,
                branch: branch.to_string(),
                sha: Some(info.sha),
            };

            let response = if repo_info.platform == "gitee" {
                client
                    .put(&url)
                    .header("Accept", accept_header)
                    .json(&upload_req)
            } else {
                client
                    .put(&url)
                    .header("Authorization", format!("token {}", token))
                    .header("Accept", accept_header)
                    .json(&upload_req)
            }
            .send()
            .await
            .map_err(|e| format!("上传请求失败: {}", e))?;

            if response.status().is_success() {
                return Ok(true);
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!("更新文件失败: HTTP {} - {}", status, error_text));
            }
        },
        Ok(None) => {
            // 文件不存在（404），创建新文件
            let upload_req = UploadRequest {
                message: format!("Add {}", remote_path),
                content: encoded,
                branch: branch.to_string(),
                sha: None, // 新文件不需要 SHA
            };

            let response = if repo_info.platform == "gitee" {
                client
                    .put(&url)
                    .header("Accept", accept_header)
                    .json(&upload_req)
            } else {
                client
                    .put(&url)
                    .header("Authorization", format!("token {}", token))
                    .header("Accept", accept_header)
                    .json(&upload_req)
            }
            .send()
            .await
            .map_err(|e| format!("上传请求失败: {}", e))?;

            if response.status().is_success() {
                return Ok(true);
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                // 如果提示需要 SHA，说明仓库配置有问题（如分支不存在）
                if error_text.contains("sha is missing") || error_text.contains("sha is empty") {
                    return Err(format!("仓库配置错误：仓库或分支不存在。请先在 Gitee 上创建仓库并初始化（添加 README.md 或任何文件），或手动创建 '{}' 分支。", branch));
                }

                return Err(format!("创建文件失败: HTTP {} - {}", status, error_text));
            }
        },
        Err(e) => {
            // 获取文件信息失败
            return Err(format!("无法获取文件信息: {}", e));
        }
    }
}

/// 下载单个文件
async fn download_file(
    client: &reqwest::Client,
    repo_info: &ParsedRepoInfo,
    token: &str,
    base_url: &str,
    accept_header: &str,
    remote_path: &str,
    local_dir: &Path,
) -> Result<bool, String> {
    let url = format!(
        "{}/repos/{}/{}/contents/{}",
        base_url, repo_info.username, repo_info.repo_name, remote_path
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("token {}", token))
        .header("Accept", accept_header)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !response.status().is_success() {
        if response.status() == 404 {
            return Ok(false); // 文件不存在，跳过
        }
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let file_info: FileInfo = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 解码 Base64 内容
    use base64::{engine::general_purpose::STANDARD, Engine};
    let remote_content = STANDARD
        .decode(&file_info.content.ok_or("文件内容为空")?)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    // 检查是否需要下载
    let local_path = local_dir.join(&file_info.name);
    if local_path.exists() {
        let local_content =
            fs::read(&local_path).map_err(|e| format!("读取本地文件失败: {}", e))?;
        let local_sha = calculate_file_sha(&local_content)?;
        let remote_sha = calculate_file_sha(&remote_content)?;
        if local_sha == remote_sha {
            return Ok(false); // 内容相同，跳过
        }
    }

    // 写入文件
    fs::write(&local_path, remote_content).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(true)
}

/// 下载目录（递归）
fn download_directory<'a>(
    client: &'a reqwest::Client,
    repo_info: &'a ParsedRepoInfo,
    token: &'a str,
    base_url: &'a str,
    accept_header: &'a str,
    remote_path: String,
    local_dir: &'a Path,
    total: &'a mut usize,
    downloaded: &'a mut usize,
    skipped: &'a mut usize,
    errors: &'a mut Vec<String>,
) -> impl std::future::Future<Output = Result<(), String>> + 'a {
    Box::pin(async move {
        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            base_url, repo_info.username, repo_info.repo_name, remote_path
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("token {}", token))
            .header("Accept", accept_header)
            .send()
            .await
            .map_err(|e| format!("列出目录失败: {}", e))?;

        if !response.status().is_success() {
            if response.status() == 404 {
                return Ok(()); // 目录不存在，跳过
            }
            return Err(format!("列出目录失败: HTTP {}", response.status()));
        }

        let items: Vec<FileInfo> = response
            .json()
            .await
            .map_err(|e| format!("解析目录列表失败: {}", e))?;

        for item in items {
            *total += 1;

            if item.r#type.as_deref() == Some("dir") {
                // 子目录，递归处理
                let sub_dir = local_dir.join(&item.name);
                fs::create_dir_all(&sub_dir).map_err(|e| format!("创建目录失败: {}", e))?;

                let sub_remote_path = format!("{}/{}", remote_path, item.name);
                download_directory(
                    client,
                    repo_info,
                    token,
                    base_url,
                    accept_header,
                    sub_remote_path,
                    &sub_dir,
                    total,
                    downloaded,
                    skipped,
                    errors,
                )
                .await?;
            } else {
                // 文件，下载
                let item_remote_path = format!("{}/{}", remote_path, item.name);
                match download_file(
                    client,
                    repo_info,
                    token,
                    base_url,
                    accept_header,
                    &item_remote_path,
                    local_dir,
                )
                .await
                {
                    Ok(true) => *downloaded += 1,
                    Ok(false) => *skipped += 1,
                    Err(e) => errors.push(format!("{}: {}", item_remote_path, e)),
                }
            }
        }

        Ok(())
    })
}

/// 获取远程文件信息
async fn get_remote_file_info(
    client: &reqwest::Client,
    repo_info: &ParsedRepoInfo,
    token: &str,
    branch: &str,
    base_url: &str,
    accept_header: &str,
    remote_path: &str,
) -> Result<Option<FileInfo>, String> {
    // 构建 URL（Gitee 使用 access_token 参数）
    let url = if repo_info.platform == "gitee" {
        format!(
            "{}/repos/{}/{}/contents/{}?ref={}&access_token={}",
            base_url, repo_info.username, repo_info.repo_name, remote_path, branch, token
        )
    } else {
        format!(
            "{}/repos/{}/{}/contents/{}?ref={}",
            base_url, repo_info.username, repo_info.repo_name, remote_path, branch
        )
    };

    let response = if repo_info.platform == "gitee" {
        client
            .get(&url)
            .header("Accept", accept_header)
    } else {
        client
            .get(&url)
            .header("Authorization", format!("token {}", token))
            .header("Accept", accept_header)
    }
    .send()
    .await
    .map_err(|e| format!("获取文件信息失败: {}", e))?;

    if response.status() == 404 {
        return Ok(None);
    }

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("获取文件信息失败: HTTP {} - {}", status, error_text));
    }

    // 先读取响应文本
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 检查是否是空数组（仓库为空或文件不存在）
    if response_text.trim() == "[]" || response_text.trim().is_empty() {
        return Ok(None);
    }

    // 尝试解析 JSON
    match serde_json::from_str::<FileInfo>(&response_text) {
        Ok(info) => Ok(Some(info)),
        Err(e) => {
            // JSON 解析失败
            if response_text.contains("Repository") || response_text.contains("仓库") {
                return Err(format!("仓库不存在或无访问权限"));
            }
            if response_text.contains("Branch") || response_text.contains("分支") {
                return Err(format!("分支 '{}' 不存在，请先在 Gitee 上创建该分支（可添加 README.md 初始化仓库）", branch));
            }
            Err(format!("解析文件信息失败: {} (响应: {})", e, response_text.chars().take(100).collect::<String>()))
        }
    }
}

/// 收集目录下所有文件
fn collect_files(dir: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let mut files = Vec::new();

    // 需要忽略的文件/目录
    const IGNORED_PATTERNS: &[&str] = &[
        ".DS_Store",
        "Thumbs.db",
        ".git",
        ".gitignore",
    ];

    fn should_ignore(path: &Path) -> bool {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        IGNORED_PATTERNS.iter().any(|pattern| {
            file_name == *pattern || file_name.starts_with(pattern)
        })
    }

    fn collect_recursive(
        dir: &Path,
        base: &Path,
        files: &mut Vec<(PathBuf, String)>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            // 跳过需要忽略的文件/目录
            if should_ignore(&path) {
                continue;
            }

            if path.is_dir() {
                collect_recursive(&path, base, files)?;
            } else {
                let relative_path = path
                    .strip_prefix(base)
                    .map_err(|e| format!("计算相对路径失败: {}", e))?
                    .to_str()
                    .ok_or("路径包含无效字符")?
                    .to_string();

                // 使用正斜杠作为路径分隔符
                let normalized_path = relative_path.replace('\\', "/");
                files.push((path, normalized_path));
            }
        }
        Ok(())
    }

    collect_recursive(dir, dir, &mut files)?;
    Ok(files)
}

/// 计算文件 SHA
fn calculate_file_sha(content: &[u8]) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// 保存同步时间
fn save_sync_time(base_dir: &Path) -> Result<(), String> {
    let sync_time_path = base_dir.join("sync_time.json");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("获取系统时间失败: {}", e))?
        .as_secs();

    let content = serde_json::json!({ "last_sync": now });

    fs::write(&sync_time_path, content.to_string())
        .map_err(|e| format!("写入同步时间失败: {}", e))?;

    Ok(())
}
