# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Install all workspace dependencies (run once after clone)
pnpm install

# Dev servers (hot-reload Rust + frontend)
pnpm dev:win          # Windows
pnpm dev:macos        # macOS

# Production builds
pnpm build:win        # → target/release/bundle/msi/*.msi + target/release/clipsnap.exe
pnpm build:macos      # → target/release/bundle/dmg/*.dmg

# Tests
pnpm test                                     # frontend vitest (all, single run)
pnpm --filter clipsnap-frontend test:watch    # frontend vitest watch mode
cargo test --workspace                        # all Rust unit tests

# Static analysis (clippy + tsc + eslint in one shot)
pnpm check            # or: bash scripts/check.sh

# Individual checks
pnpm typecheck        # tsc --noEmit
pnpm lint             # eslint src
cargo clippy --workspace --all-targets -- -D warnings
```

Running Rust tests locally on Linux requires system libs:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libxdo-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

## Architecture

### Workspace layout

```
core/rust-lib/   — clipsnap-core rlib: ALL business logic (DB, clipboard, hotkey, paste, snippets, notes, settings, backup, expander)
core/frontend/   — React 19 + TS + Tailwind v4 + Vite 7 (shared by all platforms)
win/src-tauri/   — Windows bundle shell: 2-line main.rs + Tauri config + capabilities
macos/src-tauri/ — macOS bundle shell: 2-line main.rs + Tauri config + capabilities
```

Both platform shells contain only `clipsnap_core::run(tauri::generate_context!())`. All logic is in `core/rust-lib`. The Tauri CLI is invoked per platform via `pnpm --filter clipsnap-{win,macos} tauri {dev,build}`.

### Adding a new IPC command (end-to-end)

1. Implement logic in the relevant `core/rust-lib/src/*.rs` module.
2. Add a `#[tauri::command]` wrapper in `core/rust-lib/src/commands.rs`.
3. Register it in the `invoke_handler![]` macro in `core/rust-lib/src/lib.rs`.
4. Add a typed `invoke("command_name", { ...args })` wrapper in `core/frontend/src/lib/ipc.ts`.

### Database — four tables in one SQLite file

`DbHandle = Arc<Mutex<Connection>>` (rusqlite + parking_lot). Managed as Tauri state. File location:
- Windows: `%APPDATA%\ClipSnap\history.db`
- macOS: `~/Library/Application Support/ClipSnap/history.db`

| Table | Purpose | Notes |
|---|---|---|
| `entries` | Clipboard history | SHA-256 deduped; capped at 1 000 rows via `prune_locked`; sorted by `last_used_at DESC` |
| `snippets` | Text expander templates | `abbreviation` + `title` + `body`; index on `abbreviation` |
| `notes` | Persistent bookmarks | Not pruned; `title` + `category`; any clipboard entry can be saved here |
| `settings` | Key/value app settings | Simple `key TEXT PK, value TEXT`; used for expander hotkey & enabled flag |

Rust unit tests use `Connection::open_in_memory()` — no temp files needed.

### Frontend data flow and `ListEntry` union

The history tab renders a unified `ListEntry` discriminated union:

```ts
type ListEntry =
  | { kind: "clip";    data: ClipEntry }
  | { kind: "snippet"; data: Snippet }
  | { kind: "calc";    data: { display: string; ... } }
```

Assembly order in `App.tsx`: calc result first → snippet matches → fuzzy clips. Snippet matches come from `findSnippets(query)` (backend prefix/contains SQL). The inline calculator (`core/frontend/src/lib/calc.ts`) runs `tryEvaluate(query)` — returns non-null only when the input contains an operator, function, or constant (plain numbers/text are ignored).

### Tabs

`App.tsx` manages `activeTab: "history" | "snippets" | "notes" | "settings"`. Each tab is a separate panel component:

| Tab | Component | Backing data |
|---|---|---|
| History | `HistoryList` + `PreviewPanel` | `useClipboardHistory` + `useFuzzySearch` |
| Snippets | `SnippetsPanel` | `useSnippets` |
| Notes | `NotesPanel` | `useNotes` |
| Settings | `SettingsPanel` | IPC to `settings.rs` + `expander.rs` |

### Tauri events

| Rust `app.emit(...)` | Purpose |
|---|---|
| `"clipboard-changed"` | Triggers history re-fetch in `useClipboardHistory` |
| `"window-shown"` | Resets to history tab + focuses search on hotkey press |
| `"open-snippets-tab"` | Tray "Manage Snippets" → switches tab |
| `"open-notes-tab"` | Tray "Manage Notes" → switches tab |
| `"ocr-permission-needed"` | OCR hotkey pressed but Screen Recording not granted → frontend banner + Settings tab |
| `"expander-permission-needed"` | Expander hotkey pressed but Accessibility not granted → frontend banner + Settings tab |

### Text expander (`expander.rs`)

Three expansion modes exist:

1. **Search-based** (always on): type an abbreviation in the search field → matching snippets appear at top of list → Enter pastes. Handled entirely in the frontend via `findSnippets()`.

2. **Abbreviation hotkey** (`expander.rs`, default hotkey `Alt+Digit1` — shown as `Alt+1`): fires from any app without opening the popup. Three paths via `text_field::FieldAccess::try_replace_word_before_cursor` → `ReplaceOutcome`:
   - **`Replaced`** — AX/UIA read the word + replaced it in place; on macOS this is verified by re-reading `AXValue`. No clipboard touch.
   - **`SelectionActive`** — AX *selected* the abbreviation but the in-place text set was a no-op (Electron / Chromium / Mac-Catalyst: WhatsApp, Slack, Discord, VS Code, …). `expander::paste_over_selection` pastes the body over the live selection (one clipboard write + paste + restore, **no** re-select).
   - **`Unsupported`** — the focused element exposes no settable text attributes → legacy cycle: save clipboard → `Opt/Ctrl+Shift+←` selects previous word → copy → look up → paste body → restore clipboard.
   Enabled/disabled + hotkey configurable in Settings tab (with `Alt+1`/`Alt+2`/`Alt+3` quick-pick presets). Pre-0.12 the default was `Alt+Backquote`, unreachable on German ISO Macs — `expander::migrate_legacy_default` bumps an un-customised install to `Alt+Digit1` once (idempotent). **Terminals are unsupported by this mode** (no AX-exposed input line, no GUI word-select on a shell prompt) — pressing the hotkey there does nothing.

3. **Direct hotkey → snippet slots** (`expander.rs` + `hotkey::register_direct_slots`, v0.13.0): bind a hotkey straight to a snippet — `expander::DirectSlot { hotkey, snippet_id }`, persisted as a JSON array under settings key `expander.direct_slots`. On press: `expander::paste_snippet_body` (AX-gated on macOS, runs on main thread) → write body to clipboard → synthesize `Cmd/Ctrl+V` → restore clipboard. Reads nothing, so it works **everywhere including terminals**. `register_direct_slots` validates against collisions with the popup/OCR/abbreviation hotkeys + duplicates. `ExpanderShortcutState.direct: Vec<(Shortcut, i64)>`. IPC: `get_direct_slots` / `set_direct_slots`. Re-registered at startup from settings. Settings UI: "Direct hotkey → snippet" section (rows of `[HotkeyCapture] → [snippet <select>] [×]` + Add + Save).

On macOS, if Accessibility isn't granted the hotkey handler short-circuits *before* the doomed cycle: `expand_at_cursor` returns the `expander::ERR_NO_ACCESSIBILITY` (`"ax.permission_denied"`) sentinel, and `hotkey::register_expander`'s callback pre-checks `accessibility_granted()` → on a miss it shows the popup + emits `"expander-permission-needed"` (frontend turns it into an amber banner). Mirrors the OCR `screen.permission_denied` path.

The Settings panel includes a **"Diagnose"** button that calls `diagnose_at_cursor` — runs the capture half (no paste) and returns what would have been matched (or, on macOS without Accessibility, an explanatory error), for debugging.

### Screen-region OCR (`region_picker.rs`, `ocr.rs`)

Triggered by `Cmd/Ctrl+Shift+O` (registered alongside the popup hotkey in `hotkey::register`) or the tray's **OCR Region** menu. Pipeline lives in `commands::run_ocr_pipeline(app)`, shared between the IPC `ocr_region` command, the global-shortcut callback, and the tray handler. Always dispatched to a worker thread (`std::thread::spawn`) because `screencapture -i` blocks until the user finishes the marquee.

- **Region capture** (macOS) shells out to `/usr/sbin/screencapture -i -x -t png <tmpfile>`. Read the file back, delete it. Empty / missing file = user pressed Esc → return `region_picker::Cancelled`. Windows path is stubbed.
- **OCR** (macOS) uses Vision via raw `objc2` msg_send: `NSData::dataWithBytes:length:` → `VNImageRequestHandler.alloc().initWithData:options:` → `VNRecognizeTextRequest` (recognitionLevel=0/Accurate, usesLanguageCorrection=true) → `performRequests:error:` synchronously → enumerate `request.results` taking `topCandidates(1).string`. Vision is linked explicitly via `core/rust-lib/build.rs` (`cargo:rustc-link-lib=framework=Vision`).
- **Output**: text written to system clipboard (with `WatcherState::mark_self_write` so the watcher doesn't recapture it), plus two history entries — the recognised text and the source PNG. Returns `OcrResult { text, cancelled, chars }` so the frontend can show "recognised N chars" toasts.

### Image tools (`recolor.rs`, `cutout_ml.rs`)

Two image actions surface in the preview pane:

- **Recolor** (`recolor.rs`) — `image::load_from_memory` → for each RGBA pixel, replace RGB with `lerp(target, white, luminance)` (alpha untouched) → re-encode → `db::upsert_clip` as a new history row. Eligibility gate: `image_chromaticity` samples up to 4096 opaque pixels (`max((max-min)/max)`). Toolbar only shown when chromaticity < 0.12 (logos / silhouettes).
- **Cut-out background** (`cutout_ml.rs`) — runs U2Netp via the `ort` crate (ONNX Runtime). Decode → resize to 320×320 → ImageNet-normalise (mean `[0.485, 0.456, 0.406]`, std `[0.229, 0.224, 0.225]`) → inference → resize mask back → apply as alpha on the original RGB → encode PNG. Output to `~/Downloads/<name>-cutout-<ts>.png`. Triggered by button in PreviewPanel or `Cmd/Ctrl+B`. Works on real photos (subject/background colour overlap is no longer fatal).
  - Model file: `core/rust-lib/models/u2netp.onnx` (~4.5 MB, embedded via `include_bytes!`).
  - ONNX Runtime is statically linked via `ort`'s `download-binaries` feature → release binary ~40 MB.
  - Session is held in `OnceLock<Mutex<Session>>` so the first cutout pays the model-load cost (~150 ms) once and subsequent calls reuse it.
- **Cutout source variants** — IPC has both `cut_out_image_entry(id)` (clipboard image rows) and `cut_out_image_file(path)` (single-file Files entries pointing at PNG/JPG/WebP/GIF/BMP). Same `cutout_ml::cut_out_subject` underneath via `commands::write_cutout`.
- The legacy chroma-key (`cutout.rs`) stays in the tree under `#![allow(dead_code)]` as a future fast-path for true-uniform-background images.

Both modules share the 16 MP hard cap and the multi-format `image` 0.25 dependency (PNG / JPEG / WebP / GIF / BMP).

### Clipboard capture priority

`clipboard_watcher::capture` checks formats in this order: **image → files → html → rtf → text**. Image-before-files matters on macOS, where copying a PNG/JPG/HEIC from Finder puts both the bitmap and the file path on the pasteboard — capturing as Files first meant the user only saw paths in history.

### `UiState` and modal focus

`UiState.suppress_hide` (AtomicBool, Tauri state) prevents the popup's "hide on focus-loss" handler from firing while a native file dialog is open. The frontend toggles it via `set_suppress_hide` before/after calling `tauri-plugin-dialog` commands (`dialog:allow-open`, `dialog:allow-save`).

### Platform-specific behaviour in shared code

- **Paste shortcut** (`paste.rs`): `Key::Meta` (Cmd+V) on macOS; `Key::Control` (Ctrl+V) elsewhere.
- **Focus-settle delay** (`paste.rs`): 120 ms on macOS, 50 ms on Windows/Linux.
- **Word-select modifier** (`expander.rs`): `Key::Alt` (Option) on macOS; `Key::Control` elsewhere.
- **Accessibility check** (`expander.rs`): `AXIsProcessTrusted()` via direct CoreFoundation FFI on macOS; always `true` on other platforms.
- **Dock visibility** (`lib.rs`): `set_activation_policy(Accessory)` on macOS.
- **Autostart tray label** (`lib.rs`): `cfg!(target_os = "windows")` → "Start with Windows", else "Start at Login". As of v0.14.0 it's a `CheckMenuItem` reflecting the current state (probed from `app.autolaunch().is_enabled()` on tray build); toggling updates the check + emits `autostart-changed`. IPC: `get_autostart_enabled` / `set_autostart_enabled`. Settings → Startup mirrors the toggle.

### macOS notes

`macos/src-tauri/Cargo.toml` requires `tauri = { features = ["macos-private-api"] }` for transparent windows. The `entitlements.plist` intentionally **omits** `com.apple.security.automation.apple-events` — `enigo`'s `CGEventPost` is gated by the TCC Accessibility permission (System Settings → Privacy → Accessibility), not that entitlement. The first paste or expander use triggers the Accessibility prompt. After granting, a relaunch is required (macOS caches `AXIsProcessTrusted` per process). The Settings panel detects the just-granted state and offers a one-click relaunch.

### Backup

`backup.rs` serialises history + snippets + notes into a single versioned JSON document. Import merges: snippets upsert by abbreviation, history upserts by hash (dedup), notes append verbatim (no dedup key → re-import creates duplicates). `CURRENT_VERSION = 1`; importing a higher version is rejected.
