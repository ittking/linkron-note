use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;

// 笔记数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    #[serde(rename = "type")]
    pub note_type: String,
    pub content: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    pub images: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

// 创建笔记的数据结构
#[derive(Debug, Deserialize)]
pub struct NoteData {
    #[serde(rename = "type")]
    pub note_type: Option<String>,
    pub content: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    pub images: Option<Vec<String>>,
}

// 更新笔记的数据结构
#[derive(Debug, Deserialize)]
pub struct NoteUpdate {
    pub content: Option<String>,
    pub images: Option<Vec<String>>,
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
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL DEFAULT 'text',
                content TEXT NOT NULL,
                source_url TEXT,
                images TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 创建索引
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_type ON notes(type)",
            [],
        )?;

        Ok(())
    }

    /// 获取所有笔记
    pub fn get_all_notes(&self) -> SqliteResult<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, type, content, source_url, images, created_at, updated_at
             FROM notes ORDER BY updated_at DESC"
        )?;

        let notes = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                note_type: row.get(1)?,
                content: row.get(2)?,
                source_url: row.get(3)?,
                images: serde_json::from_str(row.get::<_, String>(4)?.as_str()).unwrap_or_default(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        notes.collect()
    }

    /// 获取单个笔记
    pub fn get_note(&self, id: &str) -> SqliteResult<Option<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, type, content, source_url, images, created_at, updated_at
             FROM notes WHERE id = ?"
        )?;

        let mut notes = stmt.query_map(params![id], |row| {
            Ok(Note {
                id: row.get(0)?,
                note_type: row.get(1)?,
                content: row.get(2)?,
                source_url: row.get(3)?,
                images: serde_json::from_str(row.get::<_, String>(4)?.as_str()).unwrap_or_default(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        match notes.next() {
            Some(Ok(note)) => Ok(Some(note)),
            _ => Ok(None),
        }
    }

    /// 创建笔记
    pub fn create_note(&self, note_data: NoteData) -> SqliteResult<Note> {
        let id = Ulid::new().to_string();
        let note_type = note_data.note_type.unwrap_or_else(|| "text".to_string());
        let now = chrono::Utc::now().to_rfc3339();

        let images = note_data.images.unwrap_or_default();
        let images_json = serde_json::to_string(&images).unwrap();

        self.conn.execute(
            "INSERT INTO notes (id, type, content, source_url, images, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &id,
                &note_type,
                &note_data.content,
                &note_data.source_url,
                &images_json,
                &now,
                &now
            ],
        )?;

        Ok(Note {
            id,
            note_type,
            content: note_data.content,
            source_url: note_data.source_url,
            images,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 更新笔记
    pub fn update_note(&self, id: &str, updates: NoteUpdate) -> SqliteResult<Note> {
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(content) = &updates.content {
            self.conn.execute(
                "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content, &now, id],
            )?;
        }

        if let Some(images) = &updates.images {
            let images_json = serde_json::to_string(images).unwrap();
            self.conn.execute(
                "UPDATE notes SET images = ?1, updated_at = ?2 WHERE id = ?3",
                params![&images_json, &now, id],
            )?;
        }

        // 返回更新后的笔记
        self.get_note(id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
    }

    /// 删除笔记
    pub fn delete_note(&self, id: &str) -> SqliteResult<()> {
        self.conn.execute("DELETE FROM notes WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 搜索笔记
    pub fn search_notes(&self, keyword: &str) -> SqliteResult<Vec<Note>> {
        let search_pattern = format!("%{}%", keyword);

        let mut stmt = self.conn.prepare(
            "SELECT id, type, content, source_url, images, created_at, updated_at
             FROM notes WHERE content LIKE ?1
             ORDER BY updated_at DESC"
        )?;

        let notes = stmt.query_map(params![search_pattern], |row| {
            Ok(Note {
                id: row.get(0)?,
                note_type: row.get(1)?,
                content: row.get(2)?,
                source_url: row.get(3)?,
                images: serde_json::from_str(row.get::<_, String>(4)?.as_str()).unwrap_or_default(),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        notes.collect()
    }

    /// 获取笔记数量
    pub fn count_notes(&self) -> SqliteResult<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
    }
}

/// 获取数据库路径
fn get_database_path(work_directory: Option<String>) -> Result<String, String> {
    if let Some(work_dir) = work_directory {
        // 使用工作目录
        let mut path = PathBuf::from(work_dir);
        path.push("notes.db");
        Ok(path.to_string_lossy().to_string())
    } else {
        // 使用应用数据目录
        let app_data_dir = dirs::data_local_dir()
            .ok_or("Failed to get app data directory")?;
        let mut path = PathBuf::from(app_data_dir);
        path.push("iterm");
        
        // 确保目录存在
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        
        path.push("notes.db");
        Ok(path.to_string_lossy().to_string())
    }
}

/// 获取 JSON 文件路径
fn get_json_path(work_directory: Option<String>) -> Result<String, String> {
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

/// Tauri 命令：初始化数据库
#[tauri::command]
pub async fn init_database(work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    Database::new(&db_path).map_err(|e| format!("Failed to init database: {}", e))?;
    Ok(())
}

/// Tauri 命令：获取所有笔记
#[tauri::command]
pub async fn get_all_notes(work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_all_notes().map_err(|e| format!("Failed to get notes: {}", e))
}

/// Tauri 命令：获取单个笔记
#[tauri::command]
pub async fn get_note(id: String, work_directory: Option<String>) -> Result<Option<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_note(&id).map_err(|e| format!("Failed to get note: {}", e))
}

/// Tauri 命令：创建笔记
#[tauri::command]
pub async fn create_note(note_data: NoteData, work_directory: Option<String>) -> Result<Note, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.create_note(note_data).map_err(|e| format!("Failed to create note: {}", e))
}

/// Tauri 命令：更新笔记
#[tauri::command]
pub async fn update_note(id: String, updates: NoteUpdate, work_directory: Option<String>) -> Result<Note, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.update_note(&id, updates).map_err(|e| format!("Failed to update note: {}", e))
}

/// Tauri 命令：删除笔记
#[tauri::command]
pub async fn delete_note(id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.delete_note(&id).map_err(|e| format!("Failed to delete note: {}", e))
}

/// Tauri 命令：搜索笔记
#[tauri::command]
pub async fn search_notes(keyword: String, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.search_notes(&keyword).map_err(|e| format!("Failed to search notes: {}", e))
}

/// Tauri 命令：从 JSON 迁移到 SQLite
#[tauri::command]
pub async fn migrate_from_json(work_directory: Option<String>) -> Result<usize, String> {
    let json_path = get_json_path(work_directory.clone())?;
    let db_path = get_database_path(work_directory)?;

    // 检查 JSON 文件是否存在
    if !PathBuf::from(&json_path).exists() {
        return Err("JSON file not found".to_string());
    }

    // 读取 JSON 文件
    let json_content = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;

    // 解析 JSON
    let json_notes: Vec<serde_json::Value> = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if json_notes.is_empty() {
        return Ok(0);
    }

    // 初始化数据库
    let db = Database::new(&db_path)
        .map_err(|e| format!("Failed to init database: {}", e))?;

    // 检查数据库是否已有数据
    let count = db.count_notes()
        .map_err(|e| format!("Failed to check database: {}", e))?;

    if count > 0 {
        return Err("Database already has data".to_string());
    }

    // 迁移数据
    let mut migrated_count = 0;
    for note_json in json_notes {
        let note_data = NoteData {
            note_type: note_json.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
            content: note_json.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_url: note_json.get("sourceUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
            images: note_json.get("images").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            }),
        };

        db.create_note(note_data)
            .map_err(|e| format!("Failed to migrate note: {}", e))?;
        migrated_count += 1;
    }

    // 备份 JSON 文件
    let backup_path = format!("{}.backup", json_path);
    std::fs::rename(&json_path, &backup_path)
        .map_err(|e| format!("Failed to backup JSON file: {}", e))?;

    Ok(migrated_count)
}