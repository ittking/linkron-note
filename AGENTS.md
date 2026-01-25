# iFlow CLI 项目上下文

## 项目概述

**项目名称：** iFlow CLI (iterm)

**项目类型：** Tauri + Vue 3 桌面应用程序

**技术栈：**
- **前端：** Vue 3 (Composition API with `<script setup>`) + Vite
- **后端：** Rust (Tauri 2)
- **构建工具：** Vite + Tauri CLI
- **包管理器：** pnpm

**项目架构：**
这是一个跨平台桌面应用程序，使用 Tauri 框架将 Vue 3 前端与 Rust 后端结合。前端运行在 Vite 开发服务器上，通过 Tauri 的 IPC 机制调用 Rust 函数。项目采用标准的 Tauri + Vue 3 模板结构。

**项目标识符：** `com.administrator.iterm`

## 项目结构

```
iterm/
├── src/                      # Vue 3 前端源代码
│   ├── App.vue              # 主应用组件（使用 <script setup>）
│   ├── main.js              # 应用入口文件
│   └── assets/              # 静态资源
├── src-tauri/               # Rust 后端和 Tauri 配置
│   ├── src/
│   │   ├── main.rs          # Rust 入口点
│   │   └── lib.rs           # Tauri 命令和运行逻辑
│   ├── Cargo.toml           # Rust 依赖配置
│   ├── tauri.conf.json      # Tauri 应用配置
│   ├── build.rs             # Rust 构建脚本
│   ├── capabilities/        # Tauri 能力配置
│   └── icons/               # 应用图标资源
├── public/                  # 公共静态资源
├── index.html               # HTML 入口文件
├── vite.config.js           # Vite 配置
└── package.json             # Node.js 依赖和脚本
```

## 构建和运行

### 开发模式

启动开发服务器（同时运行前端和后端）：
```bash
pnpm tauri dev
```

这会：
1. 启动 Vite 开发服务器（端口 1420）
2. 启动 Tauri 应用程序
3. 启用热模块替换（HMR）

### 生产构建

构建可分发的应用程序：
```bash
pnpm tauri build
```

这会：
1. 运行 `vite build` 构建前端
2. 编译 Rust 代码
3. 打包成可执行文件（根据平台）

### 仅前端开发

如果只需要开发前端（不涉及 Rust 代码）：
```bash
pnpm dev
```

### 预览生产构建

预览前端构建结果：
```bash
pnpm preview
```

## 开发约定

### Vue 3 前端

- **组件风格：** 使用 Vue 3 的 `<script setup>` 语法
- **状态管理：** 使用 Vue 3 的 `ref` 和 `reactive` API
- **与 Rust 通信：** 使用 `@tauri-apps/api/core` 的 `invoke` 函数调用 Rust 命令
- **样式：** 支持全局样式和 scoped 样式，包含暗色模式支持

### Rust 后端

- **命令定义：** 使用 `#[tauri::command]` 宏定义可被前端调用的函数
- **命令注册：** 在 `lib.rs` 中使用 `tauri::generate_handler!` 注册所有命令
- **序列化：** 使用 `serde` 和 `serde_json` 处理数据序列化
- **入口点：** `main.rs` 调用 `iterm_lib::run()` 启动应用

### Tauri 配置

- **开发端口：** 1420（固定端口，必须可用）
- **HMR 端口：** 1421
- **窗口大小：** 800x600（默认）
- **CSP：** 未设置（null）

### 依赖管理

**前端依赖：**
- `vue@^3.5.13` - Vue 3 框架
- `@tauri-apps/api@^2` - Tauri 前端 API
- `@tauri-apps/plugin-opener@^2` - Tauri 插件

**开发依赖：**
- `@vitejs/plugin-vue@^5.2.1` - Vite Vue 插件
- `vite@^6.0.3` - Vite 构建工具
- `@tauri-apps/cli@^2` - Tauri CLI

**Rust 依赖：**
- `tauri@^2` - Tauri 核心库
- `tauri-plugin-opener@^2` - Tauri 插件
- `serde@^1` - 序列化框架
- `serde_json@^1` - JSON 序列化

## Tauri 命令示例

当前项目包含一个示例命令：

**Rust 端（src-tauri/src/lib.rs:5）：**
```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
```

**Vue 端（src/App.vue:7）：**
```javascript
async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}
```

## 推荐开发环境

- **IDE：** VS Code
- **VS Code 扩展：**
  - Vue - Official (Vue.volar)
  - Tauri (tauri-apps.tauri-vscode)
  - rust-analyzer (rust-lang.rust-analyzer)

## 重要注意事项

1. **端口冲突：** 开发模式需要端口 1420 和 1421 可用，如被占用需修改 `vite.config.js`
2. **Rust 编译：** 首次构建可能需要较长时间，因为需要编译 Rust 依赖
3. **跨平台：** 项目支持 Windows、macOS 和 Linux，但当前在 Windows 环境下开发
4. **热重载：** 前端代码修改会自动热重载，Rust 代码修改需要重新编译
5. **文件监听：** Vite 配置为忽略 `src-tauri` 目录的监听，避免不必要的重载

## 相关资源

- [Tauri 文档](https://tauri.app/develop/)
- [Vue 3 文档](https://v3.vuejs.org/)
- [Vite 文档](https://vite.dev/)
- [Script Setup 语法](https://v3.vuejs.org/api/sfc-script-setup.html)