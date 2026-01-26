use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;
use regex::Regex;

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

// 标签数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub path: String,
    pub level: i32,
    pub color: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

// 标签统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TagStats {
    #[serde(flatten)]
    pub tag: Tag,
    pub count: i64,
}

// 创建标签的数据结构
#[derive(Debug, Deserialize)]
pub struct TagData {
    pub name: String,
    pub color: Option<String>,
}

// 笔记标签关联数据
#[derive(Debug, Deserialize)]
pub struct NoteTagData {
    pub note_id: String,
    pub tag_id: String,
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
        // 创建 notes 表
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

        // 创建 tags 表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                path TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                color TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 创建 note_tags 关联表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS note_tags (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
                UNIQUE(note_id, tag_id)
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

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_path ON tags(path)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_level ON tags(level)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_note_tags_note_id ON note_tags(note_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_note_tags_tag_id ON note_tags(tag_id)",
            [],
        )?;

        Ok(())
    }

    /// 获取所有笔记（分页）
    pub fn get_all_notes(&self, page: u32, page_size: u32) -> SqliteResult<Vec<Note>> {
        let offset = (page - 1) * page_size;
        let mut stmt = self.conn.prepare(
            "SELECT id, type, content, source_url, images, created_at, updated_at
             FROM notes ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )?;

        let notes = stmt.query_map(params![page_size, offset], |row| {
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

    /// 获取笔记总数
    pub fn count_notes(&self) -> SqliteResult<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
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

        self.parse_and_create_tags(&id, &note_data.content)?;

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
        // 先检查笔记是否存在
        let _existing_note = self.get_note(id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

        let now = chrono::Utc::now().to_rfc3339();

        if let Some(content) = &updates.content {
            self.conn.execute(
                "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content, &now, id],
            )?;
            
            // 删除旧的标签关联
            self.conn.execute(
                "DELETE FROM note_tags WHERE note_id = ?1",
                params![id],
            )?;
            
            // 解析并创建新的标签关联
            self.parse_and_create_tags(id, content)?;
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

/// Tauri 命令：获取所有笔记（分页）
#[tauri::command]
pub async fn get_all_notes(page: u32, page_size: u32, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_all_notes(page, page_size).map_err(|e| format!("Failed to get notes: {}", e))
}

/// Tauri 命令：获取笔记总数
#[tauri::command]
pub async fn get_notes_count(work_directory: Option<String>) -> Result<i64, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.count_notes().map_err(|e| format!("Failed to count notes: {}", e))
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

// ========== 标签相关函数 ==========

impl Database {
    /// 解析多级标签名称
    /// 返回 (全名, 显示名称, 路径, 层级)
    fn parse_tag_name(name: &str) -> (String, String, String, i32) {
        let parts: Vec<&str> = name.split('/').collect();
        let level = parts.len() as i32;
        let display_name = parts.last().map_or(name, |v| v).to_string();
        let path = if parts.len() > 1 {
            parts[..parts.len()-1].join("/")
        } else {
            String::new()
        };
        (name.to_string(), display_name, path, level)
    }

    /// 解析内容中的标签并创建关联
    fn parse_and_create_tags(&self, note_id: &str, content: &str) -> SqliteResult<()> {
        // 使用正则表达式匹配标签：#标签名 或 #标签名/子标签
        let re = Regex::new(r"#([a-zA-Z0-9_\u4e00-\u9fa5/]+)").unwrap();
        
        for cap in re.captures_iter(content) {
            if let Some(tag_name) = cap.get(1) {
                let tag_name_str = tag_name.as_str();
                // 创建或获取标签
                if let Ok(tag) = self.create_or_get_tag(tag_name_str, None) {
                    // 创建标签关联
                    self.conn.execute(
                        "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                        params![note_id, &tag.id],
                    )?;
                }
            }
        }
        Ok(())
    }

    /// 创建或获取标签
    pub fn create_or_get_tag(&self, name: &str, color: Option<&str>) -> SqliteResult<Tag> {
        let (full_name, display_name, path, level) = Self::parse_tag_name(name);
        let now = chrono::Utc::now().to_rfc3339();

        // 尝试获取已存在的标签
        if let Some(tag) = self.get_tag_by_name(&full_name)? {
            return Ok(tag);
        }

        // 创建新标签
        let id = Ulid::new().to_string();
        self.conn.execute(
            "INSERT INTO tags (id, name, display_name, path, level, color, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![&id, &full_name, &display_name, &path, level, color, &now, &now],
        )?;

        Ok(Tag {
            id,
            name: full_name,
            display_name,
            path,
            level,
            color: color.map(|c| c.to_string()),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 根据名称获取标签
    pub fn get_tag_by_name(&self, name: &str) -> SqliteResult<Option<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, display_name, path, level, color, created_at, updated_at
             FROM tags WHERE name = ?"
        )?;

        let mut tags = stmt.query_map(params![name], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                path: row.get(3)?,
                level: row.get(4)?,
                color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        match tags.next() {
            Some(Ok(tag)) => Ok(Some(tag)),
            _ => Ok(None),
        }
    }

    /// 获取所有标签
    pub fn get_all_tags(&self) -> SqliteResult<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, display_name, path, level, color, created_at, updated_at
             FROM tags ORDER BY name ASC"
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                path: row.get(3)?,
                level: row.get(4)?,
                color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        tags.collect()
    }

    /// 获取标签统计（带使用次数）
    pub fn get_tags_with_stats(&self) -> SqliteResult<Vec<TagStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.display_name, t.path, t.level, t.color,
                t.created_at, t.updated_at, COUNT(nt.note_id) as count
         FROM tags t
         LEFT JOIN note_tags nt ON t.id = nt.tag_id
         GROUP BY t.id
         ORDER BY t.name ASC"
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(TagStats {
                tag: Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    path: row.get(3)?,
                    level: row.get(4)?,
                    color: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                },
                count: row.get(8)?,
            })
        })?;

        tags.collect()
    }

    /// 获取笔记的所有标签
    pub fn get_note_tags(&self, note_id: &str) -> SqliteResult<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.display_name, t.path, t.level, t.color,
                t.created_at, t.updated_at
         FROM tags t
         INNER JOIN note_tags nt ON t.id = nt.tag_id
         WHERE nt.note_id = ?
         ORDER BY t.name ASC"
        )?;

        let tags = stmt.query_map(params![note_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                path: row.get(3)?,
                level: row.get(4)?,
                color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        tags.collect()
    }

    /// 为笔记添加标签
    pub fn add_tag_to_note(&self, note_id: &str, tag_name: &str) -> SqliteResult<Tag> {
        // 创建或获取标签
        let tag = self.create_or_get_tag(tag_name, None)?;

        // 检查是否已关联
        let exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
            params![note_id, &tag.id],
            |row| row.get(0)
        )?;

        if exists == 0 {
            let id = Ulid::new().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            self.conn.execute(
                "INSERT INTO note_tags (id, note_id, tag_id, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![&id, note_id, &tag.id, &now],
            )?;
        }

        Ok(tag)
    }

    /// 从笔记移除标签
    pub fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> SqliteResult<()> {
        self.conn.execute(
            "DELETE FROM note_tags WHERE note_id = ? AND tag_id = ?",
            params![note_id, tag_id]
        )?;
        Ok(())
    }

    /// 删除标签
    pub fn delete_tag(&self, tag_id: &str) -> SqliteResult<()> {
        self.conn.execute("DELETE FROM tags WHERE id = ?", params![tag_id])?;
        Ok(())
    }

    /// 按标签查询笔记
    pub fn get_notes_by_tag(&self, tag_id: &str, page: u32, page_size: u32) -> SqliteResult<Vec<Note>> {
        let offset = (page - 1) * page_size;
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT n.id, n.type, n.content, n.source_url, n.images, n.created_at, n.updated_at
         FROM notes n
         INNER JOIN note_tags nt ON n.id = nt.note_id
         WHERE nt.tag_id = ?
         ORDER BY n.updated_at DESC
         LIMIT ? OFFSET ?"
        )?;

        let notes = stmt.query_map(params![tag_id, page_size, offset], |row| {
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

    /// 搜索标签（用于自动完成）
    pub fn search_tags(&self, keyword: &str) -> SqliteResult<Vec<Tag>> {
        let search_pattern = format!("{}%", keyword);

        let mut stmt = self.conn.prepare(
            "SELECT id, name, display_name, path, level, color, created_at, updated_at
         FROM tags
         WHERE name LIKE ?1 OR display_name LIKE ?1
         ORDER BY created_at DESC
         LIMIT 5"
        )?;

        let tags = stmt.query_map(params![search_pattern], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                path: row.get(3)?,
                level: row.get(4)?,
                color: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        tags.collect()
    }
}

/// Tauri 命令：获取所有标签
#[tauri::command]
pub async fn get_all_tags(work_directory: Option<String>) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_all_tags().map_err(|e| format!("Failed to get tags: {}", e))
}

/// Tauri 命令：获取标签统计
#[tauri::command]
pub async fn get_tags_with_stats(work_directory: Option<String>) -> Result<Vec<TagStats>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_tags_with_stats().map_err(|e| format!("Failed to get tags: {}", e))
}

/// Tauri 命令：获取笔记的标签
#[tauri::command]
pub async fn get_note_tags(note_id: String, work_directory: Option<String>) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_note_tags(&note_id).map_err(|e| format!("Failed to get note tags: {}", e))
}

/// Tauri 命令：为笔记添加标签
#[tauri::command]
pub async fn add_tag_to_note(note_id: String, tag_name: String, work_directory: Option<String>) -> Result<Tag, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.add_tag_to_note(&note_id, &tag_name).map_err(|e| format!("Failed to add tag: {}", e))
}

/// Tauri 命令：从笔记移除标签
#[tauri::command]
pub async fn remove_tag_from_note(note_id: String, tag_id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.remove_tag_from_note(&note_id, &tag_id).map_err(|e| format!("Failed to remove tag: {}", e))
}

/// Tauri 命令：删除标签
#[tauri::command]
pub async fn delete_tag(tag_id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.delete_tag(&tag_id).map_err(|e| format!("Failed to delete tag: {}", e))
}

/// Tauri 命令：按标签获取笔记
#[tauri::command]
pub async fn get_notes_by_tag(tag_id: String, page: u32, page_size: u32, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_notes_by_tag(&tag_id, page, page_size).map_err(|e| format!("Failed to get notes: {}", e))
}

/// Tauri 命令：搜索标签
#[tauri::command]
pub async fn search_tags(keyword: String, work_directory: Option<String>) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.search_tags(&keyword).map_err(|e| format!("Failed to search tags: {}", e))
}