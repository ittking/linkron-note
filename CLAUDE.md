# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**iterm** is a desktop productivity application built with Tauri 2 + Vue 3. It combines four tools in a single "capsule-style" window:

- **笔记** (Note) - Rich text editor with Tiptap
- **终端** (Terminal) - Integrated terminal using xterm.js
- **待办** (Todo) - Task management
- **设置** (Settings) - Application configuration

## Development Commands

### Frontend (Vue/Vite)
```bash
npm run dev      # Start development server on port 1420
npm run build    # Build for production
npm run preview  # Preview built application
```

### Backend (Rust/Tauri)
```bash
npm run tauri dev     # Run Tauri development mode
npm run tauri build   # Build desktop application
npm run tauri info    # Show Tauri environment info
```

### Rust-specific (in src-tauri/)
```bash
cargo build           # Build Rust code
cargo test            # Run Rust tests
cargo clippy          # Lint Rust code
```

## Architecture

### Frontend Structure

```
src/
├── views/           # Page components (Note.vue, Terminal.vue, Todo.vue, Setting.vue)
├── components/      # Reusable Vue components
├── store/           # Pinia stores for state management
├── api/             # API layer (HTTP client)
├── router/          # Vue Router configuration
├── utils/           # Utility functions
└── Main.vue         # Main window wrapper with tabbed interface
```

**Key conventions:**
- Vue 3 `<script setup>` syntax throughout
- Path alias `@` maps to `/src`
- Components use auto-import (no explicit imports needed for Vue, Vue Router, Tauri APIs)
- DaisyUI + Tailwind CSS 4.x for styling
- All pages wrapped in `Main.vue` which provides the tabbed navigation

### Backend Structure (Rust)

```
src-tauri/src/
├── lib.rs              # Main entry point, registers all Tauri commands
├── database.rs         # SQLite operations for notes/tags
├── terminal.rs         # PTY session management for integrated terminal
├── filesystem.rs       # File operations (save, read, delete images/files)
├── protocol.rs         # Custom `iterm://` URI scheme handler
├── window_manager.rs   # Window state (capsule mode, show/hide, OS-specific)
├── mouse.rs            # Mouse listener for auto-hide functionality
├── autostart.rs        # OS autostart configuration
├── file_reader.rs      # Text file reading utilities
└── web_scraper.rs      # Web content extraction
```

**Tauri commands are registered in `lib.rs`** - any new command must be added to the `invoke_handler!` macro.

### State Management (Pinia Stores)

- **noteStore.js** - Note CRUD operations, tag management, search
- **terminalStore.js** - Terminal sessions, PTY connections
- **settingStore.js** - App settings persisted to Tauri Store
- **configStore.js** - Runtime configuration

All database operations go through Tauri commands (`invoke('database_command')`).

### Communication Pattern

Frontend ↔ Backend communication follows Tauri's `invoke` pattern:

```javascript
// Frontend (auto-imported)
const result = await invoke('command_name', { arg1, arg2 })

// Backend (Rust)
#[tauri::command]
fn command_name(arg1: Type1, arg2: Type2) -> ResultType {
    // ...
}
```

## Key Technical Details

### Window Behavior
- Fixed port: **1420** (required by Tauri, will fail if occupied)
- Transparent, frameless window with custom controls
- "Capsule mode": minimized to small capsule when inactive
- Window position/size persisted across sessions
- Uses macOS private APIs for window management

### Tiptap Editor Extensions
- Code blocks with syntax highlighting (lowlight)
- Image support with inline display
- Link handling
- Mention support
- Text styling (highlight, underline)
- Placeholder text

### Terminal Integration
- `xterm.js` for rendering
- `xterm-addon-fit` for auto-sizing
- PTY sessions managed in Rust via `portable-pty`
- Each tab maintains separate PTY connection
- Shell detection and configuration via settings

### Database (SQLite)
- Notes with rich text content
- Tags with note associations
- Full-text search on notes
- JSON migration support for legacy data

### Custom URI Scheme
- `iterm://` protocol for internal resource references
- Used for embedded images and file links
- Handled by `protocol.rs`

## Build Configuration

- **Frontend dist**: `../dist` (relative to src-tauri)
- **Before dev**: `pnpm dev` (starts Vite)
- **Before build**: `pnpm build` (builds Vue)
- **Bundle targets**: All platforms (determined by Tauri)

## Platform-Specific Code

- macOS: Cocoa/AppKit window management via `cocoa` and `objc2` crates
- Windows: Win32 APIs via `windows` crate, registry via `winreg`
- Linux: Standard X11/Wayland support via Tauri

Conditional compilation uses `#[cfg(target_os = "...")]` in Rust.
