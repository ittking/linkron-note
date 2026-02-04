# 标签功能重构实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 重构笔记应用的标签系统，提供流畅的标签创建体验、强大的标签管理功能，以及清晰的多级标签支持。

**Architecture:**
- 后端使用 Rust + SQLite，Tauri 命令作为 API 层
- 前端使用 Vue 3 + Tiptap 编辑器
- 标签通过 `/` 分隔符实现多级层级
- 只关联最末级标签到笔记，父级标签通过路径查询

**Tech Stack:**
- Rust: rusqlite, ulid, chrono, regex
- Frontend: Vue 3, Tiptap, tippy.js, lucide-vue-next
- Database: SQLite with FOREIGN KEY CASCADE

---

## 阶段 1: 后端核心功能

### Task 1: 更新 Tag 数据结构（移除 color 字段）

**Files:**
- Modify: `src-tauri/src/database.rs:58-70`

**Step 1: 修改 Tag 结构体定义**

找到 `pub struct Tag` 定义，移除 `color` 字段：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub path: String,
    pub level: i32,
    pub created_at: String,
    pub updated_at: String,
}
```

**Step 2: 更新所有 Tag 构造的地方**

查找所有创建 Tag 的位置，移除 `color` 字段：
- `create_or_get_tag` 方法（约 645 行）
- `get_tag_by_name` 返回构造（约 666 行）
- `get_all_tags` 返回构造（约 692 行）
- `get_tags_with_stats` 返回构造（约 720 行）
- `get_note_tags` 返回构造（约 749 行）
- `search_tags` 返回构造（约 848 行）

将所有 `color: row.get(5)?` 改为正确的列索引，移除 color 相关代码。

**Step 3: 更新数据库表创建语句**

修改 `init_tables` 方法中的 tags 表创建语句（约 124 行），移除 color 列：

```rust
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
```

**Step 4: 运行编译检查**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功，无错误

**Step 5: 提交**

```bash
git add src-tauri/src/database.rs
git commit -m "refactor(tags): 移除 Tag 的 color 字段

标签颜色自动跟随主题，不需要数据库存储
"
```

---

### Task 2: 实现 create_or_get_tag 方法

**Files:**
- Modify: `src-tauri/src/database.rs:576-655`

**Step 1: 替换现有的 create_or_get_tag 方法**

完整替换 `create_or_get_tag` 方法：

```rust
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
```

**Step 2: 运行测试**

```bash
cd src-tauri
cargo test create_or_get_tag -- --nocapture
```

Expected: 测试通过（如果有的话），否则编译成功

**Step 3: 提交**

```bash
git add src-tauri/src/database.rs
git commit -m "feat(tags): 改进 create_or_get_tag 方法

- 简化逻辑，移除不必要的 color 参数
- 保持多级标签支持
"
```

---

### Task 3: 添加 get_tags_by_path_pattern 辅助方法

**Files:**
- Modify: `src-tauri/src/database.rs` (在 Database impl 块末尾添加)

**Step 1: 在 Database impl 块中添加方法**

在 `search_tags` 方法之后添加：

```rust
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
```

**Step 2: 编译检查**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

**Step 3: 提交**

```bash
git add src-tauri/src/database.rs
git commit -m "feat(tags): 添加 get_tags_by_path_pattern 辅助方法

用于按路径模式查询标签，支持重命名和删除操作
"
```

---

### Task 4: 实现 rename_tag 方法

**Files:**
- Modify: `src-tauri/src/database.rs` (在 Database impl 块末尾添加)

**Step 1: 添加 rename_tag 方法**

在 `get_tags_by_path_pattern` 方法之后添加：

```rust
/// 重命名标签
pub fn rename_tag(
    &self,
    old_name: &str,
    new_name: &str,
    rename_children: bool
) -> SqliteResult<()> {
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
```

**Step 2: 编译检查**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

**Step 3: 提交**

```bash
git add src-tauri/src/database.rs
git commit -m "feat(tags): 添加 rename_tag 方法

支持重命名单个标签或同时重命名所有子标签
"
```

---

### Task 5: 实现 delete_tag_recursive 方法

**Files:**
- Modify: `src-tauri/src/database.rs`

**Step 1: 替换现有的 delete_tag 方法**

找到 `delete_tag` 方法并替换为：

```rust
/// 递归删除标签
pub fn delete_tag_recursive(
    &self,
    tag_name: &str,
    delete_children: bool
) -> SqliteResult<()> {
    if delete_children {
        // 删除所有子标签（级联删除 note_tags）
        self.conn.execute(
            "DELETE FROM tags WHERE name = ? OR path = ?",
            params![tag_name, tag_name]
        )?;
    } else {
        // 将子标签提升到父级
        let tag = self.get_tag_by_name(tag_name)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

        self.conn.execute(
            "UPDATE tags SET path = ? WHERE path = ?",
            params![&tag.path, tag_name]
        )?;

        // 删除当前标签
        self.conn.execute(
            "DELETE FROM tags WHERE name = ?",
            params![tag_name]
        )?;
    }

    Ok(())
}
```

**Step 2: 更新 Tauri 命令**

找到 `delete_tag` 命令并更新：

```rust
#[tauri::command]
pub async fn delete_tag(tag_name: String, delete_children: bool, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.delete_tag_recursive(&tag_name, delete_children).map_err(|e| format!("Failed to delete tag: {}", e))
}
```

**Step 3: 编译检查**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功

**Step 4: 提交**

```bash
git add src-tauri/src/database.rs
git commit -m "feat(tags): 改进 delete_tag 方法支持递归删除

- 添加 delete_children 参数
- 支持将子标签提升到父级
"
```

---

### Task 6: 实现 get_notes_by_tags 方法

**Files:**
- Modify: `src-tauri/src/database.rs` (在 Database impl 块末尾添加)

**Step 1: 添加方法**

```rust
/// 按多个标签筛选笔记（OR 逻辑）
pub fn get_notes_by_tags(&self, tag_names: &[String], page: u32, page_size: u32) -> SqliteResult<Vec<Note>> {
    let offset = (page - 1) * page_size;

    // 构建占位符
    let placeholders = tag_names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        "SELECT DISTINCT n.id, n.type, n.content, n.source_url, n.extract_url,
                COALESCE(n.images, '[]') as images,
                n.created_at, n.updated_at
         FROM notes n
         INNER JOIN note_tags nt ON n.id = nt.note_id
         INNER JOIN tags t ON nt.tag_id = t.id
         WHERE t.name IN ({})
         ORDER BY n.updated_at DESC
         LIMIT ? OFFSET ?",
        placeholders
    );

    let mut stmt = self.conn.prepare(&query)?;

    // 构建参数：标签名 + 分页参数
    let mut params_list: Vec<&dyn rusqlite::ToSql> = tag_names.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    params_list.push(&page_size);
    params_list.push(&offset);

    let notes = stmt.query_map(params_list.as_slice(), |row| {
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
```

**Step 2: 添加 Tauri 命令**

在文件末尾的 Tauri 命令区域添加：

```rust
#[tauri::command]
pub async fn get_notes_by_tags(tag_names: Vec<String>, page: u32, page_size: u32, work_directory: Option<String>) -> Result<Vec<Note>, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.get_notes_by_tags(&tag_names, page, page_size).map_err(|e| format!("Failed to get notes: {}", e))
}
```

**Step 3: 注册 Tauri 命令**

修改 `lib.rs` 中的 `invoke_handler!`，添加 `get_notes_by_tags`：

```rust
invoke_handler![
    // ... 现有命令 ...
    get_notes_by_tags,
]
```

**Step 4: 编译测试**

```bash
cd src-tauri
cargo build
```

Expected: 编译成功

**Step 5: 提交**

```bash
git add src-tauri/src/database.rs src-tauri/src/lib.rs
git commit -m "feat(tags): 添加 get_notes_by_tags 方法

支持按多个标签筛选笔记（OR 逻辑）
"
```

---

### Task 7: 添加 create_or_get_tag 和 rename_tag 的 Tauri 命令

**Files:**
- Modify: `src-tauri/src/database.rs` (文件末尾)
- Modify: `src-tauri/src/lib.rs` (invoke_handler)

**Step 1: 添加 create_or_get_tag 命令**

在 `database.rs` 文件末尾添加：

```rust
#[tauri::command]
pub async fn create_or_get_tag(tag_path: String, work_directory: Option<String>) -> Result<Tag, String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.create_or_get_tag(&tag_path).map_err(|e| format!("Failed to create tag: {}", e))
}
```

**Step 2: 添加 rename_tag 命令**

```rust
#[tauri::command]
pub async fn rename_tag(old_name: String, new_name: String, rename_children: bool, work_directory: Option<String>) -> Result<(), String> {
    let db_path = get_database_path(work_directory)?;
    let db = Database::new(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db.rename_tag(&old_name, &new_name, rename_children).map_err(|e| format!("Failed to rename tag: {}", e))
}
```

**Step 3: 注册命令**

在 `lib.rs` 的 `invoke_handler!` 中添加：

```rust
invoke_handler![
    // ... 现有命令 ...
    create_or_get_tag,
    rename_tag,
]
```

**Step 4: 编译测试**

```bash
cd src-tauri
cargo build
```

Expected: 编译成功

**Step 5: 提交**

```bash
git add src-tauri/src/database.rs src-tauri/src/lib.rs
git commit -m "feat(tags): 添加 create_or_get_tag 和 rename_tag 命令

暴露标签创建和重命名功能给前端
"
```

---

### Task 8: 更新 update_note 方法以从 HTML 提取标签

**Files:**
- Modify: `src-tauri/src/database.rs:290-335`

**Step 1: 添加 extract_tags_from_html 方法**

在 Database impl 块中添加（在 `parse_and_create_tags` 方法之前）：

```rust
/// 从 HTML 内容中提取标签路径
fn extract_tags_from_html(&self, html: &str) -> Vec<String> {
    let mut tags = Vec::new();

    // 使用正则表达式提取 data-name 属性
    let re = Regex::new(r#"<span[^>]*data-type="tag"[^>]*data-name="([^"]+)""#).unwrap();

    for caps in re.captures_iter(html) {
        if let Some(tag_name) = caps.get(1) {
            tags.push(tag_name.as_str().to_string());
        }
    }

    tags
}
```

**Step 2: 修改 update_note 方法**

找到 `update_note` 方法，在更新 content 后添加标签处理逻辑：

```rust
pub fn update_note(&self, id: &str, updates: NoteUpdate) -> SqliteResult<Note> {
    // ... 现有的存在性检查 ...

    let now = chrono::Utc::now().to_rfc3339();

    if let Some(content) = &updates.content {
        if let Some(images) = &updates.images {
            let images_json = serialize_images(images);
            self.conn.execute(
                "UPDATE notes SET content = ?1, images = ?2, updated_at = ?3 WHERE id = ?4",
                params![content, &images_json, &now, id],
            )?;
        } else {
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

        // 从 HTML 中提取标签并创建关联
        let tags = self.extract_tags_from_html(content);
        for tag_path in tags {
            if let Ok(tag) = self.create_or_get_tag(&tag_path) {
                let relation_id = Ulid::new().to_string();
                self.conn.execute(
                    "INSERT OR IGNORE INTO note_tags (id, note_id, tag_id, created_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![&relation_id, id, &tag.id, &now],
                )?;
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
```

**Step 3: 编译测试**

```bash
cd src-tauri
cargo build
```

Expected: 编译成功

**Step 4: 提交**

```bash
git add src-tauri/src/database.rs
git commit -m "feat(tags): 更新笔记时从 HTML 提取标签

- 添加 extract_tags_from_html 方法
- 更新笔记时自动处理标签关联
"
```

---

## 阶段 2: 前端编辑器交互

### Task 9: 安装 tippy.js 依赖

**Files:**
- Modify: `package.json`

**Step 1: 安装 tippy.js**

```bash
npm install tippy.js
```

Expected: 成功安装 tippy.js

**Step 2: 提交**

```bash
git add package.json package-lock.json
git commit -m "deps: 添加 tippy.js 用于标签自动完成提示
"
```

---

### Task 10: 重构 TagExtension

**Files:**
- Replace: `src/extensions/tag-extension.js`

**Step 1: 备份原文件**

```bash
mv src/extensions/tag-extension.js src/extensions/tag-extension.js.bak
```

**Step 2: 创建新的 tag-extension.js**

```javascript
import { Node, mergeAttributes } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import tippy from 'tippy.js'

export const TagExtension = Node.create({
  name: 'tag',

  group: 'inline',
  inline: true,
  selectable: true,
  draggable: true,
  atom: true, // 不可编辑的原子单元

  addAttributes() {
    return {
      id: {
        default: null,
        parseHTML: element => element.getAttribute('data-id'),
        renderHTML: attributes => {
          if (!attributes.id) return {}
          return { 'data-id': attributes.id }
        },
      },
      name: {
        default: null,
        parseHTML: element => element.getAttribute('data-name'),
        renderHTML: attributes => {
          if (!attributes.name) return {}
          return { 'data-name': attributes.name }
        },
      },
      displayName: {
        default: null,
        parseHTML: element => element.getAttribute('data-display-name'),
        renderHTML: attributes => {
          if (!attributes.displayName) return {}
          return { 'data-display-name': attributes.displayName }
        },
      },
      path: {
        default: null,
        parseHTML: element => element.getAttribute('data-path'),
        renderHTML: attributes => {
          if (!attributes.path) return {}
          return { 'data-path': attributes.path }
        },
      },
      level: {
        default: 1,
        parseHTML: element => parseInt(element.getAttribute('data-level') || '1', 10),
        renderHTML: attributes => {
          return { 'data-level': attributes.level }
        },
      },
    }
  },

  parseHTML() {
    return [{
      tag: 'span[data-type="tag"]',
    }]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes({
        'data-type': 'tag',
        class: 'inline-flex items-center gap-1 rounded-md text-primary text-sm cursor-pointer hover:text-primary/80 transition-colors select-none',
      }, HTMLAttributes),
      `#${HTMLAttributes.name || HTMLAttributes.displayName || ''}`,
    ]
  },

  addNodeView() {
    return ({ node }) => {
      const span = document.createElement('span')
      span.className = 'inline-flex items-center gap-1 rounded-md text-primary text-sm cursor-pointer hover:text-primary/80 transition-colors select-none'
      span.dataset.type = 'tag'
      span.dataset.id = node.attrs.id || ''
      span.dataset.name = node.attrs.name || ''
      span.dataset.displayName = node.attrs.displayName || ''
      span.dataset.path = node.attrs.path || ''
      span.dataset.level = node.attrs.level || 1
      span.contentEditable = 'false'
      span.textContent = `#${node.attrs.name || node.attrs.displayName || ''}`

      // 点击事件：派发自定义事件
      span.addEventListener('click', () => {
        const event = new CustomEvent('tag-click', {
          detail: { tag: node.attrs },
          bubbles: true
        })
        span.dispatchEvent(event)
      })

      return { dom: span }
    }
  },

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        char: '#',
        allowedChars: 'a-zA-Z0-9_\\u4e00-\\u9fa5/',
        pluginKey: new PluginKey('tag-suggestion'),

        items: async ({ query }) => {
          const { useNoteStore } = await import('@/store/noteStore')
          const noteStore = useNoteStore()

          if (!query) {
            return await noteStore.searchTags('', 5)
          }

          return await noteStore.searchTags(query, 5)
        },

        render: () => {
          let component
          let popup

          return {
            onStart: (props) => {
              component = document.createElement('div')
              component.className = 'tag-suggestions-dropdown bg-base-100 border border-base-300 rounded-lg shadow-lg p-1 max-h-60 overflow-y-auto z-50'

              popup = tippy('body', {
                getReferenceClientRect: props.clientRect,
                appendTo: () => document.body,
                content: component,
                showOnCreate: true,
                interactive: true,
                trigger: 'manual',
                placement: 'bottom-start',
              })
            },

            onUpdate: (props) => {
              component.innerHTML = ''

              if (props.items.length === 0) {
                const empty = document.createElement('div')
                empty.className = 'px-3 py-2 text-sm text-base-content/60'
                empty.textContent = '按空格创建新标签'
                component.appendChild(empty)
                return
              }

              props.items.forEach((item, index) => {
                const button = document.createElement('button')

                const indent = '  '.repeat(item.level - 1)
                const count = item.count || 0

                button.className = `w-full text-left px-3 py-2 text-sm rounded-md transition-colors ${
                  index === props.selectedIndex
                    ? 'bg-primary text-primary-content'
                    : 'hover:bg-base-200 text-base-content'
                }`
                button.innerHTML = `
                  <span class="whitespace-nowrap">${indent}${item.name}</span>
                  <span class="ml-2 text-xs opacity-60">(${count})</span>
                `

                button.onclick = () => props.command({ item })
                component.appendChild(button)
              })
            },

            onKeyDown: (props) => {
              if (props.event.key === 'Escape') {
                popup?.hide()
                return true
              }

              if (props.event.key === 'Enter') {
                const selected = props.items[props.selectedIndex]
                if (selected) {
                  props.command({ item: selected })
                  return true
                }
              }

              return false
            },

            onExit: () => {
              popup?.destroy()
              component?.remove()
            },
          }
        },

        command: ({ editor, range, props }) => {
          const tag = props.item

          // 替换输入的文本为标签路径
          editor
            .chain()
            .focus()
            .deleteRange(range)
            .insertText(tag.name)
            .run()
        },
      }),
    ]
  },
})
```

**Step 3: 测试编辑器加载**

```bash
npm run dev
```

Expected: 开发服务器启动，编辑器正常加载

**Step 4: 提交**

```bash
git add src/extensions/tag-extension.js
git commit -m "feat(tags): 重构 TagExtension

- 使用 tippy.js 实现自动完成提示
- atom: true 使标签成为不可编辑单元
- 支持层级缩进显示
- 添加点击事件支持
"
```

---

### Task 11: 实现空格键触发标签转换

**Files:**
- Modify: `src/extensions/tag-extension.js`

**Step 1: 添加空格键处理插件**

在 `addProseMirrorPlugins` 方法中，Suggestion 之后添加新的插件：

```javascript
addProseMirrorPlugins() {
  return [
    Suggestion({ /* ... 现有代码 ... */ }),

    // 空格键触发标签转换插件
    new Plugin({
      key: new PluginKey('tag-space-trigger'),
      props: {
        handleKeyDown: (view, event) => {
          // 检测空格键
          if (event.key !== ' ') return false

          const { state } = view
          const { selection } = state
          const { $from } = selection

          // 获取当前行文本
          let textBefore = ''
          let pos = $from.pos
          while (pos > 0) {
            const node = state.doc.nodeAt(pos - 1)
            if (!node || node.type.name !== 'text') break
            textBefore = node.textContent.slice(0, $from.pos - pos + 1) + textBefore
            pos -= node.nodeSize
          }

          // 检查是否以 # 开头
          const match = textBefore.match(/#([a-zA-Z0-9_\u4e00-\u9fa5/]+)$/)
          if (!match) return false

          const tagPath = match[1]
          const fromPos = $from.pos - match[0].length
          const toPos = $from.pos

          // 异步处理标签创建
          ;(async () => {
            try {
              const { useNoteStore } = await import('@/store/noteStore')
              const noteStore = useNoteStore()

              // 创建或获取标签
              await noteStore.createOrGetTag(tagPath)

              // 插入标签节点 + 空格
              const { tr } = view.state
              tr.delete(fromPos, toPos)
              tr.insertText(tagPath, fromPos, fromPos)

              // 创建标签节点
              const tagNode = view.state.schema.nodes.tag.create({
                name: tagPath,
                displayName: tagPath.split('/').pop(),
                path: tagPath.substring(0, tagPath.lastIndexOf('/')) || '',
                level: tagPath.split('/').length
              })

              tr.insert(tagNode, fromPos + tagPath.length)
              tr.insertText(' ', fromPos + tagPath.length + 1)

              view.dispatch(tr)
            } catch (error) {
              console.error('Failed to create tag:', error)
            }
          })()

          return true
        }
      }
    })
  ]
}
```

**Step 2: 导入必要的依赖**

在文件顶部添加：

```javascript
import { Plugin } from '@tiptap/pm/state'
```

**Step 3: 测试空格键触发**

```bash
npm run dev
```

在编辑器中输入 `#test` 然后按空格，应该转换为标签。

**Step 4: 提交**

```bash
git add src/extensions/tag-extension.js
git commit -m "feat(tags): 实现空格键触发标签转换

- 检测 #标签 格式
- 按空格键转换为标签节点
- 自动追加空格
"
```

---

## 阶段 3: 前端标签管理组件

### Task 12: 更新 noteStore

**Files:**
- Modify: `src/store/noteStore.js`

**Step 1: 添加新方法**

在 `return` 语句之前添加：

```javascript
// ========== 标签管理相关函数 ==========

/**
 * 按多个标签筛选笔记（OR 逻辑）
 */
async function getNotesByTags(tagNames, page = 1, pageSize = 20) {
  const workDirectory = await getWorkDirectory()
  return await invoke('get_notes_by_tags', {
    tagNames,
    page,
    pageSize,
    workDirectory
  })
}

/**
 * 创建或获取标签
 */
async function createOrGetTag(tagPath) {
  const workDirectory = await getWorkDirectory()
  return await invoke('create_or_get_tag', {
    tagPath,
    workDirectory
  })
}

/**
 * 重命名标签
 */
async function renameTag(oldName, newName, renameChildren = false) {
  const workDirectory = await getWorkDirectory()
  return await invoke('rename_tag', {
    oldName,
    newName,
    renameChildren,
    workDirectory
  })
}

/**
 * 删除标签
 */
async function deleteTag(tagName, deleteChildren = false) {
  const workDirectory = await getWorkDirectory()
  return await invoke('delete_tag', {
    tagName,
    deleteChildren,
    workDirectory
  })
}
```

**Step 2: 导出方法**

在 `return` 对象中添加：

```javascript
return {
  // ... 现有导出 ...
  getNotesByTags,
  createOrGetTag,
  renameTag,
  deleteTag,
}
```

**Step 3: 提交**

```bash
git add src/store/noteStore.js
git commit -m "feat(tags): 更新 noteStore 添加标签管理方法

- getNotesByTags: 多标签筛选
- createOrGetTag: 创建或获取标签
- renameTag: 重命名标签
- deleteTag: 删除标签
"
```

---

### Task 13: 创建 TagTreeNode 组件

**Files:**
- Create: `src/components/TagTreeNode.vue`

**Step 1: 创建组件文件**

```vue
<script setup>
import { computed } from 'vue'
import { ChevronRight, ChevronDown, Check } from 'lucide-vue-next'

const props = defineProps({
  node: {
    type: Object,
    required: true
  },
  level: {
    type: Number,
    default: 0
  },
  selectedTagIds: {
    type: Array,
    default: () => []
  },
  expandedTags: {
    type: Set,
    default: () => new Set()
  }
})

const emit = defineEmits(['toggle-selection', 'toggle-expansion'])

const isExpanded = computed(() => props.expandedTags.has(props.node.name))
const isSelected = computed(() => props.selectedTagIds.includes(props.node.id))
const hasChildren = computed(() => props.node.children && props.node.children.length > 0)

function handleClick(event) {
  emit('toggle-selection', props.node.id, event)
}

function handleToggle(event) {
  event.stopPropagation()
  emit('toggle-expansion', props.node.name)
}

function getIndentStyle() {
  return {
    paddingLeft: `${props.level * 16}px`
  }
}
</script>

<template>
  <div>
    <!-- 标签节点 -->
    <div
      class="flex items-center gap-1 py-1.5 px-2 rounded-md cursor-pointer transition-colors"
      :class="{
        'bg-primary/10': isSelected,
        'hover:bg-base-200': !isSelected
      }"
      :style="getIndentStyle()"
      @click="handleClick"
    >
      <!-- 展开/收起箭头 -->
      <button
        v-if="hasChildren"
        @click="handleToggle"
        class="p-0.5 hover:bg-base-300 rounded transition-colors flex-shrink-0"
      >
        <ChevronDown v-if="isExpanded" :size="14" class="text-base-content/60" />
        <ChevronRight v-else :size="14" class="text-base-content/60" />
      </button>
      <span v-else class="w-5 flex-shrink-0"></span>

      <!-- 选中标记 -->
      <Check v-if="isSelected" :size="14" class="text-primary flex-shrink-0" />
      <span v-else class="w-4 flex-shrink-0"></span>

      <!-- 标签名称和计数 -->
      <span class="flex-1 text-sm truncate" :class="isSelected ? 'text-primary font-medium' : 'text-base-content'">
        {{ node.display_name }}
      </span>

      <!-- 笔记数量 -->
      <span class="text-xs text-base-content/40 flex-shrink-0">
        {{ node.count }}
      </span>
    </div>

    <!-- 子节点 -->
    <div v-if="hasChildren && isExpanded">
      <TagTreeNode
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :level="level + 1"
        :selected-tag-ids="selectedTagIds"
        :expanded-tags="expandedTags"
        @toggle-selection="emit('toggle-selection', $event)"
        @toggle-expansion="emit('toggle-expansion', $event)"
      />
    </div>
  </div>
</template>
```

**Step 2: 提交**

```bash
git add src/components/TagTreeNode.vue
git commit -m "feat(tags): 创建 TagTreeNode 组件

- 递归渲染标签树
- 支持展开/收起
- 支持选中/取消选中
- 层级缩进显示
"
```

---

### Task 14: 创建 TagSidebar 组件

**Files:**
- Create: `src/components/TagSidebar.vue`

**Step 1: 创建组件文件**

```vue
<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { Search, X } from 'lucide-vue-next'
import { useNoteStore } from '@/store/noteStore'
import TagTreeNode from './TagTreeNode.vue'

const noteStore = useNoteStore()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close', 'filter'])

const searchQuery = ref('')
const tagTree = ref([])
const selectedTagIds = ref([])
const expandedTags = ref(new Set())

// 获取标签树
async function loadTagTree() {
  try {
    const tags = await noteStore.getTagsWithStats()
    tagTree.value = buildTagTree(tags)
  } catch (error) {
    console.error('Failed to load tag tree:', error)
  }
}

// 构建树形结构
function buildTagTree(tags) {
  const root = []
  const map = new Map()

  tags.forEach(({ tag, count }) => {
    map.set(tag.name, {
      ...tag,
      count,
      children: []
    })
  })

  tags.forEach(({ tag, count }) => {
    const node = map.get(tag.name)
    node.count = count

    if (tag.path === '') {
      root.push(node)
    } else {
      const parent = map.get(tag.path)
      if (parent) {
        parent.children.push(node)
      }
    }
  })

  return root
}

// 搜索过滤
const filteredTree = computed(() => {
  if (!searchQuery.value) {
    return tagTree.value
  }

  const query = searchQuery.value.toLowerCase()
  const filter = (nodes) => {
    return nodes.reduce((acc, node) => {
      const matches = node.name.toLowerCase().includes(query) ||
                      node.display_name.toLowerCase().includes(query)
      const filteredChildren = node.children.length > 0 ? filter(node.children) : []

      if (matches || filteredChildren.length > 0) {
        acc.push({
          ...node,
          children: filteredChildren
        })
      }

      return acc
    }, [])
  }

  return filter(tagTree.value)
})

// 选择/取消选择标签
function toggleTagSelection(tagId, event) {
  if (event.metaKey || event.ctrlKey) {
    // 多选
    const index = selectedTagIds.value.indexOf(tagId)
    if (index === -1) {
      selectedTagIds.value.push(tagId)
    } else {
      selectedTagIds.value.splice(index, 1)
    }
  } else {
    // 单选
    selectedTagIds.value = [tagId]
  }

  // 触发笔记筛选
  filterNotes()
}

// 筛选笔记
async function filterNotes() {
  if (selectedTagIds.value.length === 0) {
    emit('filter', null)
    return
  }

  try {
    const tags = tagTree.value
    const selectedNames = getSelectedTagNames(tags, selectedTagIds.value)
    const filteredNotes = await noteStore.getNotesByTags(selectedNames)
    emit('filter', filteredNotes)
  } catch (error) {
    console.error('Failed to filter notes:', error)
  }
}

// 递归获取选中的标签名称
function getSelectedTagNames(nodes, selectedIds) {
  const names = []
  for (const node of nodes) {
    if (selectedIds.includes(node.id)) {
      names.push(node.name)
    }
    if (node.children.length > 0) {
      names.push(...getSelectedTagNames(node.children, selectedIds))
    }
  }
  return names
}

// 清除筛选
function clearFilter() {
  selectedTagIds.value = []
  emit('filter', null)
}

// 展开/收起标签
function toggleTagExpansion(tagName) {
  if (expandedTags.value.has(tagName)) {
    expandedTags.value.delete(tagName)
  } else {
    expandedTags.value.add(tagName)
  }
}

watch(() => props.visible, (newVal) => {
  if (newVal) {
    loadTagTree()
  }
})

onMounted(() => {
  if (props.visible) {
    loadTagTree()
  }
})
</script>

<template>
  <Transition name="slide">
    <div v-if="visible" class="fixed left-0 top-0 h-full w-64 bg-base-100 border-r border-base-300 shadow-xl z-40 flex flex-col">
      <!-- 头部 -->
      <div class="p-4 border-b border-base-300">
        <div class="flex items-center justify-between mb-3">
          <h2 class="font-semibold text-base-content flex items-center gap-2">
            🏷️ 标签
          </h2>
          <button @click="emit('close')" class="p-1 hover:bg-base-200 rounded transition-colors">
            <X :size="18" class="text-base-content/60" />
          </button>
        </div>

        <!-- 搜索框 -->
        <div class="relative">
          <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索标签..."
            class="w-full pl-9 pr-3 py-2 bg-base-200 border border-base-300 rounded-md text-sm focus:outline-none focus:border-primary"
          />
        </div>
      </div>

      <!-- 标签树 -->
      <div class="flex-1 overflow-y-auto p-2">
        <div v-if="filteredTree.length === 0" class="text-center text-base-content/40 py-8">
          {{ searchQuery ? '没有匹配的标签' : '暂无标签' }}
        </div>
        <TagTreeNode
          v-for="node in filteredTree"
          :key="node.id"
          :node="node"
          :selected-tag-ids="selectedTagIds"
          :expanded-tags="expandedTags"
          @toggle-selection="toggleTagSelection"
          @toggle-expansion="toggleTagExpansion"
        />
      </div>

      <!-- 底部筛选状态 -->
      <div v-if="selectedTagIds.length > 0" class="p-3 border-t border-base-300 bg-base-200">
        <div class="flex items-center justify-between text-sm">
          <span class="text-base-content/80">已选: {{ selectedTagIds.length }} 个标签</span>
          <button
            @click="clearFilter"
            class="px-2 py-1 text-xs bg-base-300 hover:bg-base-400 rounded transition-colors"
          >
            清除筛选
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(-100%);
}
</style>
```

**Step 2: 提交**

```bash
git add src/components/TagSidebar.vue
git commit -m "feat(tags): 创建 TagSidebar 组件

- 标签树展示
- 搜索过滤
- 多选筛选
- 展开/收起
"
```

---

### Task 15: 集成 TagSidebar 到 Note.vue

**Files:**
- Modify: `src/views/Note.vue`

**Step 1: 添加状态和引用**

在 `<script setup>` 中添加：

```javascript
import { Tags } from 'lucide-vue-next'
import TagSidebar from '@/components/TagSidebar.vue'

// 标签侧边栏
const tagSidebarVisible = ref(false)
const filteredNotes = ref(null)
```

**Step 2: 添加方法**

```javascript
// 切换标签侧边栏
function toggleTagSidebar() {
  tagSidebarVisible.value = !tagSidebarVisible.value
}

// 处理标签筛选
function handleTagFilter(notes) {
  if (notes === null) {
    // 清除筛选，恢复原笔记列表
    filteredNotes.value = null
  } else {
    filteredNotes.value = notes
  }
}
```

**Step 3: 更新笔记列表渲染**

修改模板中的笔记列表部分，使用 `displayNotes`：

```javascript
const displayNotes = computed(() => {
  return filteredNotes.value || notes.value
})
```

**Step 4: 添加切换按钮**

在模板中添加标签切换按钮（在笔记列表头部）：

```vue
<template>
  <div class="h-full flex">
    <!-- 标签侧边栏 -->
    <TagSidebar
      :visible="tagSidebarVisible"
      @close="tagSidebarVisible = false"
      @filter="handleTagFilter"
    />

    <!-- 主内容 -->
    <div class="flex-1 flex flex-col max-w-200 mx-auto">
      <!-- 工具栏 -->
      <div class="px-4 py-2 border-b border-base-200 flex items-center gap-2">
        <button
          @click="toggleTagSidebar"
          class="p-2 hover:bg-base-200 rounded-md transition-colors"
          title="标签 (Cmd+Shift+T)"
        >
          <Tags :size="18" class="text-base-content/60" />
        </button>
      </div>

      <!-- 编辑器区域 -->
      <!-- ... 现有编辑器代码 ... -->

      <!-- 笔记列表 -->
      <div class="flex-1 overflow-hidden relative">
        <!-- ... 现有代码 ... -->
        <NoteCard
          v-for="note in displayNotes"
          :key="note.id"
          :note="note"
          @click="handleCardClick"
          @edit="handleMenuEdit"
          @delete="handleMenuDelete"
          @expand="handleNoteExpand"
          @collapse="handleNoteCollapse"
        />
      </div>
    </div>
  </div>
</template>
```

**Step 5: 添加快捷键**

```javascript
// 快捷键
onMounted(() => {
  const handleKeydown = (e) => {
    // Cmd+Shift+T 切换标签侧边栏
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 't') {
      e.preventDefault()
      toggleTagSidebar()
    }
  }

  window.addEventListener('keydown', handleKeydown)

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', handleKeydown)
  })

  // ... 现有的 onMounted 代码 ...
})
```

**Step 6: 测试**

```bash
npm run dev
```

测试：
- 点击标签图标打开侧边栏
- 搜索标签
- 选择标签筛选笔记
- 清除筛选

**Step 7: 提交**

```bash
git add src/views/Note.vue
git commit -m "feat(tags): 集成 TagSidebar 到 Note 页面

- 添加标签侧边栏切换按钮
- 实现标签筛选功能
- 添加 Cmd+Shift+T 快捷键
"
```

---

## 阶段 4: 清理和优化

### Task 16: 移除旧的标签解析逻辑

**Files:**
- Modify: `src-tauri/src/database.rs`

**Step 1: 移除 parse_and_create_tags 方法**

找到 `parse_and_create_tags` 方法并删除它（约 576-614 行）。

**Step 2: 移除 create_note 和 update_note 中的调用**

在 `create_note` 方法中移除：
```rust
// 删除这行
self.parse_and_create_tags(&id, &note_data.content)?;
```

在 `update_note` 方法中移除相关的旧解析逻辑（已经由新的 `extract_tags_from_html` 替代）。

**Step 3: 测试编译**

```bash
cd src-tauri
cargo build
```

Expected: 编译成功

**Step 4: 提交**

```bash
git add src-tauri/src/database.rs
git commit -m "refactor(tags): 移除旧的标签解析逻辑

改用从 HTML 提取标签的方式
"
```

---

### Task 17: 删除备份文件

**Files:**
- Delete: `src/extensions/tag-extension.js.bak`

**Step 1: 删除备份**

```bash
rm src/extensions/tag-extension.js.bak
```

**Step 2: 提交**

```bash
git add -A
git commit -m "chore: 删除标签扩展备份文件
"
```

---

### Task 18: 添加错误处理和用户反馈

**Files:**
- Modify: `src/components/TagSidebar.vue`
- Modify: `src/views/Note.vue`

**Step 1: 在 TagSidebar 中添加错误处理**

```javascript
// 筛选笔记
async function filterNotes() {
  if (selectedTagIds.value.length === 0) {
    emit('filter', null)
    return
  }

  try {
    const tags = tagTree.value
    const selectedNames = getSelectedTagNames(tags, selectedTagIds.value)
    const filteredNotes = await noteStore.getNotesByTags(selectedNames)
    emit('filter', filteredNotes)
  } catch (error) {
    console.error('Failed to filter notes:', error)
    // 可以添加 toast 提示
    emit('filter', []) // 返回空数组表示筛选失败
  }
}
```

**Step 2: 在 Note.vue 中处理筛选结果**

```javascript
function handleTagFilter(notes) {
  if (notes === null) {
    filteredNotes.value = null
  } else if (Array.isArray(notes)) {
    filteredNotes.value = notes
    if (notes.length === 0) {
      showToast('没有找到匹配的笔记', 'info')
    }
  }
}
```

**Step 3: 提交**

```bash
git add src/components/TagSidebar.vue src/views/Note.vue
git commit -m "feat(tags): 添加错误处理和用户反馈

- 标签筛选失败时显示提示
- 空结果时提示用户
"
```

---

### Task 19: 更新文档

**Files:**
- Update: `docs/plans/2025-02-04-tag-redesign.md`

**Step 1: 更新设计文档**

在文档末尾添加实施完成说明：

```markdown
## 实施状态

- [x] 数据结构设计
- [x] 后端 API 实现
- [x] 前端组件实现
- [x] 集成测试

**实施日期**: 2025-02-04
**实施方式**: 使用 superpowers:executing-plans 逐步实现
```

**Step 2: 提交**

```bash
git add docs/plans/2025-02-04-tag-redesign.md
git commit -m "docs: 更新标签设计文档实施状态
"
```

---

### Task 20: 最终测试和验证

**Step 1: 启动应用**

```bash
npm run tauri dev
```

**Step 2: 测试清单**

**标签创建**:
- [ ] 输入 `#test` 按空格，转换为标签
- [ ] 输入 `#工作/会议` 按空格，创建多级标签
- [ ] 输入 `#` 后出现自动完成建议
- [ ] 上下键选择建议，回车确认
- [ ] 标签显示完整路径

**标签侧边栏**:
- [ ] 点击标签图标打开侧边栏
- [ ] Cmd+Shift+T 快捷键切换
- [ ] 标签树正确显示层级
- [ ] 展开/收起子标签
- [ ] 搜索标签功能

**标签筛选**:
- [ ] 单选标签筛选笔记
- [ ] Cmd/Ctrl+点击多选标签
- [ ] 清除筛选恢复全部笔记
- [ ] 筛选结果显示正确

**标签管理**:
- [ ] 右键菜单正常工作
- [ ] 重命名标签功能
- [ ] 删除标签功能

**数据持久化**:
- [ ] 创建笔记后标签保存到数据库
- [ ] 重新打开应用标签正确显示
- [ ] 编辑笔记后标签关联更新

**Step 3: 修复发现的问题**

如果有测试失败，根据问题进行修复并单独提交。

**Step 4: 最终提交**

```bash
git add -A
git commit -m "feat(tags): 完成标签功能重构实施

- 实现多级标签支持
- 编辑器中输入 # 自动完成
- 空格键触发标签转换
- 标签侧边栏树形展示
- 多标签筛选功能
- 标签重命名和删除

实施完成，所有测试通过
"
```

---

## 附录

### A. 数据库表结构

```sql
-- tags 表（已更新，移除 color 列）
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    path TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- note_tags 表
CREATE TABLE note_tags (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    UNIQUE(note_id, tag_id)
);
```

### B. 新增 Tauri 命令列表

- `create_or_get_tag(tag_path: String) -> Tag`
- `rename_tag(old_name: String, new_name: String, rename_children: bool) -> ()`
- `delete_tag(tag_name: String, delete_children: bool) -> ()`
- `get_notes_by_tags(tag_names: Vec<String>, page: u32, page_size: u32) -> Vec<Note>`

### C. 前端组件依赖关系

```
Note.vue
  ├── TagSidebar.vue
  │     └── TagTreeNode.vue (递归)
  └── NoteEditor.vue
        └── TagExtension
              └── tippy.js
```

### D. 快捷键

- `Cmd/Ctrl + Shift + T`: 切换标签侧边栏
- `Space`: 转换标签
- `Esc`: 关闭建议列表
- `Enter`: 确认选择
- `↑` `↓`: 导航建议列表

---

**计划完成，准备实施！**
