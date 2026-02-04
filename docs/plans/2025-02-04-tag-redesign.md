# 标签功能重构设计文档

**日期**: 2025-02-04
**状态**: 待实现
**作者**: Claude Code

---

## 目录

- [1. 概述](#1-概述)
- [2. 当前问题](#2-当前问题)
- [3. 需求分析](#3-需求分析)
- [4. 数据结构设计](#4-数据结构设计)
- [5. 编辑器交互设计](#5-编辑器交互设计)
- [6. 标签管理功能](#6-标签管理功能)
- [7. 后端 API 设计](#7-后端-api-设计)
- [8. 前端组件实现](#8-前端组件实现)
- [9. 实施计划](#9-实施计划)

---

## 1. 概述

本文档描述了 iterm 应用标签功能的完整重构方案。重构旨在提供流畅的标签创建体验、强大的标签管理能力，以及清晰的多级标签支持。

### 核心目标

1. **直观的标签创建**: 输入 `#` 自动提示，空格确认转换
2. **强大的标签管理**: 侧边栏树形展示，支持重命名、删除
3. **灵活的标签筛选**: 多选标签（OR 逻辑）筛选笔记
4. **清晰的层级结构**: 支持无限层级，用 `/` 分隔

---

## 2. 当前问题

1. **标签编辑器体验不好**: 输入、选择、转换流程不流畅
2. **标签管理功能缺失**: 无法查看、编辑、删除标签
3. **多级标签逻辑混乱**: 父子关系不清晰，层级显示混乱
4. **整体架构需要重做**: 从数据结构到 UI 需要完整重新设计

---

## 3. 需求分析

### 3.1 标签形式

采用**多级标签（树形结构）**:
- 使用 `/` 分隔层级: `#工作/项目A/会议`
- 支持无限层级
- 父子关系通过 `/` 分隔符体现

### 3.2 编辑器交互

**输入触发**: 输入 `#` 字符

**自动完成**:
- 显示最近使用的 5 条标签
- 实时过滤匹配
- 支持完整路径和分段匹配

**键盘操作**:
| 按键 | 行为 |
|------|------|
| `↑` `↓` | 在建议列表中导航 |
| `Enter` | 将选中标签的完整路径替换当前输入 |
| `Esc` | 关闭建议列表 |
| `Space` | **触发标签转换** |

**转换流程**:
```
输入: #工[Enter] → 替换为 #工作/项目A
按空格: 转换为 <span data-type="tag">#工作/项目A</span> + " "
```

### 3.3 多级标签处理

**规则**:
- 使用 `/` 分隔层级
- 支持无限层级: `#一级/二级/三级/...`
- 自动创建父级标签（如果不存在）

**示例**:
```
输入: #技术/前端/Vue/组件开发
自动创建:
  - 技术 (level 1)
  - 技术/前端 (level 2)
  - 技术/前端/Vue (level 3)
  - 技术/前端/Vue/组件开发 (level 4)
关联: 只关联最末级标签
```

### 3.4 标签节点行为

- **显示**: 完整路径 `#工作/项目A`
- **不可编辑**: `contentEditable="false"`
- **整体单元**: 类似 emoji，光标直接跳过
- **点击**: 触发筛选该标签的笔记
- **删除**: Backspace/Delete 删除整个标签

### 3.5 标签管理

**侧边栏**:
- 位置: 左侧，可收起
- 树形结构展示标签层级
- 显示每个标签的笔记数量
- 支持搜索过滤
- 点击展开/收起子标签

**筛选逻辑**:
- 支持多选（OR 逻辑）
- 同时选中多个标签，显示包含任一标签的笔记
- 按住 Cmd/Ctrl 点击进行多选

**右键菜单**:
- 重命名（可同时重命名子标签）
- 删除（可递归删除子标签）
- 查看关联的笔记

---

## 4. 数据结构设计

### 4.1 数据模型

```rust
pub struct Tag {
    pub id: String,              // ULID 唯一标识
    pub name: String,            // 完整路径，如 "工作/项目A"
    pub display_name: String,    // 显示名称，如 "项目A"
    pub path: String,            // 父级路径，如 "工作"（顶级为空字符串）
    pub level: i32,              // 层级深度，从 1 开始
    pub created_at: String,      // 创建时间
    pub updated_at: String,      // 更新时间
}

pub struct TagStats {
    #[serde(flatten)]
    pub tag: Tag,
    pub count: i64,              // 关联的笔记数量
}

pub struct TagNode {
    #[serde(flatten)]
    pub tag: Tag,
    pub count: i64,
    pub children: Vec<TagNode>,  // 子标签树
}
```

### 4.2 数据库表结构

**tags 表**:
```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,         -- 完整路径
    display_name TEXT NOT NULL,        -- 显示名称
    path TEXT NOT NULL,                -- 父级路径
    level INTEGER NOT NULL DEFAULT 1,  -- 层级深度
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

**note_tags 表**:
```sql
CREATE TABLE note_tags (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,              -- 只关联最末级标签
    created_at TEXT NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    UNIQUE(note_id, tag_id)
);
```

**索引**:
```sql
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_path ON tags(path);
CREATE INDEX idx_tags_level ON tags(level);
CREATE INDEX idx_tags_updated_at ON tags(updated_at DESC);
CREATE INDEX idx_note_tags_note_id ON note_tags(note_id);
CREATE INDEX idx_note_tags_tag_id ON note_tags(tag_id);
```

---

## 5. 编辑器交互设计

### 5.1 自动完成机制

**触发条件**: 输入 `#` 字符

**匹配逻辑**:
```javascript
// 1. 完整路径匹配: #工 可匹配 "工作/项目A"
// 2. 分段匹配: #工/项 可匹配 "工作/项目A"
// 3. 显示名称匹配: #项 可匹配 "工作/项目A"

SELECT * FROM tags
WHERE name LIKE ? || '%'
   OR display_name LIKE ? || '%'
ORDER BY updated_at DESC
LIMIT 5;
```

**建议列表 UI**（层级缩进）:
```
工作/项目A (12)
  工作/会议 (8)
技术/Vue (15)
  技术/React (10)
```

### 5.2 新标签创建

**场景**: 输入的标签路径不存在

**处理逻辑**:
```javascript
// 输入: #新品/活动/春节
// 解析: ['新品', '活动', '春节']

async function createOrGetTag(path) {
  const parts = path.split('/')
  let currentPath = ''

  for (let i = 0; i < parts.length; i++) {
    const part = parts[i]
    const fullPath = i === 0 ? part : `${currentPath}/${part}`

    // 检查标签是否存在
    let tag = await getTagByName(fullPath)

    if (!tag) {
      // 创建新标签
      tag = await createTag({
        name: fullPath,
        displayName: part,
        path: currentPath,
        level: i + 1
      })
    }

    currentPath = fullPath
  }

  return tag // 返回最末级标签
}
```

### 5.3 标签节点属性

```javascript
{
  type: 'tag',
  attrs: {
    name: '工作/项目A',      // 完整路径
    displayName: '项目A',     // 显示名称
    path: '工作',             // 父级路径
    level: 2                  // 层级深度
  }
}
```

**HTML 渲染**:
```html
<span data-type="tag"
      data-name="工作/项目A"
      data-display-name="项目A"
      data-path="工作"
      data-level="2"
      class="inline-flex items-center rounded-md text-primary text-sm cursor-pointer hover:text-primary/80 transition-colors select-none"
      contenteditable="false">
  #工作/项目A
</span>
```

### 5.4 空格键触发转换

```javascript
// 检测空格键触发转换
if (event.key === ' ') {
  const textBeforeCursor = editor.state.doc.textBetween(from, to)

  if (textBeforeCursor.startsWith('#')) {
    const tagPath = textBeforeCursor.slice(1) // 去掉 #

    // 创建或获取标签
    await createOrGetTag(tagPath)

    // 插入标签节点 + 空格
    editor.chain()
      .deleteRange({ from, to })
      .insertContent({
        type: 'tag',
        attrs: { name: tagPath }
      })
      .insertContent(' ')
      .run()
  }
}
```

---

## 6. 标签管理功能

### 6.1 侧边栏 UI

**布局结构**:
```
┌─────────────────────────────┐
│  🏷️ 标签              [×]   │
├─────────────────────────────┤
│  🔍 [搜索标签...]           │
├─────────────────────────────┤
│  ▼ 工作 (12)                │
│    ✓ 项目A (5)              │
│      会议 (7)               │
│  ▶ 技术 (20)                │
│  ▶ 生活 (15)                │
├─────────────────────────────┤
│  [已选: 2个标签] [清除筛选]  │
└─────────────────────────────┘
```

### 6.2 树形结构渲染

```javascript
// 将扁平的标签列表转换为树形结构
function buildTagTree(tags) {
  const root = []
  const map = new Map()

  // 创建所有节点的映射
  tags.forEach(tag => {
    map.set(tag.name, {
      ...tag,
      children: []
    })
  })

  // 构建树形结构
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
```

### 6.3 标签筛选逻辑

**多选（OR 逻辑）**:
```javascript
// 选中的标签 ID 数组
const selectedTagIds = ref([])

// 筛选笔记
async function filterByTags() {
  if (selectedTagIds.value.length === 0) {
    return await loadNotes()
  }

  const filteredNotes = await invoke('get_notes_by_tags', {
    tagIds: selectedTagIds.value
  })

  notes.value = filteredNotes
}
```

**后端 SQL**:
```sql
SELECT DISTINCT n.*
FROM notes n
INNER JOIN note_tags nt ON n.id = nt.note_id
WHERE nt.tag_id IN (?, ?, ...)
ORDER BY n.updated_at DESC
```

### 6.4 标签重命名

**对话框**:
```
┌──────────────────────────┐
│  重命名标签              │
├──────────────────────────┤
│  当前: #工作/项目A        │
│  新名称: [____________]   │
│                          │
│  ☐ 同时重命名子标签       │
│    ☑ 工作/项目A/子任务    │
│    ☑ 工作/项目A/会议      │
├──────────────────────────┤
│      [取消]  [确定]       │
└──────────────────────────┘
```

**处理逻辑**:
```sql
-- 更新当前标签
UPDATE tags
SET name = ?, display_name = ?, updated_at = ?
WHERE name = ?;

-- 更新子标签路径（如果勾选）
UPDATE tags
SET name = REPLACE(name, ?, ?),
    path = REPLACE(path, ?, ?),
    updated_at = ?
WHERE path LIKE ? || '%';
```

### 6.5 标签删除

**确认对话框**:
```
┌──────────────────────────┐
│  删除标签                │
├──────────────────────────┤
│  确定要删除 "工作/项目A"? │
│                          │
│  ☐ 同时删除子标签         │
│    ☐ 工作/项目A/子任务    │
│    ☐ 工作/项目A/会议      │
│                          │
│  这将同时移除 5 个笔记     │
│  中的标签关联。           │
├──────────────────────────┤
│      [取消]  [删除]       │
└──────────────────────────┘
```

**级联删除**:
```sql
-- SQLite 的 FOREIGN KEY ON DELETE CASCADE
DELETE FROM tags WHERE name = ?;
```

---

## 7. 后端 API 设计

### 7.1 标签查询 API

```rust
/// 获取所有标签（带统计信息）
#[tauri::command]
pub async fn get_tags_with_stats(
    work_directory: Option<String>
) -> Result<Vec<TagStats>, String>;

/// 获取标签树形结构
#[tauri::command]
pub async fn get_tag_tree(
    work_directory: Option<String>
) -> Result<Vec<TagNode>, String>;
```

### 7.2 标签 CRUD API

```rust
/// 创建或获取标签（支持多级）
#[tauri::command]
pub async fn create_or_get_tag(
    tag_path: String,
    work_directory: Option<String>
) -> Result<Tag, String>;

/// 重命名标签
#[tauri::command]
pub async fn rename_tag(
    old_name: String,
    new_name: String,
    rename_children: bool,
    work_directory: Option<String>
) -> Result<(), String>;

/// 删除标签
#[tauri::command]
pub async fn delete_tag(
    tag_name: String,
    delete_children: bool,
    work_directory: Option<String>
) -> Result<(), String>;
```

### 7.3 标签搜索 API

```rust
/// 搜索标签（用于自动完成）
#[tauri::command]
pub async fn search_tags(
    keyword: String,
    limit: u32,
    work_directory: Option<String>
) -> Result<Vec<Tag>, String>;
```

### 7.4 笔记筛选 API

```rust
/// 按多个标签筛选笔记（OR 逻辑）
#[tauri::command]
pub async fn get_notes_by_tags(
    tag_names: Vec<String>,
    page: u32,
    page_size: u32,
    work_directory: Option<String>
) -> Result<Vec<Note>, String>;
```

### 7.5 Database 实现

```rust
impl Database {
    /// 创建或获取标签（支持多级）
    pub fn create_or_get_tag(&self, name: &str) -> SqliteResult<Tag> {
        // 解析标签路径
        let parts: Vec<&str> = name.split('/').collect();
        let level = parts.len() as i32;
        let display_name = parts.last().unwrap_or(&name).to_string();
        let path = if parts.len() > 1 {
            parts[..parts.len()-1].join("/")
        } else {
            String::new()
        };

        // 检查是否已存在
        if let Some(tag) = self.get_tag_by_name(name)? {
            return Ok(tag);
        }

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

    /// 重命名标签
    pub fn rename_tag(
        &self,
        old_name: &str,
        new_name: &str,
        rename_children: bool
    ) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        // 更新当前标签
        let parts: Vec<&str> = new_name.split('/').collect();
        self.conn.execute(
            "UPDATE tags SET name = ?1, display_name = ?2, updated_at = ?3 WHERE name = ?4",
            params![new_name, parts.last().unwrap_or(&""), &now, old_name]
        )?;

        // 更新子标签
        if rename_children {
            let pattern = format!("{}%", old_name);
            for tag in self.get_tags_by_path_pattern(&pattern)? {
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
}
```

---

## 8. 前端组件实现

### 8.1 组件结构

```
src/
├── components/
│   ├── TagSidebar.vue       # 标签侧边栏主组件
│   ├── TagTreeNode.vue      # 标签树节点组件
│   └── TagRenameDialog.vue  # 标签重命名对话框
├── extensions/
│   └── tag-extension.js     # Tiptap 标签扩展
└── store/
    └── noteStore.js         # 添加标签相关方法
```

### 8.2 TagExtension 重构要点

1. **atom: true** - 标签作为不可编辑的原子单元
2. **空格键触发** - 检测空格键，将 `#标签` 转换为节点
3. **自动完成** - 使用 Tippy.js 渲染建议列表
4. **层级缩进** - 根据 level 属性添加缩进
5. **点击事件** - 派发 `tag-click` 自定义事件

### 8.3 noteStore 新增方法

```javascript
// 按多个标签筛选笔记
async function getNotesByTags(tagNames, page = 1, pageSize = 20)

// 创建或获取标签
async function createOrGetTag(tagPath)

// 重命名标签
async function renameTag(oldName, newName, renameChildren = false)

// 删除标签
async function deleteTag(tagName, deleteChildren = false)
```

---

## 9. 实施计划

### 9.1 后端实现

**任务列表**:
1. [ ] 更新 Tag 数据结构（移除 color 字段）
2. [ ] 实现 `create_or_get_tag` 方法
3. [ ] 实现 `rename_tag` 方法
4. [ ] 实现 `delete_tag_recursive` 方法
5. [ ] 实现 `get_notes_by_tags` 方法
6. [ ] 实现 `get_tags_by_path_pattern` 辅助方法
7. [ ] 更新 `update_note` 方法，从 HTML 提取标签
8. [ ] 注册所有新的 Tauri 命令

### 9.2 前端实现

**任务列表**:
1. [ ] 重构 TagExtension
   - [ ] 实现空格键触发转换
   - [ ] 实现自动完成建议列表
   - [ ] 实现层级缩进显示
   - [ ] 添加点击事件支持
2. [ ] 创建 TagSidebar 组件
   - [ ] 实现标签树加载
   - [ ] 实现搜索过滤
   - [ ] 实现展开/收起
3. [ ] 创建 TagTreeNode 组件
   - [ ] 实现递归渲染
   - [ ] 实现选择交互
4. [ ] 创建 TagRenameDialog 组件
5. [ ] 更新 noteStore
6. [ ] 更新 Note.vue 集成侧边栏

### 9.3 测试计划

1. **单元测试**: Database 方法的测试
2. **集成测试**: 前后端交互测试
3. **UI 测试**: 编辑器交互流程测试
4. **边界测试**: 特殊字符、极深层级等

### 9.4 部署注意事项

1. **数据库迁移**: 不需要，复用现有表结构
2. **兼容性**: 移除 color 字段，不需要数据迁移
3. **渐进式发布**: 可以先发布后端，再发布前端

---

## 附录

### A. 标签命名规则

- 支持字符: `a-zA-Z0-9_中文`
- 分隔符: `/`
- 长度限制: 单级标签名不超过 50 字符
- 层级限制: 理论上无限制，建议不超过 5 层

### B. 性能优化

1. **索引**: 为常用查询字段添加索引
2. **分页**: 标签列表和笔记列表都支持分页
3. **缓存**: 前端缓存标签树结构
4. **懒加载**: 子标签按需加载

### C. 用户快捷键

- `Cmd/Ctrl + Shift + T`: 切换标签侧边栏
- `Esc`: 关闭建议列表
- `Space`: 转换标签
- `Enter`: 确认选择
- `↑` `↓`: 导航建议列表

---

**文档版本**: 1.0
**最后更新**: 2025-02-04
