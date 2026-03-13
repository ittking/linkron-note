# iFlow CLI (linkron) - Agent Development Guide

## Project Overview

**Name:** linkron (iterm) - Tauri + Vue 3 Desktop App

**Stack:**
- Frontend: Vue 3 (Composition API with `<script setup>`) + Vite + Tailwind CSS 4 + DaisyUI
- Backend: Rust (Tauri 2) + SQLite
- Editor: TipTap rich text editor
- Icons: Lucide Vue Next

**Package Manager:** pnpm

## Commands

```bash
pnpm tauri dev              # Start dev server (frontend:1420, hmr:1421)
pnpm dev                    # Frontend only (no Rust recompilation)
pnpm build                  # Build frontend
pnpm preview                # Preview production build
pnpm tauri build            # Build full distributable app

cargo build                 # Build Rust backend
cargo test                  # Run Rust tests (if any exist)
cargo clippy                # Run Rust linter
cargo fmt                   # Format Rust code
```

**Note:** No test framework configured - tests must be added first.

## Code Style Guidelines

### Vue 3 Frontend

**Component Structure:**
```vue
<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  modelValue: { type: String, default: '' }
})

const emit = defineEmits(['update:modelValue', 'submit'])
const value = ref('')

onMounted(() => {
  // initialization logic
})
</script>

<template>
  <div class="p-4">
    <!-- content -->
  </div>
</template>
```

**Imports & Auto-Import:**
- Vue APIs (`ref`, `computed`, `watch`, etc.) are auto-imported via unplugin-auto-import
- Tauri APIs (`invoke`, `listen`) are also auto-imported
- Still explicitly import: composables, utilities, components, libraries

**Naming Conventions:**
- Components: PascalCase (`NoteEditor.vue`, `TodoDialog.vue`)
- Composables: camelCase with `use` prefix (`useWorkDirectory`, `useReminder`)
- Props: camelCase
- CSS classes: kebab-case, Tailwind utility classes preferred
- Stores: camelCase with `Store` suffix (`settingStore`)

**State Management:**
- Use `ref` for primitives, `reactive` for objects
- Use Pinia stores for global state (see `store/settingStore.js`)
- Destructure with `.value` when needed

**Styling:**
- Use Tailwind CSS utility classes
- DaisyUI components for UI primitives
- Custom styles in scoped `<style>` only when necessary

**Error Handling:**
- Wrap Tauri invoke calls in try-catch
- Provide user-friendly error messages
- Show loading states for async operations

### Rust Backend

**Command Definition:**
```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    Ok("success".to_string())
}
```

**Module Structure:**
- Each feature in its own module file (`note.rs`, `todo.rs`, `tag.rs`, etc.)
- Use `mod` declarations in `lib.rs`
- Re-export types with `pub use` for cleaner API

**Naming Conventions:**
- Functions: snake_case
- Types/Structs: PascalCase
- Constants: SCREAMING_SNAKE_CASE
- Module files: snake_case

**Error Handling:**
- Use `Result<T, String>` for Tauri commands
- Use `SqliteResult<T>` for database operations
- Use `?` operator for error propagation

**Database:**
- Use rusqlite for SQLite operations
- Always parameterize queries (prevent SQL injection)
- Create indexes on frequently queried columns
- Use transactions for multiple related operations

**Serialization:**
- Use `#[derive(Serialize, Deserialize)]` for structs passed to frontend
- Use `#[serde(rename = "fieldName")]` for camelCase/kebabCase mapping

### File Organization

**Frontend:** `src/components/` (reusable, ui/), `src/views/` (pages), `src/composables/`, `src/store/`, `src/utils/`, `src/extensions/` (TipTap), `src/router/`

**Backend:** `src-tauri/src/lib.rs` (Tauri builder, command registration), `src-tauri/src/main.rs` (entry), `src-tauri/src/database.rs`, module files per feature

## Key Patterns

### Calling Rust from Vue
```javascript
const result = await invoke("command_name", { param: value })
```

### Registering New Tauri Commands
1. Define command in Rust with `#[tauri::command]`
2. Add to `tauri::generate_handler![]` macro in `lib.rs`

### Adding New Dependencies
- Frontend: `pnpm add <package>`
- Rust: Add to `[dependencies]` in `src-tauri/Cargo.toml`

## Development Notes

- Dev ports: 1420 (Vite), 1421 (HMR) - must be available
- Vite ignores `src-tauri/` directory watching
- Rust changes require full recompile
- Frontend changes hot-reload automatically
- No configured test framework - add Vitest/Jest if needed
