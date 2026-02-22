use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// 云同步配置（简化版）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiSyncConfig {
    pub repo_url: String,  // 仓库URL (如: https://gitee.com/user/repo.git 或 user/repo)
    pub token: String,     // 访问 Token（用于 Git 认证）
    #[serde(default = "default_branch")]
    pub branch: String,    // 分支（默认 main）
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

/// 检查 Git 是否安装
#[tauri::command]
pub fn check_git_installed() -> Result<bool, String> {
    let output = Command::new("git")
        .arg("--version")
        .output();

    match output {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 验证同步配置
#[tauri::command]
pub async fn validate_sync_config(config: ApiSyncConfig) -> Result<SyncResult, String> {
    // 检查 Git 是否安装
    if !check_git_installed()? {
        return Ok(SyncResult {
            success: false,
            message: "Git 未安装，请先安装 Git: https://git-scm.com/downloads".to_string(),
            details: None,
        });
    }

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

    // 尝试克隆仓库到临时目录测试连接
    let temp_dir = std::env::temp_dir().join("linkron_sync_test");

    // 清理临时目录
    let _ = fs::remove_dir_all(&temp_dir);

    eprintln!("[DEBUG] 测试克隆仓库: {}", config.repo_url);

    // 构建认证 URL
    let auth_url = build_git_auth_url(&config.repo_url, &config.token)?;

    let output = Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(&config.branch)
        .arg(&auth_url)
        .arg(&temp_dir)
        .output();

    // 清理临时目录
    let _ = fs::remove_dir_all(&temp_dir);

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(SyncResult {
                    success: true,
                    message: "连接成功，仓库访问正常".to_string(),
                    details: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                eprintln!("[DEBUG] 克隆失败: {}", stderr);

                if stderr.contains("Authentication failed") || stderr.contains("could not read Username") {
                    Ok(SyncResult {
                        success: false,
                        message: "Token 无效或权限不足".to_string(),
                        details: None,
                    })
                } else if stderr.contains("not found") || stderr.contains("Repository not found") {
                    Ok(SyncResult {
                        success: false,
                        message: "仓库不存在或无访问权限".to_string(),
                        details: None,
                    })
                } else {
                    Ok(SyncResult {
                        success: false,
                        message: format!("连接失败: {}", stderr.trim()),
                        details: None,
                    })
                }
            }
        }
        Err(e) => Ok(SyncResult {
            success: false,
            message: format!("执行 Git 命令失败: {}", e),
            details: None,
        }),
    }
}

/// 构建 Git 认证 URL
fn build_git_auth_url(repo_url: &str, token: &str) -> Result<String, String> {
    let url = repo_url.trim();

    // 如果是完整 URL，插入 Token
    if url.contains("://") {
        if url.contains("@") {
            // 已经包含认证信息
            Ok(url.to_string())
        } else {
            // 插入 Token
            let parts: Vec<&str> = url.splitn(2, "://").collect();
            if parts.len() == 2 {
                Ok(format!("{}://oauth2:{}@{}", parts[0], token, parts[1]))
            } else {
                Err("仓库地址格式无效".to_string())
            }
        }
    } else if !url.contains('/') {
        Err("仓库地址格式无效，应为：用户名/仓库名 或完整的仓库URL".to_string())
    } else {
        // 简短格式，转换为完整 URL
        Ok(format!("https://oauth2:{}@gitee.com/{}.git", token, url))
    }
}


/// 推送到云端
#[tauri::command]
pub async fn sync_to_remote(
    config: ApiSyncConfig,
    work_directory: Option<String>,
) -> Result<SyncResult, String> {
    // 检查 Git 是否安装
    if !check_git_installed()? {
        return Ok(SyncResult {
            success: false,
            message: "Git 未安装，请先安装 Git: https://git-scm.com/downloads".to_string(),
            details: None,
        });
    }

    let base_dir = get_base_directory(&work_directory)?;
    let git_dir = base_dir.join(".git");

    eprintln!("[DEBUG] 开始同步到云端");
    eprintln!("[DEBUG] 工作目录: {:?}", base_dir);

    // 检查是否是 Git 仓库
    if !git_dir.exists() {
        eprintln!("[DEBUG] 不是 Git 仓库，初始化...");

        // 初始化 Git 仓库
        let output = Command::new("git")
            .arg("init")
            .current_dir(&base_dir)
            .output()
            .map_err(|e| format!("初始化 Git 仓库失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("初始化 Git 仓库失败: {}", stderr));
        }

        eprintln!("[DEBUG] Git 仓库初始化成功");
    }

    // 配置远程仓库
    let auth_url = build_git_auth_url(&config.repo_url, &config.token)?;
    eprintln!("[DEBUG] 配置远程仓库: {}", auth_url.replace(&config.token, "***"));

    let output = Command::new("git")
        .arg("remote")
        .arg("set-url")
        .arg("origin")
        .arg(&auth_url)
        .current_dir(&base_dir)
        .output()
        .map_err(|e| format!("配置远程仓库失败: {}", e))?;

    if !output.status.success() && !output.stderr.is_empty() {
        // 远程仓库可能不存在，添加它
        eprintln!("[DEBUG] 远程仓库不存在，添加...");
        let output = Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(&auth_url)
            .current_dir(&base_dir)
            .output()
            .map_err(|e| format!("添加远程仓库失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("添加远程仓库失败: {}", stderr));
        }
    }

    // 添加所有文件
    eprintln!("[DEBUG] 添加文件到暂存区...");
    let output = Command::new("git")
        .arg("add")
        .arg("-A")
        .current_dir(&base_dir)
        .output()
        .map_err(|e| format!("添加文件失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("添加文件失败: {}", stderr));
    }

    // 提交更改（即使没有更改也尝试提交，可能会失败但没关系）
    eprintln!("[DEBUG] 提交更改...");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("获取时间失败: {}", e))?
        .as_secs();

    let commit_output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(format!("Sync at {}", timestamp))
        .arg("--allow-empty")
        .current_dir(&base_dir)
        .output()
        .map_err(|e| format!("提交失败: {}", e))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        // 如果是因为没有更改，继续推送
        if !stderr.contains("nothing to commit") {
            return Err(format!("提交失败: {}", stderr));
        }
        eprintln!("[DEBUG] 没有新更改需要提交");
    } else {
        eprintln!("[DEBUG] 提交成功");
    }

    // 推送到远程
    eprintln!("[DEBUG] 推送到远程分支: {}", config.branch);

    // 先强制拉取远程更新
    eprintln!("[DEBUG] 强制拉取远程更新...");

    // 配置使用 merge 策略
    let _ = Command::new("git")
        .arg("config")
        .arg("pull.rebase")
        .arg("false")
        .current_dir(&base_dir)
        .output();

    let pull_output = Command::new("git")
        .arg("pull")
        .arg("origin")
        .arg(&config.branch)
        .arg("--allow-unrelated-histories")
        .arg("--no-ff")
        .arg("-X")
        .arg("theirs")
        .current_dir(&base_dir)
        .output();

    match pull_output {
        Ok(result) => {
            if result.status.success() {
                eprintln!("[DEBUG] 拉取成功，已采用远程版本");
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                eprintln!("[DEBUG] 拉取失败或无需拉取: {}", stderr.trim());
                // 如果拉取失败（比如远程为空），继续尝试推送
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] 拉取命令执行失败，继续推送: {}", e);
        }
    }

    // 重新添加所有文件并提交（拉取后可能有新文件）
    eprintln!("[DEBUG] 拉取后重新添加文件...");
    let output = Command::new("git")
        .arg("add")
        .arg("-A")
        .current_dir(&base_dir)
        .output()
        .map_err(|e| format!("添加文件失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("添加文件失败: {}", stderr));
    }

    // 再次提交（合并拉取后的更改）
    eprintln!("[DEBUG] 提交合并后的更改...");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("获取时间失败: {}", e))?
        .as_secs();

    let commit_output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(format!("Sync at {}", timestamp))
        .arg("--allow-empty")
        .current_dir(&base_dir)
        .output()
        .map_err(|e| format!("提交失败: {}", e))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        if !stderr.contains("nothing to commit") {
            return Err(format!("提交失败: {}", stderr));
        }
        eprintln!("[DEBUG] 没有新更改需要提交");
    } else {
        eprintln!("[DEBUG] 提交成功");
    }

    // 现在推送
    let output = Command::new("git")
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg(&config.branch)
        .current_dir(&base_dir)
        .output()
        .map_err(|e| format!("推送失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[DEBUG] 推送失败: {}", stderr);

        if stderr.contains("Authentication failed") {
            return Err(format!("推送失败：Token 无效或权限不足"));
        } else if stderr.contains("remote rejected") {
            return Err(format!("推送失败：远程拒绝。可能是因为远程有更新，请先执行拉取操作"));
        } else {
            return Err(format!("推送失败: {}", stderr.trim()));
        }
    }

    eprintln!("[DEBUG] 推送成功");

    Ok(SyncResult {
        success: true,
        message: "同步成功".to_string(),
        details: Some(SyncDetails {
            uploaded: 1,
            downloaded: 0,
            skipped: 0,
            total: 1,
        }),
    })
}

/// 从云端拉取
#[tauri::command]
pub async fn sync_from_remote(
    config: ApiSyncConfig,
    work_directory: Option<String>,
) -> Result<SyncResult, String> {
    // 检查 Git 是否安装
    if !check_git_installed()? {
        return Ok(SyncResult {
            success: false,
            message: "Git 未安装，请先安装 Git: https://git-scm.com/downloads".to_string(),
            details: None,
        });
    }

    let base_dir = get_base_directory(&work_directory)?;
    let git_dir = base_dir.join(".git");

    eprintln!("[DEBUG] 开始从云端拉取");
    eprintln!("[DEBUG] 工作目录: {:?}", base_dir);

    // 检查是否是 Git 仓库
    if !git_dir.exists() {
        // 不是 Git 仓库，需要先克隆
        eprintln!("[DEBUG] 不是 Git 仓库，开始克隆...");

        let auth_url = build_git_auth_url(&config.repo_url, &config.token)?;
        eprintln!("[DEBUG] 克隆 URL: {}", auth_url.replace(&config.token, "***"));

        // 备份现有文件
        let backup_dir = std::env::temp_dir().join("linkron_backup");
        let _ = fs::remove_dir_all(&backup_dir);

        if base_dir.exists() {
            let _ = fs::create_dir_all(&backup_dir);
            for entry in fs::read_dir(&base_dir).map_err(|e| format!("读取目录失败: {}", e))? {
                let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
                let path = entry.path();
                if path.is_dir() && path.file_name().map(|n| n != ".git").unwrap_or(true) {
                    let dest = backup_dir.join(entry.file_name());
                    fs::rename(&path, &dest).map_err(|e| format!("备份失败: {}", e))?;
                }
            }
        }

        // 克隆仓库
        let output = Command::new("git")
            .arg("clone")
            .arg("--branch")
            .arg(&config.branch)
            .arg(&auth_url)
            .arg(&base_dir)
            .output()
            .map_err(|e| format!("克隆失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // 恢复备份
            if backup_dir.exists() {
                for entry in fs::read_dir(&backup_dir).map_err(|e| format!("读取备份目录失败: {}", e))? {
                    let entry = entry.map_err(|e| format!("读取备份项失败: {}", e))?;
                    let src = entry.path();
                    let dest = base_dir.join(entry.file_name());
                    let _ = fs::rename(&src, &dest);
                }
            }

            return Err(format!("克隆失败: {}", stderr.trim()));
        }

        eprintln!("[DEBUG] 克隆成功");
    } else {
        // 是 Git 仓库，拉取更新
        eprintln!("[DEBUG] 是 Git 仓库，拉取更新...");

        // 配置远程仓库
        let auth_url = build_git_auth_url(&config.repo_url, &config.token)?;
        let _ = Command::new("git")
            .arg("remote")
            .arg("set-url")
            .arg("origin")
            .arg(&auth_url)
            .current_dir(&base_dir)
            .output()
            .map_err(|e| format!("配置远程仓库失败: {}", e))?;

        // 拉取更新
        let output = Command::new("git")
            .arg("pull")
            .arg("origin")
            .arg(&config.branch)
            .current_dir(&base_dir)
            .output()
            .map_err(|e| format!("拉取失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[DEBUG] 拉取失败: {}", stderr);

            if stderr.contains("Authentication failed") {
                return Err(format!("拉取失败：Token 无效或权限不足"));
            } else if stderr.contains("conflict") {
                return Err(format!("拉取失败：存在冲突，请手动解决"));
            } else {
                return Err(format!("拉取失败: {}", stderr.trim()));
            }
        }

        eprintln!("[DEBUG] 拉取成功");
    }

    Ok(SyncResult {
        success: true,
        message: "同步成功".to_string(),
        details: Some(SyncDetails {
            uploaded: 0,
            downloaded: 1,
            skipped: 0,
            total: 1,
        }),
    })
}
