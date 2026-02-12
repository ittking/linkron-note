use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use chrono::Datelike;

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
    #[serde(default)]
    pub pinned: bool,
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
    #[serde(rename = "extractUrl")]
    pub extract_url: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
}

// 更新笔记的数据结构
#[derive(Debug, Deserialize, Clone)]
pub struct NoteUpdate {
    pub content: Option<String>,
    pub images: Option<Vec<String>>,
}

/// 序列化图片数组为 JSON 字符串
fn serialize_images(images: &Vec<String>) -> String {
    serde_json::to_string(images).unwrap_or_else(|_| "[]".to_string())
}

/// 反序列化 JSON 字符串为图片数组
fn deserialize_images(images_str: &str) -> Vec<String> {
    serde_json::from_str(images_str).unwrap_or_default()
}

/// 初始化笔记表
pub fn init_tables(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
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

    // 检查并添加 images 列
    conn.execute("ALTER TABLE notes ADD COLUMN images TEXT DEFAULT '[]'", []).ok();
    conn.execute("ALTER TABLE notes ADD COLUMN extract_url TEXT", []).ok();
    conn.execute("ALTER TABLE notes ADD COLUMN pinned INTEGER DEFAULT 0", []).ok();

    // 创建索引
    conn.execute("CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at DESC)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_notes_type ON notes(type)", [])?;

    Ok(())
}

/// 获取所有笔记（分页）
pub fn get_all_notes(conn: &Connection, page: u32, page_size: u32) -> SqliteResult<Vec<Note>> {
    let offset = (page - 1) * page_size;
    let mut stmt = conn.prepare(
        "SELECT id, type, content, source_url,
         COALESCE(images, '[]') as images,
         COALESCE(pinned, 0) as pinned,
         created_at, updated_at,
         extract_url
         FROM notes ORDER BY pinned DESC, created_at DESC LIMIT ? OFFSET ?"
    )?;

    let notes = stmt.query_map(params![page_size, offset], |row| {
        let images_str: String = row.get(4)?;
        let pinned: i32 = row.get(5)?;
        Ok(Note {
            id: row.get(0)?,
            note_type: row.get(1)?,
            content: row.get(2)?,
            source_url: row.get(3)?,
            extract_url: row.get(8)?,
            images: deserialize_images(&images_str),
            pinned: pinned == 1,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;

    notes.collect()
}

/// 获取笔记总数
pub fn count_notes(conn: &Connection) -> SqliteResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
}

/// 获取单个笔记
pub fn get_note(conn: &Connection, id: &str) -> SqliteResult<Option<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, content, source_url,
         COALESCE(images, '[]') as images,
         COALESCE(pinned, 0) as pinned,
         created_at, updated_at,
         extract_url
         FROM notes WHERE id = ?"
    )?;

    let mut notes = stmt.query_map(params![id], |row| {
        let images_str: String = row.get(4)?;
        let pinned: i32 = row.get(5)?;
        Ok(Note {
            id: row.get(0)?,
            note_type: row.get(1)?,
            content: row.get(2)?,
            source_url: row.get(3)?,
            extract_url: row.get(8)?,
            images: deserialize_images(&images_str),
            pinned: pinned == 1,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;

    match notes.next() {
        Some(Ok(note)) => Ok(Some(note)),
        _ => Ok(None),
    }
}

/// 创建笔记
pub fn create_note(conn: &Connection, note_data: NoteData) -> SqliteResult<Note> {
    let id = Ulid::new().to_string();
    let note_type = note_data.note_type.unwrap_or_else(|| "text".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let images_json = serialize_images(&note_data.images);

    conn.execute(
        "INSERT INTO notes (id, type, content, source_url, images, pinned, created_at, updated_at, extract_url)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            &id,
            &note_type,
            &note_data.content,
            &note_data.source_url,
            &images_json,
            0,
            &now,
            &now,
            &note_data.extract_url
        ],
    )?;

    Ok(Note {
        id,
        note_type,
        content: note_data.content,
        source_url: note_data.source_url,
        extract_url: note_data.extract_url,
        images: note_data.images,
        pinned: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 更新笔记
pub fn update_note(conn: &Connection, id: &str, updates: NoteUpdate) -> SqliteResult<()> {
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(content) = &updates.content {
        if let Some(images) = &updates.images {
            let images_json = serialize_images(images);
            conn.execute(
                "UPDATE notes SET content = ?1, images = ?2, updated_at = ?3 WHERE id = ?4",
                params![content, &images_json, &now, id],
            )?;
        } else {
            conn.execute(
                "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content, &now, id],
            )?;
        }
    } else if let Some(images) = &updates.images {
        let images_json = serialize_images(images);
        conn.execute(
            "UPDATE notes SET images = ?1, updated_at = ?2 WHERE id = ?3",
            params![&images_json, &now, id],
        )?;
    }

    Ok(())
}

/// 删除笔记
pub fn delete_note(conn: &Connection, id: &str) -> SqliteResult<()> {
    conn.execute("DELETE FROM notes WHERE id = ?", params![id])?;
    Ok(())
}

/// 置顶/取消置顶笔记
pub fn pin_note(conn: &Connection, id: &str) -> SqliteResult<()> {
    let mut stmt = conn.prepare("SELECT COALESCE(pinned, 0) FROM notes WHERE id = ?1")?;
    let mut notes = stmt.query_map(params![id], |row| {
        let pinned: i32 = row.get(0)?;
        Ok(pinned == 1)
    })?;

    let current_pinned = match notes.next() {
        Some(Ok(pinned)) => pinned,
        _ => return Ok(()),
    };

    let new_pinned = if current_pinned { 0 } else { 1 };
    conn.execute(
        "UPDATE notes SET pinned = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_pinned, &chrono::Utc::now().to_rfc3339(), id],
    )?;

    Ok(())
}

/// 搜索笔记
pub fn search_notes(conn: &Connection, keyword: &str) -> SqliteResult<Vec<Note>> {
    let search_pattern = format!("%{}%", keyword);

    let mut stmt = conn.prepare(
        "SELECT id, type, content, source_url, extract_url,
         COALESCE(images, '[]') as images,
         COALESCE(pinned, 0) as pinned,
         created_at, updated_at
         FROM notes WHERE content LIKE ?1
         ORDER BY pinned DESC, created_at DESC"
    )?;

    let notes = stmt.query_map(params![search_pattern], |row| {
        let images_str: String = row.get(5)?;
        let pinned: i32 = row.get(6)?;
        Ok(Note {
            id: row.get(0)?,
            note_type: row.get(1)?,
            content: row.get(2)?,
            source_url: row.get(3)?,
            extract_url: row.get(4)?,
            images: deserialize_images(&images_str),
            pinned: pinned == 1,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;

    notes.collect()
}

/// 根据标签筛选笔记
pub fn get_notes_by_tags(conn: &Connection, tags: Vec<String>) -> SqliteResult<Vec<Note>> {
    if tags.is_empty() {
        return get_all_notes(conn, 1, 1000);
    }

    let mut where_clauses = Vec::new();
    let mut all_params: Vec<String> = Vec::new();

    for tag_full_name in &tags {
        where_clauses.push(format!("content LIKE ? OR content LIKE ?"));
        all_params.push(format!("%<span class=\"tag\">#{}</span>%", tag_full_name));
        all_params.push(format!("%<span class=\"tag\">#{}/%</span>%", tag_full_name));
    }

    let where_clause = where_clauses.join(" OR ");
    let sql = format!(
        "SELECT id, type, content, source_url,
         COALESCE(images, '[]') as images,
         COALESCE(pinned, 0) as pinned,
         created_at, updated_at,
         extract_url
         FROM notes WHERE {} ORDER BY pinned DESC, created_at DESC",
        where_clause
    );

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

    let notes = stmt.query_map(&params_refs[..], |row| {
        let images_str: String = row.get(4)?;
        let pinned: i32 = row.get(5)?;
        Ok(Note {
            id: row.get(0)?,
            note_type: row.get(1)?,
            content: row.get(2)?,
            source_url: row.get(3)?,
            extract_url: row.get(8)?,
            images: deserialize_images(&images_str),
            pinned: pinned == 1,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;

    notes.collect()
}

/// 根据标签获取笔记数量
pub fn count_notes_by_tags(conn: &Connection, tags: Vec<String>) -> SqliteResult<i64> {
    if tags.is_empty() {
        return count_notes(conn);
    }

    let mut where_clauses = Vec::new();
    let mut all_params: Vec<String> = Vec::new();

    for tag_full_name in &tags {
        where_clauses.push(format!("content LIKE ? OR content LIKE ?"));
        all_params.push(format!("%<span class=\"tag\">#{}%</span>%", tag_full_name));
        all_params.push(format!("%<span class=\"tag\">#{}/%</span>%", tag_full_name));
    }

    let where_clause = where_clauses.join(" OR ");
    let sql = format!("SELECT COUNT(*) FROM notes WHERE {}", where_clause);

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

    stmt.query_row(&params_refs[..], |row| row.get(0))
}

/// 获取笔记热度图数据
pub fn get_notes_heatmap(conn: &Connection) -> SqliteResult<Vec<crate::database::MonthData>> {
    let now = chrono::Utc::now();
    let mut result = Vec::new();

    // 计算最近12个月
    for i in 0..12 {
        let date = now - chrono::Duration::days(30 * (11 - i) as i64);
        let year = date.year();
        let month = date.month() as i32;

        result.push(crate::database::MonthData {
            year,
            month,
            weeks: [0; 5],
        });
    }

    // 查询最近12个月的数据
    let start_date = now - chrono::Duration::days(365);
    let start_date_str = start_date.format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT strftime('%Y', created_at) as year,
                strftime('%m', created_at) as month,
                strftime('%d', created_at) as day,
                COUNT(*) as count
         FROM notes
         WHERE created_at >= ?
         GROUP BY year, month, day
         ORDER BY year, month, day"
    )?;

    let mut rows = stmt.query_map([&start_date_str], |row| {
        let year_str: String = row.get(0)?;
        let month_str: String = row.get(1)?;
        let day_str: String = row.get(2)?;
        let count: i32 = row.get(3)?;

        let year = year_str.parse::<i32>().unwrap_or(0);
        let month = month_str.parse::<i32>().unwrap_or(0);
        let day = day_str.parse::<i32>().unwrap_or(0);

        Ok((year, month, day, count))
    })?;

    // 将查询结果填充到对应的月份中
    while let Some(Ok((year, month, day, count))) = rows.next() {
        // 找到对应的月份
        if let Some(month_data) = result.iter_mut().find(|m| m.year == year && m.month == month) {
            let week_index = ((day - 1) / 7).min(4) as usize;
            month_data.weeks[week_index] += count;
        }
    }

    Ok(result)
}