use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ulid::Ulid;

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

/// 初始化标签表
pub fn init_tables(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
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

    conn.execute("ALTER TABLE tags ADD COLUMN parent_id TEXT", []).ok();
    conn.execute("ALTER TABLE tags ADD COLUMN name TEXT", []).ok();
    conn.execute("ALTER TABLE tags ADD COLUMN full_name TEXT", []).ok();
    conn.execute("ALTER TABLE tags ADD COLUMN display_name TEXT", []).ok();
    conn.execute("ALTER TABLE tags ADD COLUMN pinned INTEGER DEFAULT 0", []).ok();

    // 创建索引
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tags_parent_id ON tags(parent_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tags_full_name ON tags(full_name)", [])?;

    Ok(())
}

/// 解析笔记内容中的标签
pub fn parse_tags_from_content(_conn: &Connection, content: &str) -> Vec<String> {
    let mut tags = HashSet::new();
    let html_tag_regex = regex::Regex::new(r#"<span class="tag">#([^<]+)</span>"#).unwrap();

    for caps in html_tag_regex.captures_iter(content) {
        if let Some(tag_name) = caps.get(1) {
            tags.insert(tag_name.as_str().to_string());
        }
    }

    tags.into_iter().collect()
}

/// 创建或更新标签（处理多级标签结构）
pub fn create_or_update_tags(conn: &Connection, tags: Vec<String>) -> SqliteResult<Vec<Tag>> {
    let mut created_tags = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    for tag_full_name in tags {
        let parts: Vec<&str> = tag_full_name.split('/').collect();
        let mut parent_id: Option<String> = None;
        let mut current_full_name = String::new();

        for (index, part) in parts.iter().enumerate() {
            if index == 0 {
                current_full_name = part.to_string();
            } else {
                current_full_name = format!("{}/{}", current_full_name, part);
            }

            let existing_tag = get_tag_by_full_name(conn, &current_full_name)?;

            let tag = if let Some(existing) = existing_tag {
                let updated_tag = Tag {
                    id: existing.id.clone(),
                    parent_id: parent_id.clone(),
                    name: part.to_string(),
                    full_name: current_full_name.clone(),
                    pinned: existing.pinned,
                    created_at: existing.created_at,
                    updated_at: now.clone(),
                };

                conn.execute(
                    "UPDATE tags SET parent_id = ?1, name = ?2, updated_at = ?3 WHERE id = ?4",
                    params![&parent_id, part, &now, &existing.id],
                )?;

                updated_tag
            } else {
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

                conn.execute(
                    "INSERT INTO tags (id, parent_id, name, full_name, display_name, pinned, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![&id, &parent_id, part, &current_full_name, part, 0, &now, &now],
                )?;

                new_tag
            };

            parent_id = Some(tag.id.clone());

            if index == parts.len() - 1 {
                created_tags.push(tag);
            }
        }
    }

    Ok(created_tags)
}

/// 根据完整名称获取标签
pub fn get_tag_by_full_name(conn: &Connection, full_name: &str) -> SqliteResult<Option<Tag>> {
    let mut stmt = conn.prepare(
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

/// 获取所有标签
pub fn get_all_tags(conn: &Connection) -> SqliteResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
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
pub fn delete_tag(conn: &Connection, id: &str) -> SqliteResult<()> {
    delete_tag_recursive(conn, id)?;
    Ok(())
}

/// 递归删除标签及其所有子标签
fn delete_tag_recursive(conn: &Connection, id: &str) -> SqliteResult<()> {
    let mut stmt = conn.prepare("SELECT id FROM tags WHERE parent_id = ?1")?;
    let child_ids: Vec<String> = stmt.query_map(params![id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for child_id in child_ids {
        delete_tag_recursive(conn, &child_id)?;
    }

    conn.execute("DELETE FROM tags WHERE id = ?", params![id])?;
    Ok(())
}

/// 置顶/取消置顶标签
pub fn pin_tag(conn: &Connection, id: &str) -> SqliteResult<()> {
    let mut stmt = conn.prepare("SELECT COALESCE(pinned, 0) FROM tags WHERE id = ?1")?;
    let mut tags = stmt.query_map(params![id], |row| {
        let pinned: i32 = row.get(0)?;
        Ok(pinned == 1)
    })?;

    let current_pinned = match tags.next() {
        Some(Ok(pinned)) => pinned,
        _ => return Ok(()),
    };

    let new_pinned = if current_pinned { 0 } else { 1 };
    conn.execute(
        "UPDATE tags SET pinned = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_pinned, &chrono::Utc::now().to_rfc3339(), id],
    )?;

    Ok(())
}

/// 搜索标签
pub fn search_tags(conn: &Connection, query: &str) -> SqliteResult<Vec<Tag>> {
    let pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, parent_id, name, COALESCE(full_name, name) as full_name, COALESCE(pinned, 0) as pinned, created_at, updated_at
             FROM tags
             WHERE name LIKE ?1 OR full_name LIKE ?1
             ORDER BY
               CASE
                 WHEN full_name = ?2 THEN 0
                 WHEN name = ?2 THEN 1
                 WHEN full_name LIKE ?2 || '%' THEN 2
                 WHEN name LIKE ?2 || '%' THEN 3
                 ELSE 4
               END,
               updated_at DESC,
               pinned DESC,
               full_name ASC
             LIMIT 5"
    )?;

    let tags = stmt.query_map(params![pattern, query], |row| {
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