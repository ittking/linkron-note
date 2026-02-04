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
    #[serde(rename = "extractUrl")]
    pub extract_url: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// 序列化图片数组为 JSON 字符串
fn serialize_images(images: &Vec<String>) -> String {
    serde_json::to_string(images).unwrap_or_else(|_| "[]".to_string())
}

/// 反序列化 JSON 字符串为图片数组
fn deserialize_images(images_str: &str) -> Vec<String> {
    serde_json::from_str(images_str).unwrap_or_default()
}

// 创建笔记的数据结构
#[derive(Debug, Deserialize)]
pub struct NoteData {
    #[serde(rename = "type")]
    pub note_type: Option<String>,
    pub content: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    #[serde(rename = "extractUrl")]
    pub extract_url: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
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
                extract_url TEXT,
                images TEXT DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 检查并添加 images 列（兼容旧版本数据库）
        self.conn.execute(
            "ALTER TABLE notes ADD COLUMN images TEXT DEFAULT '[]'",
            [],
        ).ok(); // 忽略列已存在的错误

        // 检查并添加 extract_url 列（兼容旧版本数据库）
        self.conn.execute(
            "ALTER TABLE notes ADD COLUMN extract_url TEXT",
            [],
        ).ok(); // 忽略列已存在的错误

        // 创建 tags 表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                path TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
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
            "SELECT id, type, content, source_url,
             COALESCE(images, '[]') as images,
             created_at, updated_at,
             extract_url
             FROM notes ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )?;

        let notes = stmt.query_map(params![page_size, offset], |row| {
            let images_str: String = row.get(4)?;
            Ok(Note {
                id: row.get(0)?,
                note_type: row.get(1)?,
                content: row.get(2)?,
                source_url: row.get(3)?,
                extract_url: row.get(7)?,
                images: deserialize_images(&images_str),
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
            "SELECT id, type, content, source_url,
             COALESCE(images, '[]') as images,
             created_at, updated_at,
             extract_url
             FROM notes WHERE id = ?"
        )?;

        let mut notes = stmt.query_map(params![id], |row| {
            let images_str: String = row.get(4)?;
            Ok(Note {
                id: row.get(0)?,
                note_type: row.get(1)?,
                content: row.get(2)?,
                source_url: row.get(3)?,
                extract_url: row.get(7)?,
                images: deserialize_images(&images_str),
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
        let images_json = serialize_images(&note_data.images);

        self.conn.execute(
            "INSERT INTO notes (id, type, content, source_url, images, created_at, updated_at, extract_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &id,
                &note_type,
                &note_data.content,
                &note_data.source_url,
                &images_json,
                &now,
                &now,
                &note_data.extract_url
            ],
        )?;

        self.parse_and_create_tags(&id, &note_data.content)?;

        Ok(Note {
            id,
            note_type,
            content: note_data.content,
            source_url: note_data.source_url,
            extract_url: note_data.extract_url,
            images: note_data.images,
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

        // 根据提供的字段构建更新语句
        if let Some(content) = &updates.content {
            if let Some(images) = &updates.images {
                // 同时更新 content 和 images
                let images_json = serialize_images(images);
                self.conn.execute(
                    "UPDATE notes SET content = ?1, images = ?2, updated_at = ?3 WHERE id = ?4",
                    params![content, &images_json, &now, id],
                )?;
            } else {
                // 只更新 content
                self.conn.execute(
                    "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
                    params![content, &now, id],
                )?;
            }
            
            // 删除旧的标签关联
            self.conn.execute(
                "DELETE FROM note_tags WHERE note_id = ?1",
                params![id],
            )?;
            
            // 解析并创建新的标签关联
            self.parse_and_create_tags(id, content)?;
        } else if let Some(images) = &updates.images {
            // 只更新 images
            let images_json = serialize_images(images);
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

    /// 删除笔记关联的资源文件
    /// 包括 images 数组中的图片、content 中的图片引用，以及附件笔记的 extractUrl
    fn delete_note_resources(note: &Note, work_directory: &Option<String>) {
        // 1. 删除 images 数组中的图片
        for image_url in &note.images {
            let _ = super::filesystem::delete_resource_by_url(
                image_url.clone(),
                work_directory.clone()
            );
        }

        // 2. 删除 content 中的本地图片引用
        let img_regex = Regex::new(r#"<img[^>]+src="([^"]+)""#).unwrap();
        for caps in img_regex.captures_iter(&note.content) {
            if let Some(image_url) = caps.get(1) {
                let _ = super::filesystem::delete_resource_by_url(
                    image_url.as_str().to_string(),
                    work_directory.clone()
                );
            }
        }

        // 3. 如果是附件笔记，删除 extractUrl 指向的文件
        if note.note_type == "file" {
            if let Some(extract_url) = &note.extract_url {
                let _ = super::filesystem::delete_resource_by_url(
                    extract_url.clone(),
                    work_directory.clone()
                );
            }
        }
    }

    /// 搜索笔记
    pub fn search_notes(&self, keyword: &str) -> SqliteResult<Vec<Note>> {
        let search_pattern = format!("%{}%", keyword);

        let mut stmt = self.conn.prepare(
            "SELECT id, type, content, source_url, extract_url,
             COALESCE(images, '[]') as images,
             created_at, updated_at
             FROM notes WHERE content LIKE ?1
             ORDER BY updated_at DESC"
        )?;

        let notes = stmt.query_map(params![search_pattern], |row| {
            let images_str: String = row.get(5)?;
            Ok(Note {
                id: row.get(0)?,
                note_type: row.get(1)?,
                content: row.get(2)?,
                source_url: row.get(3)?,
                extract_url: row.get(4)?,
                images: deserialize_images(&images_str),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
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
    let db_path = get_database_path(work_directory.clone())?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    // 先获取笔记信息，用于清理资源
    let note = db.get_note(&id)
        .map_err(|e| format!("Failed to get note: {}", e))?
        .ok_or_else(|| format!("Note not found: {}", id))?;

    // 删除笔记关联的资源文件
    Database::delete_note_resources(&note, &work_directory);

    // 删除笔记记录
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
            extract_url: note_json.get("extractUrl").and_then(|v| v.as_str()).map(|s| s.to_string()),
            images: vec![], // 迁移的笔记默认为空图片数组
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
    /// 解析内容中的标签并创建关联（直接匹配文本中的 #标签名 格式）
    fn parse_and_create_tags(&self, note_id: &str, content: &str) -> SqliteResult<()> {
        // 直接在内容中查找标签：#标签名 或 #标签名/子标签
        let tag_re = Regex::new(r"#([a-zA-Z0-9_\u4e00-\u9fa5/]+)").unwrap();

        for tag_cap in tag_re.captures_iter(content) {
            if let Some(tag_name) = tag_cap.get(1) {
                let tag_name_str = tag_name.as_str();

                // 解析多级标签
                let parts: Vec<&str> = tag_name_str.split('/').collect();

                // 创建或获取所有层级的标签（从父级到子级）
                let mut current_path = String::new();
                for (i, part) in parts.iter().enumerate() {
                    let full_name = if i == 0 {
                        part.to_string()
                    } else {
                        format!("{}/{}", current_path, part)
                    };

                    // 创建或获取标签
                    if let Ok(_tag) = self.create_or_get_tag(&full_name) {
                        // 只将最末级的标签与笔记关联
                        if i == parts.len() - 1 {
                            self.conn.execute(
                                "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                                params![note_id, &_tag.id],
                            )?;
                        }
                    }

                    // 更新当前路径
                    current_path = full_name;
                }
            }
        }
        Ok(())
    }

    /// 创建或获取标签（支持多级路径）
    pub fn create_or_get_tag(&self, name: &str) -> SqliteResult<Tag> {
        // 检查是否已存在
        if let Some(tag) = self.get_tag_by_name(name)? {
            return Ok(tag);
        }

        // 解析标签路径
        let parts: Vec<&str> = name.split('/').collect();
        let level = parts.len() as i32;
        let display_name = parts.last().unwrap_or(&name).to_string();

        // path 是父级标签的完整路径（不包括当前标签名）
        let path = if parts.len() > 1 {
            parts[..parts.len()-1].join("/")
        } else {
            String::new()
        };

        // 创建新标签
        let id = Ulid::new().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO tags (id, name, display_name, path, level, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![&id, &name, &display_name, &path, level, &now, &now],
        )?;

        Ok(Tag {
            id,
            name: name.to_string(),
            display_name,
            path,
            level,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 根据名称获取标签
    pub fn get_tag_by_name(&self, name: &str) -> SqliteResult<Option<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, display_name, path, level, created_at, updated_at
             FROM tags WHERE name = ?"
        )?;

        let mut tags = stmt.query_map(params![name], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                path: row.get(3)?,
                level: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
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
            "SELECT id, name, display_name, path, level, created_at, updated_at
             FROM tags ORDER BY name ASC"
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                path: row.get(3)?,
                level: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        tags.collect()
    }

    /// 获取标签统计（带使用次数）
    pub fn get_tags_with_stats(&self) -> SqliteResult<Vec<TagStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.display_name, t.path, t.level,
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
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                },
                count: row.get(7)?,
            })
        })?;

        tags.collect()
    }

    /// 获取笔记的所有标签
    pub fn get_note_tags(&self, note_id: &str) -> SqliteResult<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.display_name, t.path, t.level,
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
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        tags.collect()
    }

    /// 为笔记添加标签
    pub fn add_tag_to_note(&self, note_id: &str, tag_name: &str) -> SqliteResult<Tag> {
        // 创建或获取标签
        let tag = self.create_or_get_tag(tag_name)?;

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
            "SELECT DISTINCT n.id, n.type, n.content, n.source_url, n.extract_url,
             COALESCE(n.images, '[]') as images,
             n.created_at, n.updated_at
         FROM notes n
         INNER JOIN note_tags nt ON n.id = nt.note_id
         WHERE nt.tag_id = ?
         ORDER BY n.updated_at DESC
         LIMIT ? OFFSET ?"
        )?;

        let notes = stmt.query_map(params![tag_id, page_size, offset], |row| {
            let images_str: String = row.get(5)?;
            Ok(Note {
                id: row.get(0)?,
                note_type: row.get(1)?,
                content: row.get(2)?,
                source_url: row.get(3)?,
                extract_url: row.get(4)?,
                images: deserialize_images(&images_str),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        notes.collect()
    }

    /// 搜索标签（用于自动完成）
    pub fn search_tags(&self, keyword: &str) -> SqliteResult<Vec<Tag>> {
        let search_pattern = format!("{}%", keyword);

        let mut stmt = self.conn.prepare(
            "SELECT id, name, display_name, path, level, created_at, updated_at
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
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        tags.collect()
    }

    /// 按路径模式获取标签（用于重命名/删除操作）
    fn get_tags_by_path_pattern(&self, pattern: &str) -> SqliteResult<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, display_name, path, level, created_at, updated_at
         FROM tags WHERE name LIKE ? ESCAPE '\' OR path LIKE ? ESCAPE '\'"
        )?;

        let tags = stmt.query_map(params![pattern, pattern], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                path: row.get(3)?,
                level: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        tags.collect()
    }

    /// 重命名标签
    pub fn rename_tag(&self, old_name: &str, new_name: &str, rename_children: bool) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        // 解析新旧路径
        let new_parts: Vec<&str> = new_name.split('/').collect();
        let new_display_name = new_parts.last().unwrap_or(&new_name);

        // 更新当前标签
        self.conn.execute(
            "UPDATE tags SET name = ?1, display_name = ?2, updated_at = ?3 WHERE name = ?4",
            params![new_name, new_display_name, &now, old_name]
        )?;

        // 如果需要重命名子标签
        if rename_children {
            let pattern = format!("{}%", old_name.replace("%", "\\%").replace("_", "\\_"));

            for tag in self.get_tags_by_path_pattern(&pattern)? {
                // 跳过自己（已经更新过）
                if tag.name == old_name {
                    continue;
                }

                let new_child_name = tag.name.replacen(old_name, new_name, 1);
                let new_child_path = if tag.path.is_empty() {
                    new_name.to_string()
                } else {
                    tag.path.replacen(old_name, new_name, 1)
                };

                self.conn.execute(
                    "UPDATE tags SET name = ?1, path = ?2, updated_at = ?3 WHERE id = ?4",
                    params![&new_child_name, &new_child_path, &now, &tag.id]
                )?;
            }
        }

        Ok(())
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