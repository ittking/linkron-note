# Linkron Note

一个基于 Tauri 2 + Vue 3 的桌面生产力应用，将笔记、终端、待办和设置集成在一个极简的胶囊窗口中。

## 功能特性

- **笔记** - 基于 Tiptap 的富文本编辑器
- **终端** - 集成终端 (xterm.js)
- **待办** - 任务管理
- **设置** - 应用配置

## 技术栈

- **前端**: Vue 3 + Vite + Tailwind CSS + DaisyUI
- **后端**: Tauri 2 + Rust
- **数据库**: SQLite
- **富文本**: Tiptap
- **终端**: xterm.js

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- pnpm

### 开发模式

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev
```

### 构建

```bash
# 构建应用
pnpm tauri build
```

## 项目结构

```
linkron-note/
├── src/                    # Vue 源码
│   ├── views/             # 页面组件
│   ├── components/        # 可复用组件
│   ├── store/            # Pinia 状态管理
│   └── api/              # API 层
├── src-tauri/            # Tauri (Rust) 后端
│   ├── src/              # Rust 源码
│   └── capabilities/     # 权限配置
└── public/               # 静态资源
```

## 自动更新

应用支持自动更新功能，详见 [部署文档](./DEPLOYMENT.md)。

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

## Tauri 命令

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

详细的构建和部署说明请参阅 [DEPLOYMENT.md](./DEPLOYMENT.md)。

## 许可证

MIT

## 链接

- [Tauri 官方文档](https://tauri.app/)
- [Vue 3 文档](https://vuejs.org/)
- [项目仓库](https://github.com/ittking/linkron-note)
