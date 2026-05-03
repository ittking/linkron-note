# Linkron Note

一个基于 Tauri 2 + Vue 3 的跨平台桌面生产力应用，集笔记、待办和设置于一体的极简笔记工具，随时随记，简约高效。

## 功能特性

### 📝 笔记管理
- 基于 Tiptap 的富文本编辑器，支持多种格式
- 灵活的标签分类，支持多级标签嵌套
- 标签树视图，快速定位笔记
- 标签删除时自动清理笔记中的标签引用

### ✅ 待办事项
- 任务管理功能，清晰追踪每项工作
- 日历视图，直观规划时间

### ☁️ 云同步
- 支持 Git 仓库同步（GitHub、Gitee 等）
- 可配置自动同步延迟
- 手动触发同步，同步状态实时显示

### ⚙️ 应用设置
- 主题配置（DaisyUI 主题系统）
- 工作目录设置
- 自动更新配置
- 偏好设置持久化存储

### 🔄 自动更新
- 内置自动更新检查
- 支持手动检查更新
- 可视化下载进度和安装状态

## 技术栈

- **前端**: Vue 3 + Vite + Tailwind CSS + DaisyUI
- **后端**: Tauri 2 + Rust
- **数据库**: SQLite
- **富文本**: Tiptap
- **状态管理**: Pinia
- **同步**: Git 协议

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- pnpm 或 npm

### 开发模式

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 构建生产版本

```bash
# 构建应用
npm run tauri build
```

## 项目结构

```
linkron-note/
├── src/                    # Vue 源码
│   ├── views/             # 页面组件 (Note, Todo, Setting)
│   ├── components/        # 可复用组件
│   │   ├── ui/           # UI 基础组件 (Button, Input 等)
│   │   └── TagTreeNode.vue / NoteSidebar.vue
│   ├── store/            # 状态管理 (Pinia Stores)
│   ├── composables/      # 组合式函数 (useSync, useConfirmDialog 等)
│   ├── router/           # Vue Router 配置
│   └── utils/            # 工具函数
├── src-tauri/            # Tauri (Rust) 后端
│   ├── src/
│   │   ├── lib.rs       # 主入口，注册 Tauri 命令
│   │   ├── database.rs  # 数据库操作
│   │   ├── tag.rs       # 标签相关逻辑
│   │   ├── note.rs      # 笔记相关逻辑
│   │   └── api_sync.rs  # 云同步功能
│   └── capabilities/     # 权限配置
└── public/               # 静态资源
```

## 核心设计

### 数据存储
- SQLite 本地数据库存储笔记、标签、待办事项
- 设置和偏好使用 Tauri Store 持久化
- 支持自定义工作目录

### 标签系统
- 支持多级标签嵌套（父/子标签）
- 删除标签时自动清理关联的笔记内容
- 标签置顶功能

### 云同步机制
- 基于 Git 协议的增量同步
- 可配置同步延迟防抖
- 同步状态和进度实时显示

## 开发指南

### 推荐的 IDE 设置

- [VS Code](https://code.visualstudio.com/)
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### 代码规范

- Vue 3 使用 `<script setup>` 语法
- 使用路径别名 `@` 映射到 `/src`
- 组件自动导入（无需显式导入 Vue、Vue Router、Tauri API）
- Rust 代码遵循标准 Rust 风格

### Tauri 通信模式

前端与后端通信使用 Tauri 的 `invoke` 模式：

```javascript
// 前端调用
const result = await invoke('command_name', { arg1, arg2 })

// 后端定义 (Rust)
#[tauri::command]
fn command_name(arg1: Type1, arg2: Type2) -> ResultType {
    // ...
}
```

## 部署

应用支持自动更新功能，更新配置通过 `tauri.conf.json` 管理。

## 许可证

MIT

## 链接

- [项目仓库 (GitHub)](https://github.com/ittking/linkron-note)
- [项目仓库 (GitCode)](https://gitcode.com/linkron/note)
- [Tauri 官方文档](https://tauri.app/)
- [Vue 3 文档](https://vuejs.org/)
