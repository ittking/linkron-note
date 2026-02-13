use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Import submodules
use crate::note;
use crate::tag;
use crate::todo;

// Re-export types from submodules
pub use crate::note::{Note, NoteData, NoteUpdate};
pub use crate::tag::Tag;
pub use crate::todo::Todo;

// 热度图数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonthData {
    pub year: i32,
    pub month: i32,
    pub weeks: [i32; 5], // 每月5周的笔记数量
}

// 数据库管理器
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// 初始化数据库表
    fn init_tables(&self) -> SqliteResult<()> {
        note::init_tables(&self.conn)?;
        tag::init_tables(&self.conn)?;
        todo::init_tables(&self.conn)?;
        Ok(())
    }
}

/// 获取数据库路径
pub fn get_database_path(work_directory: Option<String>) -> Result<String, String> {
    if let Some(work_dir) = work_directory {
        let mut path = PathBuf::from(work_dir);
        path.push("notes.db");
        Ok(path.to_string_lossy().to_string())
    } else {
        let app_data_dir = dirs::data_local_dir()
            .ok_or("Failed to get app data directory")?;
        let mut path = PathBuf::from(app_data_dir);
        path.push("iterm");

        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        path.push("notes.db");
        Ok(path.to_string_lossy().to_string())
    }
}

/// 获取 JSON 文件路径
pub fn get_json_path(work_directory: Option<String>) -> Result<String, String> {
    if let Some(work_dir) = work_directory {
        let mut path = PathBuf::from(work_dir);
        path.push("notes.json");
        Ok(path.to_string_lossy().to_string())
    } else {
        let app_data_dir = dirs::data_local_dir()
            .ok_or("Failed to get app data directory")?;
        let mut path = PathBuf::from(app_data_dir);
        path.push("iterm");
        path.push("notes.json");
        Ok(path.to_string_lossy().to_string())
    }
}

// ========== Tauri 命令 ==========

/// Tauri 命令：初始化数据库
#[tauri::command]
pub async fn init_database(work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    Database::new(&db_path).map_err(|e| format!("Failed to init database: {}", e))?;
    Ok(())
}

/// Tauri 命令：获取所有笔记（分页）
#[tauri::command]
pub async fn get_all_notes(page: u32, page_size: u32, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::get_all_notes(&db.conn, page, page_size).map_err(|e| format!("Failed to get notes: {}", e))
}

/// Tauri 命令：获取单个笔记
#[tauri::command]
pub async fn get_note(id: String, work_directory: Option<String>) -> Result<Option<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::get_note(&db.conn, &id).map_err(|e| format!("Failed to get note: {}", e))
}

/// Tauri 命令：创建笔记
#[tauri::command]
pub async fn create_note(note_data: NoteData, work_directory: Option<String>) -> Result<Note, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    
    let note = note::create_note(&db.conn, note_data).map_err(|e| format!("Failed to create note: {}", e))?;
    
    // 同步标签
    let tags = tag::parse_tags_from_content(&db.conn, &note.content);
    if !tags.is_empty() {
        tag::create_or_update_tags(&db.conn, tags).map_err(|e| format!("Failed to sync tags: {}", e))?;
    }
    
    Ok(note)
}

/// Tauri 命令：更新笔记
#[tauri::command]
pub async fn update_note(id: String, updates: NoteUpdate, work_directory: Option<String>) -> Result<Note, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    
    note::update_note(&db.conn, &id, updates.clone()).map_err(|e| format!("Failed to update note: {}", e))?;
    
    // 同步标签
    if let Some(content) = &updates.content {
        let tags = tag::parse_tags_from_content(&db.conn, content);
        if !tags.is_empty() {
            tag::create_or_update_tags(&db.conn, tags).map_err(|e| format!("Failed to sync tags: {}", e))?;
        }
    }
    
    note::get_note(&db.conn, &id)
        .map_err(|e| format!("Failed to get updated note: {}", e))?
        .ok_or_else(|| format!("Note not found: {}", id))
}

/// Tauri 命令：删除笔记
#[tauri::command]
pub async fn delete_note(id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory.clone())?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let note = note::get_note(&db.conn, &id)
        .map_err(|e| format!("Failed to get note: {}", e))?
        .ok_or_else(|| format!("Note not found: {}", id))?;

    // 删除笔记关联的资源文件
    crate::filesystem::delete_resources_from_note(&note, &work_directory);

    note::delete_note(&db.conn, &id).map_err(|e| format!("Failed to delete note: {}", e))
}

/// Tauri 命令：搜索笔记
#[tauri::command]
pub async fn search_notes(keyword: String, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::search_notes(&db.conn, &keyword).map_err(|e| format!("Failed to search notes: {}", e))
}

/// Tauri 命令：从 JSON 迁移到 SQLite
#[tauri::command]
pub async fn migrate_from_json(work_directory: Option<String>) -> Result<usize, String> {
    let json_path = get_json_path(work_directory.clone())?;
    let db_path = get_database_path(work_directory)?;

    if !PathBuf::from(&json_path).exists() {
        return Err("JSON file not found".to_string());
    }

    let json_content = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;

    let json_notes: Vec<serde_json::Value> = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if json_notes.is_empty() {
        return Ok(0);
    }

    let db = Database::new(&db_path)
        .map_err(|e| format!("Failed to init database: {}", e))?;

    let count = note::count_notes(&db.conn)
        .map_err(|e| format!("Failed to check database: {}", e))?;

    if count > 0 {
        return Err("Database already has data".to_string());
    }

    let mut migrated_count = 0;
    for note_json in json_notes {
        let note_data = NoteData {
            note_type: note_json.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
            content: note_json.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_url: note_json.get("sourceUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
            extract_url: note_json.get("extractUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
            images: vec![],
        };

        let note = note::create_note(&db.conn, note_data)
            .map_err(|e| format!("Failed to migrate note: {}", e))?;
        
        // 同步标签
        let tags = tag::parse_tags_from_content(&db.conn, &note.content);
        if !tags.is_empty() {
            tag::create_or_update_tags(&db.conn, tags).map_err(|e| format!("Failed to sync tags: {}", e))?;
        }
        
        migrated_count += 1;
    }

    let backup_path = format!("{}.backup", json_path);
    std::fs::rename(&json_path, &backup_path)
        .map_err(|e| format!("Failed to backup JSON file: {}", e))?;

    Ok(migrated_count)
}

// ========== 标签相关 Tauri 命令 ==========

/// Tauri 命令：解析标签
#[tauri::command]
pub fn parse_tags(content: String, work_directory: Option<String>) -> Result<Vec<String>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    Ok(tag::parse_tags_from_content(&db.conn, &content))
}

/// Tauri 命令：同步标签
#[tauri::command]
pub fn sync_tags(tags: Vec<String>, work_directory: Option<String>) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    tag::create_or_update_tags(&db.conn, tags).map_err(|e| format!("Failed to sync tags: {}", e))
}

/// Tauri 命令：获取所有标签
#[tauri::command]
pub fn get_all_tags(work_directory: Option<String>) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    tag::get_all_tags(&db.conn).map_err(|e| format!("Failed to get tags: {}", e))
}

/// Tauri 命令：删除标签
#[tauri::command]
pub fn delete_tag(id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    tag::delete_tag(&db.conn, &id).map_err(|e| format!("Failed to delete tag: {}", e))
}

/// Tauri 命令：置顶标签
#[tauri::command]
pub fn pin_tag(id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    tag::pin_tag(&db.conn, &id).map_err(|e| format!("Failed to pin tag: {}", e))
}

/// Tauri 命令：置顶笔记
#[tauri::command]
pub fn pin_note(id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::pin_note(&db.conn, &id).map_err(|e| format!("Failed to pin note: {}", e))
}

/// Tauri 命令：根据标签筛选笔记
#[tauri::command]
pub fn get_notes_by_tags(tags: Vec<String>, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::get_notes_by_tags(&db.conn, tags).map_err(|e| format!("Failed to get notes by tags: {}", e))
}

/// Tauri 命令：根据标签获取笔记数量
#[tauri::command]
pub fn count_notes_by_tags(tags: Vec<String>, work_directory: Option<String>) -> Result<i64, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::count_notes_by_tags(&db.conn, tags).map_err(|e| format!("Failed to count notes by tags: {}", e))
}

/// Tauri 命令：获取笔记总数
#[tauri::command]
pub fn count_notes(work_directory: String) -> Result<i64, String> {
    let db_path = get_database_path(Some(work_directory))?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::count_notes(&db.conn).map_err(|e| format!("Failed to count notes: {}", e))
}

/// Tauri 命令：获取笔记热度图数据
#[tauri::command]
pub fn get_notes_heatmap(work_directory: Option<String>) -> Result<Vec<MonthData>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    note::get_notes_heatmap(&db.conn).map_err(|e| format!("Failed to get notes heatmap: {}", e))
}

/// Tauri 命令：搜索标签
#[tauri::command]
pub fn search_tags(work_directory: String, query: String) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(Some(work_directory))?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    tag::search_tags(&db.conn, &query).map_err(|e| format!("Failed to search tags: {}", e))
}

// ========== 待办相关 Tauri 命令 ==========

/// Tauri 命令：创建待办事项
#[tauri::command]
pub async fn create_todo(date: String, text: String, status: String, reminder: Option<String>, work_directory: Option<String>) -> Result<i64, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    todo::create_todo(&db.conn, &date, &text, &status, reminder).map_err(|e| format!("Failed to create todo: {}", e))
}

/// Tauri 命令：更新待办事项
#[tauri::command]
pub async fn update_todo(id: i64, text: String, status: String, reminder: Option<String>, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    todo::update_todo(&db.conn, id, &text, &status, reminder).map_err(|e| format!("Failed to update todo: {}", e))
}

/// Tauri 命令：删除待办事项
#[tauri::command]
pub async fn delete_todo(id: i64, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    todo::delete_todo(&db.conn, id).map_err(|e| format!("Failed to delete todo: {}", e))
}

/// Tauri 命令：获取指定日期的待办事项
#[tauri::command]
pub async fn get_todos_by_date(date: String, work_directory: Option<String>) -> Result<Vec<Todo>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    todo::get_todos_by_date(&db.conn, &date).map_err(|e| format!("Failed to get todos: {}", e))
}

/// Tauri 命令：获取指定月份的待办事项
#[tauri::command]
pub async fn get_todos_by_month(year: i32, month: i32, work_directory: Option<String>) -> Result<Vec<Todo>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    todo::get_todos_by_month(&db.conn, year, month).map_err(|e| format!("Failed to get todos: {}", e))
}

/// Tauri 命令：获取需要提醒的待办事项
#[tauri::command]
pub async fn get_reminders(work_directory: Option<String>) -> Result<Vec<Todo>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    let result = todo::get_reminders(&db.conn).map_err(|e| format!("Failed to get reminders: {}", e))?;
    Ok(result)
}

/// Tauri 命令：统计待办事项数量
#[tauri::command]
pub async fn count_todos(work_directory: Option<String>) -> Result<i64, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    todo::count_todos(&db.conn).map_err(|e| format!("Failed to count todos: {}", e))
}

/// Tauri 命令：获取今日相关的待办事项
/// 包括今日创建的待办和提醒日期是今天的非重复提醒待办
#[tauri::command]
pub async fn get_today_todos(today_date: String, work_directory: Option<String>) -> Result<Vec<Todo>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    todo::get_today_todos(&db.conn, &today_date).map_err(|e| format!("Failed to get today todos: {}", e))
}