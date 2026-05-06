# ClipSnap — Windows 11 Clipboard Manager (Tauri 2 + React + Rust)

Build a fast, lightweight clipboard history manager for Windows 11, inspired by Alfred's clipboard viewer on macOS. Name: **ClipSnap**.

## Tech Stack (strict)

- **Tauri 2.x** (Rust backend, system webview frontend)
- **Frontend**: Vite + React 18 + TypeScript + Tailwind CSS v4
- **Database**: SQLite via `rusqlite` (bundled feature)
- **Global Hotkey**: `tauri-plugin-global-shortcut`
- **Clipboard Watcher**: `clipboard-rs` crate (real change events, NOT polling)
- **Auto-Paste**: `enigo` crate (simulates Ctrl+V after selection)
- **Fuzzy Search (frontend)**: `fuse.js`
- **List Virtualization**: `@tanstack/react-virtual`
- **Icons**: `lucide-react`
- **Package Manager**: pnpm

## Project Setup

Initialize with:
```bash
pnpm create tauri-app@latest clipsnap --template react-ts --manager pnpm
cd clipsnap
pnpm tauri add global-shortcut
pnpm tauri add clipboard-manager
pnpm tauri add sql --features sqlite
pnpm tauri add autostart
```

Add Rust deps to `src-tauri/Cargo.toml`:
```toml
clipboard-rs = "0.2"
enigo = "0.3"
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
image = "0.25"
base64 = "0.22"
```

## Features (scope)

### Core
1. **Global hotkey** `Ctrl+Shift+V` opens a centered popup window (600x500, frameless, transparent, always-on-top, `skip_taskbar`, `resizable: false`).
2. **Clipboard watcher** runs in a background thread, captures every change:
   - Text (plain)
   - RTF
   - HTML
   - Images (PNG, stored as base64 in SQLite — max 5MB per entry)
   - Files (list of paths)
3. **SQLite persistence** at `%APPDATA%\ClipSnap\history.db` (sensitive columns AES-256-GCM encrypted at rest since v0.6.0 — see [`docs/encryption.md`](./encryption.md)):
   ```sql
   CREATE TABLE IF NOT EXISTS entries (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     content_type TEXT NOT NULL,  -- 'text' | 'rtf' | 'html' | 'image' | 'files'
     content_text TEXT,            -- plain text preview (always populated for search)
     content_data BLOB,            -- raw payload (RTF/HTML string, PNG bytes, JSON array for files)
     hash TEXT NOT NULL UNIQUE,    -- SHA256 of payload, dedupe
     byte_size INTEGER NOT NULL,
     created_at INTEGER NOT NULL,  -- unix ms
     last_used_at INTEGER NOT NULL
   );
   CREATE INDEX idx_last_used ON entries(last_used_at DESC);
   CREATE INDEX idx_hash ON entries(hash);
   ```
   - Dedupe on hash: if exists, bump `last_used_at`, don't insert duplicate.
   - Cap history at **1000 entries**; prune oldest by `last_used_at` via trigger or on-insert cleanup.
4. **Fuzzy search** in the popup via `fuse.js` on `content_text`, threshold 0.4, searches as user types.
5. **Auto-paste**: after user picks an entry (Enter or click):
   - Write entry back to clipboard
   - Hide popup immediately
   - Wait 50ms for focus to return to previous window
   - Use `enigo` to send `Ctrl+V`
   - Update `last_used_at` in DB

### UX Behavior
- Popup opens **centered on the monitor where the cursor currently is**.
- Auto-focus on search input.
- `↑` / `↓` navigate list, `Enter` pastes selected, `Esc` or window blur closes popup.
- Clicking outside popup closes it (`on_focus_change`).
- **Do NOT show main window on app launch** — only tray icon.
- System tray menu: "Open (Ctrl+Shift+V)" • "Clear History" (with confirm) • "Pause Capture" (toggle) • "Start with Windows" (toggle via `tauri-plugin-autostart`) • "Quit".

### UI Design
Inspired by Alfred but clean, minimal, modern. Dark theme default with light-mode support (follows system).

**Layout** (popup):
```
┌─────────────────────────────────────────────┐
│  🔍  [search input, placeholder: "Search…"] │  ← 56px
├──────────────┬──────────────────────────────┤
│              │                              │
│  List        │   Preview Panel              │
│  (virtual,   │   (selected entry)           │
│   40%)       │   (60%)                      │
│              │                              │
│  • item 1    │   ┌──────────────────────┐   │
│  • item 2    │   │ Full content here    │   │
│  • item 3    │   │ or image or file list│   │
│  • …         │   └──────────────────────┘   │
│              │                              │
├──────────────┴──────────────────────────────┤
│  ⏎ Paste   ↑↓ Navigate   Esc Close   [1/42] │  ← 32px footer
└─────────────────────────────────────────────┘
```

**List item**: icon (type-based: T, 🖼️, 📄, </>, {RTF}) • preview (1 line, truncated) • timestamp (relative: "2m ago"). Selected row has accent background.

**Preview panel**: renders based on type:
- `text` → `<pre>` with wrap
- `html` → sandboxed `<iframe srcdoc>` (no script execution)
- `rtf` → plain-text extraction (strip RTF tags) + note "RTF formatting will be preserved on paste"
- `image` → `<img>` with max-height, show dimensions + size
- `files` → list of paths with file icons

**Colors** (Tailwind, define in CSS vars for easy theming):
- Background: `#0f0f10` / `#fafafa`
- Surface: `#1a1a1c` / `#ffffff`
- Border: `#2a2a2e` / `#e5e5e5`
- Accent: `#6366f1` (indigo-500)
- Text: `#f5f5f5` / `#18181b`

Use Tailwind v4 with `@theme` directive. Rounded corners (12px outer, 8px inner). Subtle shadow. Font: system stack (`-apple-system, Segoe UI, sans-serif`), monospace for text previews (`JetBrains Mono, Consolas, monospace`).

## File Structure

```
clipsnap/
├── src/                          # React frontend
│   ├── App.tsx                   # Popup root
│   ├── main.tsx
│   ├── styles.css                # Tailwind entry
│   ├── components/
│   │   ├── SearchBar.tsx
│   │   ├── HistoryList.tsx       # virtualized
│   │   ├── HistoryItem.tsx
│   │   ├── PreviewPanel.tsx
│   │   └── Footer.tsx
│   ├── hooks/
│   │   ├── useClipboardHistory.ts
│   │   ├── useKeyboardNav.ts
│   │   └── useFuzzySearch.ts
│   ├── lib/
│   │   ├── ipc.ts                # typed Tauri invoke wrappers
│   │   └── types.ts
│   └── assets/
└── src-tauri/
    ├── src/
    │   ├── main.rs               # entry, tray, window setup
    │   ├── clipboard_watcher.rs  # clipboard-rs listener loop
    │   ├── db.rs                 # rusqlite wrapper
    │   ├── hotkey.rs             # global shortcut registration
    │   ├── paste.rs              # enigo auto-paste
    │   ├── commands.rs           # #[tauri::command] handlers
    │   └── models.rs             # ClipEntry struct, ContentType enum
    ├── tauri.conf.json
    ├── Cargo.toml
    └── icons/
```

## Tauri Commands (IPC)

Expose these via `#[tauri::command]`:

```rust
get_history(limit: usize, offset: usize) -> Vec<ClipEntry>
search_history(query: String, limit: usize) -> Vec<ClipEntry>
paste_entry(id: i64) -> Result<(), String>   // writes to clipboard + triggers Ctrl+V
delete_entry(id: i64) -> Result<(), String>
clear_history() -> Result<(), String>
toggle_capture(paused: bool) -> Result<(), String>
get_capture_state() -> bool
```

## tauri.conf.json key settings

```json
{
  "app": {
    "windows": [{
      "label": "popup",
      "title": "ClipSnap",
      "width": 600,
      "height": 500,
      "center": true,
      "resizable": false,
      "decorations": false,
      "transparent": true,
      "alwaysOnTop": true,
      "skipTaskbar": true,
      "visible": false,
      "focus": true
    }],
    "trayIcon": {
      "id": "main",
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "targets": ["msi"],
    "windows": {
      "wix": {
        "language": "en-US"
      }
    }
  }
}
```

## Implementation Order

1. Scaffold project + install all deps.
2. Rust: `db.rs` — SQLite init, CRUD, dedupe logic, prune to 1000.
3. Rust: `clipboard_watcher.rs` — spawn thread with `clipboard-rs` `ClipboardHandler`, on change capture all formats, hash, insert via db.
4. Rust: `hotkey.rs` — register Ctrl+Shift+V, on trigger show + focus popup window (move to cursor's monitor).
5. Rust: `paste.rs` — write entry to clipboard, 50ms sleep, `enigo` `Ctrl+V`.
6. Rust: `commands.rs` — wire all IPC commands.
7. Rust: `main.rs` — build tray menu, hide popup on blur, register all plugins.
8. Frontend: Tailwind config + base styles + CSS vars.
9. Frontend: `ipc.ts` typed wrappers around `invoke`.
10. Frontend: `HistoryList` + `HistoryItem` + `PreviewPanel` + `SearchBar`.
11. Frontend: keyboard nav hook (↑↓ Enter Esc), fuzzy search hook.
12. Frontend: wire everything in `App.tsx`, listen for `window-shown` event from Rust to reset state on each open.
13. Polish: animations (popup fade-in via Tailwind `animate-in`), empty state, tray menu actions.
14. Build MSI: `pnpm tauri build`.

## Non-Goals (do NOT build)

- Cloud sync
- Multi-device
- ~~Encryption at rest (document limitation in README)~~ — **delivered in v0.6.0**, see [`docs/encryption.md`](./encryption.md)
- Categories/tags
- Pin/favorites (can be a v2)
- Sensitive-app detection

## Definition of Done

- `pnpm tauri dev` runs, tray icon visible, Ctrl+Shift+V opens popup.
- Copying text/image/file/HTML in any app appears in history within 200ms.
- Search filters live as I type.
- Enter on selected entry pastes into previous focused app.
- Closing and reopening app preserves history.
- `pnpm tauri build` produces a working `.msi` in `src-tauri/target/release/bundle/msi/`.
- README.md with install, usage, keybinds, data location, known limitations.

## Code Quality

- Rust: no `unwrap()` in production paths, use `anyhow::Result`. Structured logging via `tracing`.
- TypeScript: strict mode on, no `any`, ESLint + Prettier configured.
- Add a simple CI-free `scripts/check.sh` that runs `cargo clippy -- -D warnings` + `pnpm lint` + `pnpm tsc --noEmit`.
- Footer in README: `© 2026 Martin Pfeffer | celox.io`

Start now. Ask before deviating from this spec.
