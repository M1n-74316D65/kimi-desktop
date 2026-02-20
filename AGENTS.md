# AGENTS.md - Kimi Desktop App

This document provides guidance for AI agents working on this codebase.

## Project Overview

Kimi Desktop is a Tauri v2 desktop application that wraps Moonshot AI's Kimi interface (https://kimi.com) with a native quick launcher feature. Built with Rust backend (Tauri) and vanilla HTML/CSS/JS frontend.

## Tech Stack

- **Desktop Framework**: Tauri v2
- **Backend**: Rust (2021 edition)
- **Frontend**: Vanilla HTML, CSS, JavaScript (ES modules)
- **Package Manager**: Bun
- **Build System**: Tauri CLI via Bun scripts

## Build & Development Commands

```bash
bun install          # Install dependencies
bun run dev          # Run in development mode (hot-reload)
bun run build        # Build for production (current platform)
bun run tauri <cmd>  # Run Tauri CLI directly
```

### Rust-specific Commands

```bash
cd src-tauri && cargo check        # Check Rust code compiles
cd src-tauri && cargo build        # Build Rust backend only
cd src-tauri && cargo clippy       # Run clippy linter
cd src-tauri && cargo fmt          # Format Rust code
cd src-tauri && cargo test         # Run all Rust tests
cd src-tauri && cargo test <name>  # Run a single test by name or pattern
```

## Project Structure

```
kimi-app/
├── src/                      # Frontend files
│   ├── launcher.html         # Quick launcher UI
│   ├── launcher.css          # Launcher styles (CSS variables)
│   ├── launcher.js           # Launcher logic (ES modules)
│   ├── settings.html         # Settings window UI
│   ├── settings.js           # Settings logic
│   ├── settings.css          # Settings styles
│   └── index.html            # Offline/fallback page
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── lib.rs            # Main app logic, Tauri commands
│   │   └── main.rs           # Entry point
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # Tauri configuration
│   └── capabilities/         # Permission configs
├── package.json              # Frontend dependencies
└── bun.lock                  # Bun lockfile
```

## Code Style Guidelines

### Rust (src-tauri/)

**Imports**: Group and order: 1) std library, 2) external crates, 3) internal.

```rust
use tauri::{
    AppHandle, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Listener,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
```

**Error Handling**: Use `Result<T, E>` with `.map_err(|e| e.to_string())?` for Tauri commands. Log errors with `eprintln!` in setup functions.

```rust
#[tauri::command]
async fn my_command(app: AppHandle) -> Result<(), String> {
    something().map_err(|e| e.to_string())?;
    Ok(())
}
```

**Naming Conventions**:
- Functions/variables: `snake_case`
- Types/Structs: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Tauri commands: `snake_case` matching the JS invoke name

**Tauri Commands**: Always use `#[tauri::command]` attribute. Async commands should take `AppHandle` as first parameter. Return `Result<T, String>`.

**Window Management Pattern**:
```rust
if let Some(window) = app.get_webview_window("window_label") {
    let _ = window.show();
    let _ = window.set_focus();
}
```

**Struct Serialization**: Use `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]` for settings/data structures.

### JavaScript (src/)

**Imports**: Use ES module imports via `window.__TAURI__`:
```javascript
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
```

**Class Pattern**: Use ES6 classes to encapsulate functionality (see `LauncherApp`, `SettingsApp`).

**Naming**: Variables/functions use `camelCase`. CSS classes use `kebab-case` with component prefixes (`launcher-input`).

**Error Handling**: Wrap Tauri invocations in try-catch:
```javascript
try {
    await invoke('command_name', { param: value });
} catch (error) {
    console.error('Failed to...:', error);
}
```

**Constants**: Use `SCREAMING_SNAKE_CASE` for constants (e.g., `MAX_MESSAGE_LENGTH`).

### CSS (src/)

- Define colors/sizes in `:root` using CSS custom properties
- Support dark mode with `@media (prefers-color-scheme: dark)`
- Use `launcher-` prefix for all launcher-related classes

### HTML

- Semantic HTML5 with `aria-label` for interactive elements
- Use `type="module"` for script tags
- Set `autofocus`, `autocomplete`, `spellcheck` attributes appropriately

## Adding a New Tauri Command

1. Define in `src-tauri/src/lib.rs`:
```rust
#[tauri::command]
async fn new_command(app: AppHandle, param: String) -> Result<(), String> {
    Ok(())
}
```

2. Register in `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![existing_commands, new_command])
```

3. Call from JavaScript:
```javascript
await invoke('new_command', { param: 'value' });
```

## Key Dependencies

### Rust (Cargo.toml)
- `tauri@2` - Desktop app framework
- `tauri-plugin-global-shortcut@2` - Hotkey support
- `tauri-plugin-single-instance@2` - Prevent multiple instances
- `tauri-plugin-notification@2` - System notifications
- `tauri-plugin-store@2` - Settings persistence
- `tokio@1` - Async runtime

### JavaScript (package.json)
- `@tauri-apps/api@^2` - Tauri JavaScript API
- `@tauri-apps/cli@^2` - Build tooling

## Testing

Rust tests are located in `src-tauri/src/lib.rs` under `#[cfg(test)]` module. Run with:
```bash
cd src-tauri && cargo test
```

To run a specific test by name:
```bash
cd src-tauri && cargo test test_app_settings_default
cd src-tauri && cargo test test_inject_message  # Runs all tests matching pattern
```
