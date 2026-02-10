use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ulid::Ulid;
use regex::Regex;
use chrono::{Datelike, TimeZone};

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

// 标签数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub pinned: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

// 热度图数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonthData {
    pub year: i32,
    pub month: i32,
    pub weeks: [i32; 5], // 每月5周的笔记数量
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
                parent_id TEXT,
                name TEXT NOT NULL,
                full_name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // 检查并添加 tags 表可能缺失的列（兼容旧版本数据库）
        self.conn.execute(
            "ALTER TABLE tags ADD COLUMN parent_id TEXT",
            [],
        ).ok(); // 忽略列已存在的错误

        self.conn.execute(
            "ALTER TABLE tags ADD COLUMN name TEXT",
            [],
        ).ok(); // 忽略列已存在的错误

        self.conn.execute(
            "ALTER TABLE tags ADD COLUMN full_name TEXT",
            [],
        ).ok(); // 忽略列已存在的错误

        // 检查并添加 display_name 列（兼容旧版本数据库）
        self.conn.execute(
            "ALTER TABLE tags ADD COLUMN display_name TEXT",
            [],
        ).ok(); // 忽略列已存在的错误

        self.conn.execute(
            "ALTER TABLE tags ADD COLUMN pinned INTEGER DEFAULT 0",
            [],
        ).ok(); // 忽略列已存在的错误

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
            "CREATE INDEX IF NOT EXISTS idx_tags_parent_id ON tags(parent_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_full_name ON tags(full_name)",
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

        // 解析并同步标签
        let tags = self.parse_tags_from_content(&note_data.content);
        if !tags.is_empty() {
            self.create_or_update_tags(tags)?;
        }

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

        if let Some(content) = &updates.content {
            if let Some(images) = &updates.images {
                let images_json = serialize_images(images);
                self.conn.execute(
                    "UPDATE notes SET content = ?1, images = ?2, updated_at = ?3 WHERE id = ?4",
                    params![content, &images_json, &now, id],
                )?;

                // 解析并同步标签
                let tags = self.parse_tags_from_content(content);
                if !tags.is_empty() {
                    self.create_or_update_tags(tags)?;
                }
            } else {
                self.conn.execute(
                    "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
                    params![content, &now, id],
                )?;

                // 解析并同步标签
                let tags = self.parse_tags_from_content(content);
                if !tags.is_empty() {
                    self.create_or_update_tags(tags)?;
                }
            }
        } else if let Some(images) = &updates.images {
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

// ============ 标签相关方法 ============

impl Database {
    /// 解析笔记内容中的标签
    pub fn parse_tags_from_content(&self, content: &str) -> Vec<String> {
        let mut tags = std::collections::HashSet::new();

        // 只匹配 HTML 格式的标签 <span class="tag">#标签名</span>
        let html_tag_regex = Regex::new(r#"<span class="tag">#([^<]+)</span>"#).unwrap();

        for caps in html_tag_regex.captures_iter(content) {
            if let Some(tag_name) = caps.get(1) {
                tags.insert(tag_name.as_str().to_string());
            }
        }

        tags.into_iter().collect()
    }

    /// 创建或更新标签（处理多级标签结构）
    pub fn create_or_update_tags(&self, tags: Vec<String>) -> SqliteResult<Vec<Tag>> {
        let mut created_tags = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();

        for tag_full_name in tags {
            // 分割标签路径，例如 "测试/子标签" -> ["测试", "子标签"]
            let parts: Vec<&str> = tag_full_name.split('/').collect();
            let mut parent_id: Option<String> = None;
            let mut current_full_name = String::new();

            for (index, part) in parts.iter().enumerate() {
                // 构建当前级别的完整名称
                if index == 0 {
                    current_full_name = part.to_string();
                } else {
                    current_full_name = format!("{}/{}", current_full_name, part);
                }

                // 检查标签是否已存在
                let existing_tag = self.get_tag_by_full_name(&current_full_name)?;

                let tag = if let Some(existing) = existing_tag {
                    // 更新标签
                    let updated_tag = Tag {
                        id: existing.id.clone(),
                        parent_id: parent_id.clone(),
                        name: part.to_string(),
                        full_name: current_full_name.clone(),
                        pinned: existing.pinned,
                        created_at: existing.created_at,
                        updated_at: now.clone(),
                    };

                    self.conn.execute(
                        "UPDATE tags SET parent_id = ?1, name = ?2, updated_at = ?3 WHERE id = ?4",
                        params![&parent_id, part, &now, &existing.id],
                    )?;

                    updated_tag
                } else {
                    // 创建新标签
                    let id = Ulid::new().to_string();
                    let new_tag = Tag {
                        id: id.clone(),
                        parent_id: parent_id.clone(),
                        name: part.to_string(),
                        full_name: current_full_name.clone(),
                        pinned: false,
                        created_at: now.clone(),
                        updated_at: now.clone(),
                    };

                    self.conn.execute(
                        "INSERT INTO tags (id, parent_id, name, full_name, display_name, pinned, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![&id, &parent_id, part, &current_full_name, part, 0, &now, &now],
                    )?;

                    new_tag
                };

                // 更新 parent_id 为当前标签的 ID，用于下一级
                parent_id = Some(tag.id.clone());

                // 只将最终级别的标签添加到结果中
                if index == parts.len() - 1 {
                    created_tags.push(tag);
                }
            }
        }

        Ok(created_tags)
    }

    /// 根据完整名称获取标签
    pub fn get_tag_by_full_name(&self, full_name: &str) -> SqliteResult<Option<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, name, COALESCE(full_name, name) as full_name, COALESCE(pinned, 0) as pinned, created_at, updated_at
             FROM tags WHERE COALESCE(full_name, name) = ?1"
        )?;

        let mut tags = stmt.query_map(params![full_name], |row| {
            let pinned: i32 = row.get(4)?;
            Ok(Tag {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                full_name: row.get(3)?,
                pinned: pinned == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        match tags.next() {
            Some(Ok(tag)) => Ok(Some(tag)),
            _ => Ok(None),
        }
    }

    /// 获取所有标签（树状结构）
    pub fn get_all_tags(&self) -> SqliteResult<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, name, COALESCE(full_name, name) as full_name, COALESCE(pinned, 0) as pinned, created_at, updated_at
             FROM tags ORDER BY COALESCE(pinned, 0) DESC, COALESCE(full_name, name) ASC"
        )?;

        let tags = stmt.query_map([], |row| {
            let pinned: i32 = row.get(4)?;
            Ok(Tag {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                full_name: row.get(3)?,
                pinned: pinned == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        tags.collect()
    }

    /// 删除标签（级联删除子标签）
    pub fn delete_tag(&self, id: &str) -> SqliteResult<()> {
        // 递归删除所有子标签
        self.delete_tag_recursive(id)?;
        Ok(())
    }

    /// 递归删除标签及其所有子标签
    fn delete_tag_recursive(&self, id: &str) -> SqliteResult<()> {
        // 先查找所有子标签
        let mut stmt = self.conn.prepare("SELECT id FROM tags WHERE parent_id = ?1")?;
        let child_ids: Vec<String> = stmt.query_map(params![id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        // 递归删除每个子标签
        for child_id in child_ids {
            self.delete_tag_recursive(&child_id)?;
        }

        // 删除当前标签
        self.conn.execute("DELETE FROM tags WHERE id = ?", params![id])?;
        Ok(())
    }

    /// 置顶/取消置顶标签
    pub fn pin_tag(&self, id: &str) -> SqliteResult<()> {
        // 获取当前置顶状态
        let mut stmt = self.conn.prepare("SELECT COALESCE(pinned, 0) FROM tags WHERE id = ?1")?;
        let mut tags = stmt.query_map(params![id], |row| {
            let pinned: i32 = row.get(0)?;
            Ok(pinned == 1)
        })?;

        let current_pinned = match tags.next() {
            Some(Ok(pinned)) => pinned,
            _ => return Ok(()),
        };

        // 切换置顶状态
        let new_pinned = if current_pinned { 0 } else { 1 };
        self.conn.execute(
            "UPDATE tags SET pinned = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_pinned, &chrono::Utc::now().to_rfc3339(), id],
        )?;

        Ok(())
    }

    /// 根据标签筛选笔记（支持多个标签，OR 逻辑，支持子标签查询）
    pub fn get_notes_by_tags(&self, tags: Vec<String>) -> SqliteResult<Vec<Note>> {
        println!("[标签筛选] 开始筛选笔记，输入标签: {:?}", tags);

        if tags.is_empty() {
            return self.get_all_notes(1, 1000);
        }

        let mut where_clauses = Vec::new();

        for tag_full_name in &tags {
            // 支持子标签查询：匹配 #测试 或 #测试/开头的标签
            // 例如：查询"测试"会匹配"#测试"、"#测试/子标签"、"#测试/子标签/xxx"等
            let tag_pattern_exact = format!("%<span class=\"tag\">#{}</span>%", tag_full_name);
            let tag_pattern_with_slash = format!("%<span class=\"tag\">#{}/%</span>%", tag_full_name);
            
            // 构建正确的 SQL：content LIKE ? OR content LIKE ?
            let combined_pattern = format!("content LIKE ? OR content LIKE ?");
            where_clauses.push(combined_pattern);
        }

        let where_clause = where_clauses.join(" OR ");
        let sql = format!(
            "SELECT id, type, content, source_url,
             COALESCE(images, '[]') as images,
             created_at, updated_at,
             extract_url
             FROM notes WHERE {} ORDER BY updated_at DESC",
            where_clause
        );

        println!("[标签筛选] SQL 查询: {}", sql);

        let mut stmt = self.conn.prepare(&sql)?;
        
        // 构建参数：每个标签需要两个参数（精确匹配和子标签匹配）
        let mut all_params: Vec<String> = Vec::new();
        for tag_full_name in &tags {
            all_params.push(format!("%<span class=\"tag\">#{}</span>%", tag_full_name));
            all_params.push(format!("%<span class=\"tag\">#{}/%</span>%", tag_full_name));
        }
        let params_refs: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let notes = stmt.query_map(params_refs.as_slice(), |row| {
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

        let result: Vec<Note> = notes.collect::<Result<Vec<_>, _>>()?;
        println!("[标签筛选] 找到 {} 条笔记", result.len());
        Ok(result)
    }

    /// 根据标签获取笔记数量（支持子标签查询）
    pub fn count_notes_by_tags(&self, tags: Vec<String>) -> SqliteResult<i64> {
        println!("[标签筛选] 开始计数，输入标签: {:?}", tags);

        if tags.is_empty() {
            return self.count_notes();
        }

        let mut where_clauses = Vec::new();

        for tag_full_name in &tags {
            // 支持子标签查询
            let tag_pattern_exact = format!("%<span class=\"tag\">#{}</span>%", tag_full_name);
            let tag_pattern_with_slash = format!("%<span class=\"tag\">#{}/%</span>%", tag_full_name);
            let combined_pattern = format!("content LIKE ? OR content LIKE ?");
            where_clauses.push(combined_pattern);
        }

        let where_clause = where_clauses.join(" OR ");
        let sql = format!("SELECT COUNT(*) FROM notes WHERE {}", where_clause);

        println!("[标签筛选] 计数 SQL: {}", sql);

        let mut stmt = self.conn.prepare(&sql)?;
        
        // 构建参数
        let mut all_params: Vec<String> = Vec::new();
        for tag_full_name in &tags {
            all_params.push(format!("%<span class=\"tag\">#{}</span>%", tag_full_name));
            all_params.push(format!("%<span class=\"tag\">#{}/%</span>%", tag_full_name));
        }
        let params_refs: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let count = stmt.query_row(params_refs.as_slice(), |row| row.get(0))?;
        println!("[标签筛选] 笔记数量: {}", count);
        Ok(count)
    }

    /// 获取笔记热度图数据（最近12个月，每月5周）
    pub fn get_notes_heatmap(&self) -> SqliteResult<Vec<MonthData>> {
        let now = chrono::Utc::now();
        let mut result = Vec::new();

        println!("[热度图] 开始统计热度图数据，当前时间: {}", now);

        // 从当前月份往前推12个月
        for i in 0..12 {
            let current_date = now - chrono::Duration::days((i * 30) as i64);
            let year = current_date.year();
            let month = current_date.month();

            // 获取该月的第一天和最后一天
            let first_day = chrono::Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();
            let next_month_first_day = if month == 12 {
                chrono::Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
            } else {
                chrono::Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
            };
            let last_day = next_month_first_day - chrono::Duration::days(1);

            println!("[热度图] 月份: {}-{}, 时间范围: {} 到 {}", year, month, first_day.to_rfc3339(), last_day.to_rfc3339());

            // 获取该月的所有笔记
            let sql = "SELECT created_at FROM notes WHERE created_at >= ? AND created_at <= ? ORDER BY created_at";
            let mut stmt = self.conn.prepare(sql)?;

            let notes = stmt.query_map(params![first_day.to_rfc3339(), last_day.to_rfc3339()], |row| {
                row.get::<_, String>(0)
            })?;

            // 统计每周的笔记数量
            let mut weeks: [i32; 5] = [0; 5];
            let mut total_count = 0;
            for note in notes {
                if let Ok(date_str) = note {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&date_str) {
                        let day_of_month = dt.day();
                        let week_index = ((day_of_month - 1) / 7).min(4) as usize;
                        weeks[week_index] += 1;
                        total_count += 1;
                    }
                }
            }

            println!("[热度图] {}-{} 总共找到 {} 条笔记，每周分布: {:?}", year, month, total_count, weeks);

            result.push(MonthData {
                year,
                month: month as i32,
                weeks,
            });
        }

        // 反转顺序，让最近的月份在前面
        result.reverse();
        println!("[热度图] 最终结果: {:?}", result.iter().map(|m| format!("{}-{}: {:?}", m.year, m.month, m.weeks)).collect::<Vec<_>>());
        Ok(result)
    }
}

// ============ 标签相关的 Tauri 命令 ============

/// Tauri 命令：从笔记内容解析标签
#[tauri::command]
pub async fn parse_tags(content: String, work_directory: Option<String>) -> Result<Vec<String>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    Ok(db.parse_tags_from_content(&content))
}

/// Tauri 命令：创建或更新标签
#[tauri::command]
pub async fn sync_tags(tags: Vec<String>, work_directory: Option<String>) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.create_or_update_tags(tags).map_err(|e| format!("Failed to sync tags: {}", e))
}

/// Tauri 命令：获取所有标签
#[tauri::command]
pub async fn get_all_tags(work_directory: Option<String>) -> Result<Vec<Tag>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_all_tags().map_err(|e| format!("Failed to get tags: {}", e))
}

/// Tauri 命令：删除标签
#[tauri::command]
pub async fn delete_tag(id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.delete_tag(&id).map_err(|e| format!("Failed to delete tag: {}", e))?;
    Ok(())
}

/// Tauri 命令：置顶/取消置顶标签
#[tauri::command]
pub async fn pin_tag(id: String, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.pin_tag(&id).map_err(|e| format!("Failed to pin tag: {}", e))?;
    Ok(())
}

/// Tauri 命令：根据标签筛选笔记
#[tauri::command]
pub async fn get_notes_by_tags(tags: Vec<String>, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_notes_by_tags(tags).map_err(|e| format!("Failed to get notes by tags: {}", e))
}

/// Tauri 命令：根据标签获取笔记数量
#[tauri::command]
pub async fn count_notes_by_tags(tags: Vec<String>, work_directory: Option<String>) -> Result<i64, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.count_notes_by_tags(tags).map_err(|e| format!("Failed to count notes by tags: {}", e))
}

/// Tauri 命令：获取笔记热度图数据
#[tauri::command]
pub async fn get_notes_heatmap(work_directory: Option<String>) -> Result<Vec<MonthData>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_notes_heatmap().map_err(|e| format!("Failed to get notes heatmap: {}", e))
}
