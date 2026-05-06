<div align="center">
  <img src="docs/clipsnap.png" alt="ClipSnap" width="900" />

  # ClipSnap

  **Fast, lightweight clipboard history manager + text expander for Windows 11 & macOS**

  [![Version](https://img.shields.io/badge/version-0.6.0-blue?style=flat-square)](https://github.com/pepperonas/clipsnap/releases)
  [![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](./LICENSE)
  [![Windows 11](https://img.shields.io/badge/Windows-11-0078D4?style=flat-square&logo=windows11&logoColor=white)](./win)
  [![macOS](https://img.shields.io/badge/macOS-10.15+-000000?style=flat-square&logo=apple&logoColor=white)](./macos)
  [![Apple Silicon](https://img.shields.io/badge/arm64-Apple%20Silicon-555555?style=flat-square&logo=apple&logoColor=white)](./macos)
  [![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
  [![Rust](https://img.shields.io/badge/Rust-stable-CE422B?style=flat-square&logo=rust&logoColor=white)](https://rustup.rs)
  [![React 19](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=black)](https://react.dev)
  [![TypeScript 5](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org)
  [![Vite 7](https://img.shields.io/badge/Vite-7-646CFF?style=flat-square&logo=vite&logoColor=white)](https://vitejs.dev)
  [![Tailwind CSS v4](https://img.shields.io/badge/Tailwind-v4-38BDF8?style=flat-square&logo=tailwindcss&logoColor=white)](https://tailwindcss.com)
  [![pnpm](https://img.shields.io/badge/pnpm-10-F69220?style=flat-square&logo=pnpm&logoColor=white)](https://pnpm.io)
  [![SQLite](https://img.shields.io/badge/SQLite-bundled-003B57?style=flat-square&logo=sqlite&logoColor=white)](https://sqlite.org)
  [![ESLint](https://img.shields.io/badge/ESLint-flat%20config-4B32C3?style=flat-square&logo=eslint&logoColor=white)](https://eslint.org)
  [![Vitest](https://img.shields.io/badge/Vitest-3-6E9F18?style=flat-square&logo=vitest&logoColor=white)](https://vitest.dev)
  [![cargo test](https://img.shields.io/badge/cargo%20test-87%20passing-success?style=flat-square&logo=rust&logoColor=white)](#)
  [![Issues](https://img.shields.io/github/issues/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/issues)
  [![Stars](https://img.shields.io/github/stars/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/stargazers)
  [![Last commit](https://img.shields.io/github/last-commit/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/commits/main)
  [![Repo size](https://img.shields.io/github/repo-size/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap)
  [![Code size](https://img.shields.io/github/languages/code-size/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap)
  [![Top language](https://img.shields.io/github/languages/top/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap)
  [![CI](https://img.shields.io/github/actions/workflow/status/pepperonas/clipsnap/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/pepperonas/clipsnap/actions/workflows/ci.yml)
  [![Latest Release](https://img.shields.io/github/v/release/pepperonas/clipsnap?style=flat-square&label=download)](https://github.com/pepperonas/clipsnap/releases/latest)

  Press `Ctrl+Shift+V` — search — paste. Inspired by Alfred's clipboard viewer on macOS.
</div>

---

## Download

**Latest release:** [![Latest Release](https://img.shields.io/github/v/release/pepperonas/clipsnap?style=flat-square&label=latest&color=green)](https://github.com/pepperonas/clipsnap/releases/latest) — see the [CHANGELOG](./CHANGELOG.md) for what's new.

| Platform | File | Notes |
|----------|------|-------|
| **Windows 11 / 10** | [`ClipSnap_<ver>_x64_en-US.msi`](https://github.com/pepperonas/clipsnap/releases/latest) | MSI installer — adds Start-menu entry & uninstaller |
| **Windows 11 / 10** | [`clipsnap.exe`](https://github.com/pepperonas/clipsnap/releases/latest) | Standalone exe — no install needed |
| **macOS 10.15+ (Apple Silicon)** | [`ClipSnap_<ver>_aarch64.dmg`](https://github.com/pepperonas/clipsnap/releases/latest) | DMG for arm64 Macs |
| **macOS Intel** | — | Build from source: [`macos/README.md`](./macos/README.md) |
| **Linux** | — | Planned for a later release |

> **macOS Gatekeeper note.** Local-build releases are **not Apple-signed**. On first launch macOS will refuse to open the app — right-click → **Open** → confirm, or **System Settings → Privacy & Security → "Open Anyway"**. Then grant **Accessibility** access (for paste). Full setup in [`macos/README.md`](./macos/README.md).

---

## Platform support

| Platform   | Status                  | Location                |
|------------|-------------------------|-------------------------|
| Windows 11 | ✅ implemented (v0.2.1) | [`win/`](./win)         |
| macOS      | ✅ implemented (v0.2.1) | [`macos/`](./macos)     |
| Linux      | 🟡 planned              | `linux/` (not yet)      |

All app logic lives in [`core/`](./core) — a single frontend (`core/frontend`) and a single Rust lib (`core/rust-lib`) shared across platforms. Each OS has its own thin bundle shell that owns platform-specific details (installer config, icons, capabilities). To add a new platform, see [`CONTRIBUTING.md`](./CONTRIBUTING.md#adding-a-new-platform-shell-linux-etc).

## Features

### Clipboard History
- **Global hotkey** `Ctrl+Shift+V` opens a frameless, always-on-top popup centered on the monitor with the cursor.
- **Clipboard capture** — text, RTF, HTML, images (≤ 5 MB, stored as base64 PNG), and file lists via real OS clipboard change events (no polling).
- **Fuzzy search** (`fuse.js`, threshold 0.4) as you type.
- **Virtualized list** with a preview panel per content type (text, image, HTML render, RTF, file list).
- **Auto-paste** — Enter pastes the selected entry into the previously focused app (`enigo` simulates `Ctrl+V` on Windows / `Cmd+V` on macOS).
- **SQLite history** at `%APPDATA%\ClipSnap\history.db` (Windows) / `~/Library/Application Support/ClipSnap/history.db` (macOS), deduped on SHA-256, capped at 1 000 entries.

### Text Expander (snippets, v0.2)
- **Snippets** — store reusable text templates, each with a short abbreviation (e.g. `mfg`), an optional title, and a body.
- **Instant expansion** — type the abbreviation in the History search bar; matching snippets appear at the top of the list ranked above clipboard entries. Press Enter to paste the snippet body directly into the previously focused app.
- **Snippets tab** — dedicated management UI accessible via the **Snippets** tab button in the top-right of the popup. Create, edit, and delete snippets with a two-column form (abbreviation · title · body).
- **JSON import** — bulk-load snippets from a `.json` file via **Snippets → Import**, which opens the native file picker. Existing abbreviations are upserted (re-import is idempotent). Format reference in [`docs/snippets-import.md`](./docs/snippets-import.md); ready-to-import themed samples (signatures, dev boilerplates, markdown templates, …) under [`docs/examples/snippets/`](./docs/examples/snippets/).
- **Tray shortcut** — the system tray menu includes a **Manage Snippets** item that opens the popup directly on the Snippets tab.

### Inline Calculator (v0.2.5)
Type a math expression in the search field and the result appears as the top list item — Alfred-style. Press Enter to paste the result.

- **Operators:** `+ - * / % ^` (right-associative power), unary `+`/`-`, parentheses.
- **Numbers:** integers, decimals (`0.5`, `.5`), scientific (`1e3`, `1.5e-2`), digit grouping (`1_000`).
- **Constants:** `pi` / `π`, `tau`, `e`.
- **Functions:** `sqrt`, `cbrt`, `abs`, `sign`, `floor`, `ceil`, `round`, `ln`, `log` (base 10), `log2`, `exp`, `sin`/`cos`/`tan` (radians), `asin`/`acos`/`atan`/`atan2`, `sinh`/`cosh`/`tanh`, `min`, `max`, `pow`, `mod`.
- **Gating:** plain numbers (`42`) and plain text don't trigger calc mode — the input must contain at least one operator, function, or constant. Prefix with `=` to force evaluation of a single literal (`=pi`).
- **Implementation:** safe recursive-descent parser in [`core/frontend/src/lib/calc.ts`](./core/frontend/src/lib/calc.ts) — no `eval`. 27 unit tests in [`calc.test.ts`](./core/frontend/src/lib/calc.test.ts).

### Hex color preview + picker (v0.4.0, custom modal v0.5.0, click-to-select v0.5.1)
Type `#3366FF` (or `3366ff`, `#abc`, `#abcdef12`, …) in the search field and a color row appears at the top with a swatch + hex + RGB. Press Enter to paste the canonical `#RRGGBB` uppercase. The History tab's toolbar also has a **Color picker** button that opens an in-app **HSV picker modal** with hue slider, big preview swatch, and hex / RGB / HSL output tabs — click **Copy** to write the chosen value to the clipboard.

- 3/4-digit forms require the `#` prefix; 6/8-digit forms accept either form (so `abc` stays a search query but `abcdef` is a color).
- Preview pane shows a 128 px swatch with the hex overlaid (foreground auto-picked black/white via WCAG luminance) plus copy buttons for hex / RGB / HSL strings.
- **Two-click selection** (v0.5.1): opening the picker is *click 1*; the first click in the SV picker (or typing a hex) is *click 2 — the actual selection*. The big swatch and outputs stay in a placeholder state until then, so opening the modal never silently commits a default color.
- **Pick from screen** (v0.5.2): the modal's *Pick from screen* button samples a color from anywhere on the desktop. macOS uses Apple's own `NSColorSampler` magnifier loupe; Windows uses a fullscreen overlay + `GetPixel`. Cross-platform sampler in [`core/rust-lib/src/screen_picker.rs`](./core/rust-lib/src/screen_picker.rs).
- Pure frontend ([`core/frontend/src/lib/colors.ts`](./core/frontend/src/lib/colors.ts) + [`ColorPickerModal.tsx`](./core/frontend/src/components/ColorPickerModal.tsx)). 24 unit tests. Full reference: [`docs/colors.md`](./docs/colors.md).

### 25 bundled AI prompt snippets (v0.5.0)
First-launch seeds your snippet table with **`ai*`-prefixed prompts** covering programming (`aiplan`, `aireview`, `airefactor`, `airegex`, `aisql`, `aitest`, `aimigration`, `aibench`), web (`aithumb`, `aimobile`, `aia11y`, `aiseo`, `aicomponent`), IT security (`aithreat`, `aipentest`, `aiauth`, `aigdpr`), business workflows (`aibrief`, `airfp`, `aiokr`, `aichange`), data (`aidataq`, `aiml`, `aidashboard`), and API design (`aiapi`). Each prompt is a structured, opinionated brief — sections, bullets, output-format directives — written to be handed straight to an LLM without massaging.

- **Idempotent first-run seed** (deleted prompts stay deleted on relaunch)
- **Restore defaults** button in the Snippets-tab sidebar re-imports them all (upsert by abbreviation; your custom snippets are untouched)
- Pure data — embedded via `include_str!`. Edit, delete, extend in the Snippets tab like any other snippet.
- Full reference + the 25 abbreviations: [`docs/ai-prompts.md`](./docs/ai-prompts.md).

### Plain-text paste (default on, v0.4.0)
HTML / RTF clipboard entries are stripped to their plain-text preview at paste time, so copy-from-Word / browser / mail and paste-into-anything no longer leaks the source app's styling. Toggle in **Settings → Paste**. Hold <kbd>Shift</kbd> + <kbd>Enter</kbd> in the popup to override and paste with original formatting just for that one entry.

### Notes (v0.2.6)
Notes are **persistent, categorized clipboard items** — they live in their own SQLite table and are **not** affected by the 1 000-entry pruning of the clipboard history.

- **Bookmark a clipboard entry** — hover any History row, click the bookmark icon, the entry is copied into the `notes` table under `Uncategorized`. The note is decoupled from the clip, so even if the original gets pruned, the note stays.
- **Notes tab** — three-pane layout: **Categories sidebar** (with note counts; virtual `All` and `Uncategorized` groups), **note list**, **detail/edit pane**.
- **Free-form categories** — typing a new category name into the edit form auto-creates it; the input has a `<datalist>` autocomplete from existing categories.
- **Editable bodies** for `text`, `html`, `rtf`. `image` and `files` notes are read-only (you can still rename them and change category). Image notes paste back as images, HTML as HTML, etc. — content type is preserved.
- **From-scratch text notes** via **+ New Note** (no clipboard source needed).
- **Clear All** with confirmation; per-row delete via hover-trash.
- **Tray shortcut** — **Manage Notes** opens the popup directly on the Notes tab.
- Full reference: [`docs/notes.md`](./docs/notes.md).

### Backup — full app export / import (v0.2.6, refined in v0.2.12)
The **Settings** tab's *Backup & restore* section has **Export…** and **Import…** for a single-file JSON backup of the whole app, plus three checkboxes to pick which sections (history / snippets / notes) the export contains.

- **Export** writes a pretty-printed JSON file containing `{ version, exported_at, history, snippets, notes }` to a path of your choice (native save dialog). Unticked sections are written as empty arrays — useful for sharing snippets without leaking your clipboard history.
- **Import** merges that file back into the live database with sensible per-table semantics:
  - Snippets — **upsert by `abbreviation`** (existing rows overwritten).
  - History — **upsert by SHA-256 hash** (duplicates bump `last_used_at`; the 1 000-entry cap still applies).
  - Notes — **appended verbatim** with original timestamps preserved.
- **Versioned schema** — backups carry a `version` field; ClipSnap refuses to import a backup whose version is newer than the running build, instead of silently dropping fields.
- Full reference: [`docs/backup.md`](./docs/backup.md).

### System-wide text expander (v0.2.7, accessibility-first since v0.3.0)
Type a snippet abbreviation in **any** text field, press your configured hotkey, and ClipSnap replaces it in place with the snippet body — like aText / TextExpander, but **trigger-based** (no keylogger).

- **Default hotkey:** `Alt + Backquote` (= `Alt + ^` on a German keyboard, `Alt + \`` on US). Disabled by default — opt in from the **Settings** tab.
- **Hotkey is configurable** via a click-to-record field in the Settings panel. Bad combinations are rejected without losing your previous registration.
- **Two capture paths.** Since v0.3.0 the primary path uses the **OS accessibility layer** (`AXUIElement` on macOS, `IUIAutomation` on Windows) to read the focused field's value and selection range directly — no clipboard touch, no keystroke synthesis, no flickering selection. Falls back to the legacy `Cmd/Ctrl+Shift+← → Cmd/Ctrl+C → look up → Cmd/Ctrl+V` keystroke + clipboard roundtrip when the focused element doesn't expose accessibility info. The Settings panel's **Diagnose** button reports which path was used.
- **Cross-platform:** macOS / Windows / Linux X11. Wayland depends on the compositor's global-shortcut portal.
- **Caveats:** terminals (iTerm2, kitty, gnome-terminal) often expose AX/UIA for their content; legacy / hybrid GUI toolkits may not (Java/Swing without AccessBridge, native Carbon — those fall back to keystrokes). Image/files snippets are not expanded (text only).
- Full reference: [`docs/text-expander.md`](./docs/text-expander.md).

### Multi-monitor placement
The popup opens on the monitor that contains the mouse cursor at hotkey time, horizontally centered and ~⅓ from the top. Placement is **clamped** to the active monitor's bounds, so the window never extends past a screen edge — important for mixed-DPI setups (e.g., MacBook Retina + external display). Implementation in [`core/rust-lib/src/hotkey.rs`](./core/rust-lib/src/hotkey.rs).

### System Tray
Menu items: Open · Manage Snippets · Manage Notes · Pause Capture · Clear History · Start with Windows / Start at Login · Quit.

## Repository layout

```
clipsnap/
├── core/
│   ├── frontend/            # React 19 + TS + Tailwind v4 (cross-platform)
│   │   └── src/
│   │       ├── components/  # SearchBar, HistoryList/Item, PreviewPanel, SnippetsPanel, NotesPanel, …
│   │       ├── hooks/       # useClipboardHistory, useFuzzySearch, useSnippets, useNotes, useKeyboardNav
│   │       └── lib/         # ipc.ts, types.ts, calc.ts (Alfred-style evaluator), format.ts
│   └── rust-lib/            # Shared Rust app logic
│       └── src/
│           ├── lib.rs       # Tauri builder, plugin/tray setup, invoke_handler
│           ├── db.rs        # entries table, hash-dedup, prune
│           ├── snippets.rs  # snippets table, JSON upsert, exact-abbreviation lookup
│           ├── notes.rs     # notes table, categories, save_from_clip
│           ├── backup.rs    # full-app export/import (versioned JSON)
│           ├── settings.rs  # key/value store (expander hotkey + future prefs)
│           ├── expander.rs  # trigger-based text expander (AX/UIA primary, clipboard fallback)
│           ├── text_field/  # FieldAccess trait + macOS AX + Windows UIA implementations
│           ├── paste.rs     # write_to_clipboard + enigo paste shortcut
│           ├── hotkey.rs    # global Ctrl+Shift+V + expander hotkey, multi-monitor placement
│           ├── clipboard_watcher.rs  # event-driven capture, RTF stripping
│           └── commands.rs  # all #[tauri::command] wrappers
├── win/                     # Windows-specific bundle shell
│   ├── README.md            # Windows install & build details
│   ├── package.json         # Tauri CLI entry
│   └── src-tauri/           # main.rs, Cargo.toml, tauri.conf.json, capabilities/, icons/
├── macos/                   # macOS-specific bundle shell
│   ├── README.md            # macOS install, Gatekeeper, Accessibility, troubleshooting
│   ├── package.json
│   └── src-tauri/           # entitlements.plist, tauri.conf.json (dmg+app), capabilities/
├── .github/
│   └── workflows/
│       ├── ci.yml           # Rust + frontend tests on every push/PR
│       └── release.yml      # Builds bundles and publishes GitHub Release on v* tags
├── docs/
│   ├── spec.md              # Original product specification
│   ├── snippets-import.md   # JSON snippet import — schema, semantics, examples
│   ├── notes.md             # Notes feature — categories, edit semantics, IPC surface
│   ├── backup.md            # Full-app export/import — schema, merge semantics, jq recipes
│   ├── text-expander.md     # System-wide expander — workflow, hotkey format, per-OS caveats
│   ├── colors.md            # Inline hex preview + OS-native color picker
│   ├── ai-prompts.md        # 25 bundled default AI prompt snippets
│   ├── RELEASING.md         # Release procedure
│   └── examples/
│       └── snippets/        # 5 themed JSON examples + their own README
├── scripts/
│   └── check.sh             # cargo clippy + tsc + eslint
├── Cargo.toml               # Rust workspace (members: core/rust-lib, win/src-tauri, macos/src-tauri)
├── pnpm-workspace.yaml      # pnpm workspace (core/frontend, win, macos)
└── package.json             # Root scripts (dev:win, build:win, dev:macos, build:macos, lint, typecheck, test)
```

## Quick start

### Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| [Rust](https://rustup.rs/) | stable | MSVC toolchain on Windows; run `rustup component add clippy` |
| [Node.js](https://nodejs.org/) | 20+ | |
| [pnpm](https://pnpm.io/) | 10+ | `npm install -g pnpm` |

Platform-specific prerequisites:
- **Windows** → [`win/README.md`](./win/README.md) (WiX, MSVC build tools, WebView2)
- **macOS** → [`macos/README.md`](./macos/README.md) (Xcode CLT, Gatekeeper, Accessibility permission)

### Install & run

```bash
pnpm install          # install the whole workspace (CI uses --frozen-lockfile)

# Windows
pnpm dev:win          # tauri dev — live-reload
pnpm build:win        # → target/release/bundle/msi/ClipSnap_x.x.x_x64_en-US.msi

# macOS
pnpm dev:macos                      # tauri dev — live-reload
pnpm build:macos                    # → target/release/bundle/{macos/ClipSnap.app, dmg/ClipSnap_x.x.x_<arch>.dmg}
bash scripts/install-macos.sh       # build + re-sign + install into /Applications + launch
bash scripts/install-macos.sh --reset  # …also tccutil-reset stale Accessibility grants (use after first run)
```

> Why the `install-macos.sh` helper? Without an Apple Developer ID, every fresh `pnpm build:macos` gets a new random signing identifier, which makes macOS TCC treat each rebuild as a new app and prompt for Accessibility again. The script forces a stable ad-hoc identifier (`io.celox.clipsnap`) so the grant survives across rebuilds. Full background: [`macos/README.md` — Accessibility permission](./macos/README.md#why-the-dialog-re-appears-after-every-rebuild-and-how-to-stop-that).

> Each platform must be built on its native host (Windows for MSI, macOS for DMG/`.app`). Cross-compilation is not supported.

### Snippet import

In ClipSnap: open the popup (`Ctrl+Shift+V`) → **Snippets** tab → **Import** → pick a `.json` file. The native file picker opens (NSOpenPanel on macOS, OpenFileDialog on Windows); existing abbreviations are upserted in place so re-importing the same file is idempotent.

**Ready-to-import samples** in [`docs/examples/snippets/`](./docs/examples/snippets/):

| File | Snippets | Theme |
|------|----------|-------|
| [`getting-started.json`](./docs/examples/snippets/getting-started.json) | 3 | Address, email, German signature — first-import test |
| [`signatures.json`](./docs/examples/snippets/signatures.json) | 4 | Email signatures (DE/EN, short, OOO template) |
| [`dev.json`](./docs/examples/snippets/dev.json) | 8 | Shebang, MIT header, fn skeletons, gitignore, commit-msg |
| [`markdown.json`](./docs/examples/snippets/markdown.json) | 5 | Headings, table, `<details>`, PR-body |
| [`wrapped-form.json`](./docs/examples/snippets/wrapped-form.json) | 2 | Demonstrates `{ "snippets": [...] }` shape |

See [`docs/snippets-import.md`](./docs/snippets-import.md) for the full schema, field semantics, the sqlite3+jq export recipe, and tips/anti-patterns.

### Notes & Backup

Notes have their own tab; the categories sidebar has **+ New Note** and **Clear All**. Backup lives in the **Settings** tab now.

- **Save a clipboard entry as a note:** hover any History row → click the bookmark icon → the entry lands in the `Uncategorized` bucket of the Notes tab. Move it to a category by editing the note.
- **Export full backup:** Settings tab → **Backup & restore** → tick what to export (Clipboard history / Snippets / Notes — all default on) → **Export…** → choose a path. ClipSnap writes a single JSON file (default name `clipsnap-backup-<timestamp>.json`); unticked sections are written as empty arrays so you can share snippets without leaking your clipboard.
- **Import a backup:** Settings tab → **Backup & restore** → **Import…** → pick the JSON file. Snippets and history merge by their natural keys (abbreviation / SHA-256 hash); notes are appended. Notes / Snippets / History tabs auto-refresh.

Full feature reference: [`docs/notes.md`](./docs/notes.md). Backup file schema and merge semantics: [`docs/backup.md`](./docs/backup.md).

### Tests

```bash
pnpm test               # frontend unit tests (vitest + happy-dom) — 85 tests
cargo test --workspace  # Rust unit tests — 87 tests (db, snippets, notes, backup, settings, expander, text_field, seed, hotkey parser, clipboard_watcher, models)
```

The same commands run in [GitHub Actions CI](./.github/workflows/ci.yml) on every push and PR.

### Static analysis

```bash
pnpm check            # cargo clippy (workspace) + tsc --noEmit + eslint
```

## Known limitations

| Limitation | Detail |
|------------|--------|
| **At-rest encryption scope** | Sensitive content (clipboard text/HTML/RTF/images, snippet bodies, note bodies) is AES-256-GCM encrypted at rest with a per-install random 256-bit key (v0.6.0+). Key lives in the OS keychain; falls back to a 0600 keyfile in the data dir if the keychain is unavailable. **Not encrypted:** timestamps, content-type tags, dedup hashes, snippet abbreviations, note titles/categories — none of those reveal clipboard content. |
| **No sensitive-app detection** | ClipSnap captures everything without filtering. |
| **No cloud sync** | No automatic sync or multi-device support — but the [Backup](./docs/backup.md) export/import gives you a portable JSON file you can move between machines manually. |
| **File paste fallback** | Setting file-list clipboard payloads from Rust is not universally supported; ClipSnap falls back to pasting the newline-joined list of paths as text. |
| **macOS accessibility** | Paste simulation (`enigo`) requires Accessibility access. Grant it once in System Settings → Privacy & Security → Accessibility. If it isn't granted, ClipSnap shows an amber banner with an "Open Settings" shortcut on the next paste attempt instead of silently failing or re-firing the system dialog (v0.5.1). |
| **macOS unsigned build** | Release builds are not notarized. macOS may warn "unidentified developer" — right-click the app and choose Open to bypass Gatekeeper on first launch. |

## Contributing

Contributions welcome — see [`CONTRIBUTING.md`](./CONTRIBUTING.md) for the dev workflow, code style, and how to add IPC commands or new platform shells.

## Releasing

Push a `v*` tag to trigger the [release workflow](https://github.com/pepperonas/clipsnap/actions/workflows/release.yml), which builds the Windows and macOS bundles and attaches them to a GitHub Release. Full procedure (version bumps, pre-flight checks, troubleshooting) in [`docs/RELEASING.md`](./docs/RELEASING.md).

## Changelog

See [`CHANGELOG.md`](./CHANGELOG.md) — every release is documented with what was added, fixed, and any known issues at the time.

## License

[MIT](./LICENSE) — © 2026 Martin Pfeffer

Built on weekends and evenings — by day a happy software engineer ❤️ at [Bayootec](https://bayoo.net).
