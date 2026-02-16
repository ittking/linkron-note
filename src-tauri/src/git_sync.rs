use git2::{Cred, FetchOptions, ObjectType, RemoteCallbacks, Repository, Signature};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncConfig {
    pub platform: String, // "gitee", "github", "custom"
    pub token: String,
    pub repo_url: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub commit_id: Option<String>,
}

/// 获取 Git 仓库路径
fn get_repo_path(work_directory: Option<String>) -> Result<PathBuf, String> {
    let base_path = if let Some(work_dir) = work_directory {
        PathBuf::from(work_dir)
    } else {
        dirs::data_local_dir()
            .ok_or("Failed to get app data directory")?
            .join("iterm")
    };

    // 确保目录存在
    fs::create_dir_all(&base_path)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    Ok(base_path)
}

/// 检测 Git 仓库连接
#[tauri::command]
pub async fn test_git_connection(config: SyncConfig, work_directory: Option<String>) -> Result<SyncResult, String> {
    let repo_path = get_repo_path(work_directory)?;

    // 创建临时目录用于测试
    let test_path = repo_path.join(".git_test_clone");
    
    // 清理可能存在的测试目录
    if test_path.exists() {
        fs::remove_dir_all(&test_path)
            .map_err(|e| format!("Failed to clean test directory: {}", e))?;
    }

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::userpass_plaintext(username_from_url.unwrap(), &config.token)
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    match Repository::clone(&config.repo_url, &test_path) {
        Ok(_) => {
            // 清理测试目录
            fs::remove_dir_all(&test_path)
                .map_err(|e| format!("Failed to clean test directory: {}", e))?;
            
            Ok(SyncResult {
                success: true,
                message: "连接成功".to_string(),
                commit_id: None,
            })
        }
        Err(e) => {
            // 清理测试目录
            if test_path.exists() {
                let _ = fs::remove_dir_all(&test_path);
            }
            Ok(SyncResult {
                success: false,
                message: format!("连接失败: {}", e),
                commit_id: None,
            })
        }
    }
}

/// 初始化 Git 仓库
fn init_repo(repo_path: &Path) -> Result<Repository, String> {
    let git_path = repo_path.join(".git");

    if git_path.exists() {
        Repository::open(repo_path).map_err(|e| format!("Failed to open repository: {}", e))
    } else {
        Repository::init(repo_path).map_err(|e| format!("Failed to init repository: {}", e))
    }
}

/// 添加远程仓库
fn add_remote(repo: &Repository, name: &str, url: &str, token: &str) -> Result<(), String> {
    // 移除已存在的远程仓库
    if repo.find_remote(name).is_ok() {
        repo.remote_delete(name)
            .map_err(|e| format!("Failed to delete remote: {}", e))?;
    }

    // 添加远程仓库（带 token）
    let url_with_token = if url.contains("github.com") {
        url.replace("https://", &format!("https://{}@", token))
    } else if url.contains("gitee.com") {
        url.replace("https://", &format!("https://{}@", token))
    } else {
        // 自定义仓库，尝试替换
        if url.contains("://") {
            let parts: Vec<&str> = url.split("://").collect();
            if parts.len() == 2 {
                format!("{}://{}@{}", parts[0], token, parts[1])
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        }
    };

    repo.remote(name, &url_with_token)
        .map_err(|e| format!("Failed to add remote: {}", e))?;

    Ok(())
}

/// 执行 Git 同步（推送）
#[tauri::command]
pub async fn sync_to_remote(config: SyncConfig, work_directory: Option<String>) -> Result<SyncResult, String> {
    let repo_path = get_repo_path(work_directory)?;
    let repo = init_repo(&repo_path)?;

    // 添加远程仓库
    add_remote(&repo, "origin", &config.repo_url, &config.token)?;

    // 添加所有文件
    let mut index = repo.index().map_err(|e| format!("Failed to get index: {}", e))?;
    index.add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Failed to add files: {}", e))?;
    index.write().map_err(|e| format!("Failed to write index: {}", e))?;

    // 创建树
    let tree_id = index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?;
    let tree = repo.find_tree(tree_id).map_err(|e| format!("Failed to find tree: {}", e))?;

    // 获取 HEAD
    let head = repo.head().ok();
    let parent_commit = head.as_ref().and_then(|h| h.peel_to_commit().ok());

    // 创建签名
    let sig = Signature::now("iFlow CLI", "iterm@local")
        .map_err(|e| format!("Failed to create signature: {}", e))?;

    // 创建提交
    let parents_slice: Vec<&git2::Commit> = parent_commit.as_ref().map(|c| vec![c as &git2::Commit]).unwrap_or_default();
    let commit_id = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Auto sync from iFlow CLI",
        &tree,
        parents_slice.as_slice(),
    ).map_err(|e| format!("Failed to commit: {}", e))?;

    // 推送到远程
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::userpass_plaintext(username_from_url.unwrap(), &config.token)
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    let branch_name = config.branch.clone().unwrap_or_else(|| "main".to_string());
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

    let mut remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find remote: {}", e))?;
    remote.push(&[&refspec], Some(&mut push_options))
        .map_err(|e| format!("Failed to push: {}", e))?;

    Ok(SyncResult {
        success: true,
        message: "同步成功".to_string(),
        commit_id: Some(commit_id.to_string()),
    })
}

/// 执行 Git 同步（拉取）
#[tauri::command]
pub async fn sync_from_remote(config: SyncConfig, work_directory: Option<String>) -> Result<SyncResult, String> {
    let repo_path = get_repo_path(work_directory)?;
    let repo = init_repo(&repo_path)?;

    // 添加远程仓库
    add_remote(&repo, "origin", &config.repo_url, &config.token)?;

    // 拉取远程更新
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::userpass_plaintext(username_from_url.unwrap(), &config.token)
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find remote: {}", e))?;
    remote.fetch(&["main"], Some(&mut fetch_options), None)
        .map_err(|e| format!("Failed to fetch: {}", e))?;

    // 获取远程 HEAD
    let remote_ref = repo.refname_to_id("refs/remotes/origin/main")
        .map_err(|e| format!("Failed to get remote ref: {}", e))?;
    let fetch_head = repo.find_object(remote_ref, Some(ObjectType::Commit))
        .map_err(|e| format!("Failed to find fetch head: {}", e))?;

    // 合并到本地
    let fetch_commit = fetch_head.peel_to_commit()
        .map_err(|e| format!("Failed to peel to commit: {}", e))?;

    let repo_head = repo.head().ok();
    if let Some(head) = repo_head {
        let annotated = repo.reference_to_annotated_commit(&head)
            .map_err(|e| format!("Failed to get annotated commit: {}", e))?;

        repo.merge(&[&annotated], None, None)
            .map_err(|e| format!("Failed to merge: {}", e))?;

        // 创建合并提交
        let sig = Signature::now("iFlow CLI", "iterm@local")
            .map_err(|e| format!("Failed to create signature: {}", e))?;

        let local_commit_id = head.target()
            .ok_or("Failed to get local commit ID")?;
        let local_commit = repo.find_commit(local_commit_id)
            .map_err(|e| format!("Failed to find local commit: {}", e))?;

        let mut merge_opts = git2::MergeOptions::new();
        let mut merge_index = repo.merge_commits(&local_commit, &fetch_commit, Some(&mut merge_opts))
            .map_err(|e| format!("Failed to merge commits: {}", e))?;

        if merge_index.has_conflicts() {
            return Err("存在合并冲突，请手动解决".to_string());
        }

        let tree_id = merge_index.write_tree_to(&repo)
            .map_err(|e| format!("Failed to write tree: {}", e))?;

        let tree = repo.find_tree(tree_id)
            .map_err(|e| format!("Failed to find tree: {}", e))?;

        let commit_id = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Merge remote changes",
            &tree,
            &[&local_commit, &fetch_commit],
        ).map_err(|e| format!("Failed to commit merge: {}", e))?;

        repo.cleanup_state()
            .map_err(|e| format!("Failed to cleanup state: {}", e))?;

        Ok(SyncResult {
            success: true,
            message: "拉取成功".to_string(),
            commit_id: Some(commit_id.to_string()),
        })
    } else {
        // 本地没有提交，直接检出
        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force();
        repo.checkout_tree(&fetch_head, Some(&mut checkout_builder))
            .map_err(|e| format!("Failed to checkout tree: {}", e))?;

        repo.set_head("refs/heads/main")
            .map_err(|e| format!("Failed to set head: {}", e))?;

        Ok(SyncResult {
            success: true,
            message: "拉取成功".to_string(),
            commit_id: Some(fetch_head.id().to_string()),
        })
    }
}

/// 获取同步配置
#[tauri::command]
pub async fn get_sync_config(work_directory: Option<String>) -> Result<Option<SyncConfig>, String> {
    let repo_path = get_repo_path(work_directory)?;
    let config_file = repo_path.join(".sync_config.json");

    if !config_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&config_file)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: SyncConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(Some(config))
}

/// 保存同步配置
#[tauri::command]
pub async fn save_sync_config(config: SyncConfig, work_directory: Option<String>) -> Result<(), String> {
    let repo_path = get_repo_path(work_directory)?;
    let config_file = repo_path.join(".sync_config.json");

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_file, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// 检查 Git 仓库状态
#[tauri::command]
pub async fn get_git_status(work_directory: Option<String>) -> Result<bool, String> {
    let repo_path = get_repo_path(work_directory)?;
    let git_path = repo_path.join(".git");

    Ok(git_path.exists())
}