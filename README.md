<div align="center">
  <img src="docs/clipsnap.png" alt="ClipSnap" width="900" />
  <br />
  <img src="docs/rust.png?v=2" alt="Built with Rust" width="200" />

  # ClipSnap

  **The keyboard-first clipboard toolkit for power users — Windows 11 & macOS**

  Searchable history, system-wide snippets, inline calculator, color picker, image recolor + background removal, screen-region OCR — all behind one hotkey, all local, AES-256 encrypted at rest.

  <!-- ── Status / release ─────────────────────────────────────── -->
  [![Version](https://img.shields.io/badge/version-0.10.6-blue?style=flat-square)](https://github.com/pepperonas/clipsnap/releases)
  [![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](./LICENSE)
  [![CI](https://img.shields.io/github/actions/workflow/status/pepperonas/clipsnap/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/pepperonas/clipsnap/actions/workflows/ci.yml)
  [![Release](https://img.shields.io/github/actions/workflow/status/pepperonas/clipsnap/release.yml?branch=main&style=flat-square&label=release)](https://github.com/pepperonas/clipsnap/actions/workflows/release.yml)
  [![Latest Release](https://img.shields.io/github/v/release/pepperonas/clipsnap?style=flat-square&label=download)](https://github.com/pepperonas/clipsnap/releases/latest)
  [![Maintenance](https://img.shields.io/badge/maintained-yes-brightgreen?style=flat-square)](https://github.com/pepperonas/clipsnap/commits/main)
  [![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)](./CONTRIBUTING.md)

  <!-- ── Platforms ────────────────────────────────────────────── -->
  [![Windows 11](https://img.shields.io/badge/Windows-11-0078D4?style=flat-square&logo=windows11&logoColor=white)](./win)
  [![macOS](https://img.shields.io/badge/macOS-10.15+-000000?style=flat-square&logo=apple&logoColor=white)](./macos)
  [![Apple Silicon](https://img.shields.io/badge/arm64-Apple%20Silicon-555555?style=flat-square&logo=apple&logoColor=white)](./macos)
  [![x86_64](https://img.shields.io/badge/x86__64-supported-555555?style=flat-square)](#)
  [![Linux](https://img.shields.io/badge/Linux-planned-orange?style=flat-square&logo=linux&logoColor=white)](#)

  <!-- ── Stack ────────────────────────────────────────────────── -->
  [![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
  [![Rust](https://img.shields.io/badge/Rust-stable-CE422B?style=flat-square&logo=rust&logoColor=white)](https://rustup.rs)
  [![React 19](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=black)](https://react.dev)
  [![TypeScript 5](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org)
  [![Vite 7](https://img.shields.io/badge/Vite-7-646CFF?style=flat-square&logo=vite&logoColor=white)](https://vitejs.dev)
  [![Tailwind CSS v4](https://img.shields.io/badge/Tailwind-v4-38BDF8?style=flat-square&logo=tailwindcss&logoColor=white)](https://tailwindcss.com)
  [![pnpm](https://img.shields.io/badge/pnpm-10-F69220?style=flat-square&logo=pnpm&logoColor=white)](https://pnpm.io)
  [![Node 20](https://img.shields.io/badge/Node-20+-339933?style=flat-square&logo=node.js&logoColor=white)](https://nodejs.org)
  [![SQLite](https://img.shields.io/badge/SQLite-bundled-003B57?style=flat-square&logo=sqlite&logoColor=white)](https://sqlite.org)
  [![ONNX Runtime](https://img.shields.io/badge/ONNX%20Runtime-bundled-005CED?style=flat-square&logo=onnx&logoColor=white)](https://onnxruntime.ai)
  [![Apple Vision](https://img.shields.io/badge/Apple%20Vision-Live%20Text-000000?style=flat-square&logo=apple&logoColor=white)](#)
  [![U2Net](https://img.shields.io/badge/U%C2%B2--Net-cutout-orange?style=flat-square)](https://github.com/xuebinqin/U-2-Net)

  <!-- ── Security & ergonomics ───────────────────────────────── -->
  [![AES-256-GCM](https://img.shields.io/badge/encryption-AES--256--GCM-darkgreen?style=flat-square&logo=letsencrypt&logoColor=white)](./docs/encryption.md)
  [![Keychain-backed](https://img.shields.io/badge/keys-OS%20keychain-555555?style=flat-square)](./docs/encryption.md)
  [![Local-first](https://img.shields.io/badge/local--first-yes-brightgreen?style=flat-square)](#)
  [![No telemetry](https://img.shields.io/badge/telemetry-none-brightgreen?style=flat-square)](#)
  [![Offline](https://img.shields.io/badge/network-not%20required-brightgreen?style=flat-square)](#)
  [![Power user](https://img.shields.io/badge/audience-power%20users-blueviolet?style=flat-square)](#)
  [![Keyboard-first](https://img.shields.io/badge/UX-keyboard--first-blueviolet?style=flat-square)](#)

  <!-- ── Quality ─────────────────────────────────────────────── -->
  [![ESLint](https://img.shields.io/badge/ESLint-flat%20config-4B32C3?style=flat-square&logo=eslint&logoColor=white)](https://eslint.org)
  [![Vitest](https://img.shields.io/badge/Vitest-3-6E9F18?style=flat-square&logo=vitest&logoColor=white)](https://vitest.dev)
  [![cargo test](https://img.shields.io/badge/cargo%20test-107%20passing-success?style=flat-square&logo=rust&logoColor=white)](#)
  [![vitest](https://img.shields.io/badge/vitest-86%20passing-success?style=flat-square&logo=vitest&logoColor=white)](#)
  [![cargo clippy](https://img.shields.io/badge/cargo%20clippy-D%20warnings-success?style=flat-square&logo=rust&logoColor=white)](#)
  [![tsc strict](https://img.shields.io/badge/tsc-strict-3178C6?style=flat-square&logo=typescript&logoColor=white)](#)
  [![Prettier](https://img.shields.io/badge/code%20style-Prettier-F7B93E?style=flat-square&logo=prettier&logoColor=black)](https://prettier.io)

  <!-- ── Community ───────────────────────────────────────────── -->
  [![Issues](https://img.shields.io/github/issues/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/issues)
  [![Closed issues](https://img.shields.io/github/issues-closed/pepperonas/clipsnap?style=flat-square&color=success)](https://github.com/pepperonas/clipsnap/issues?q=is%3Aissue+is%3Aclosed)
  [![PRs](https://img.shields.io/github/issues-pr/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/pulls)
  [![Stars](https://img.shields.io/github/stars/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/stargazers)
  [![Forks](https://img.shields.io/github/forks/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/network/members)
  [![Watchers](https://img.shields.io/github/watchers/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/watchers)
  [![Contributors](https://img.shields.io/github/contributors/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/graphs/contributors)
  [![Last commit](https://img.shields.io/github/last-commit/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/commits/main)
  [![Commit activity](https://img.shields.io/github/commit-activity/m/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap/commits/main)
  [![Repo size](https://img.shields.io/github/repo-size/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap)
  [![Code size](https://img.shields.io/github/languages/code-size/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap)
  [![Top language](https://img.shields.io/github/languages/top/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap)
  [![Languages](https://img.shields.io/github/languages/count/pepperonas/clipsnap?style=flat-square)](https://github.com/pepperonas/clipsnap)
  [![Lines](https://img.shields.io/tokei/lines/github/pepperonas/clipsnap?style=flat-square&label=lines%20of%20code)](https://github.com/pepperonas/clipsnap)
  [![Made with love](https://img.shields.io/badge/made%20with-%E2%99%A5-red?style=flat-square)](#)

  Press `Ctrl+Shift+V` → search → paste. Inspired by Alfred's clipboard viewer on macOS, scoped to one tool you can keep on every machine.
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

## Workflow

ClipSnap is built for one workflow: **`Ctrl+Shift+V` → type → Enter**. The hotkey opens a frameless popup over the active monitor; whatever you type is fuzzy-searched across clipboard history, snippets, calc results, and color values; Enter pastes the top match into the previously focused app. No mouse, no menu trees, no per-app integrations.

Everything else (snippets management, notes, settings, image tools) lives in the same popup behind tabs in the top-right — there's no separate window to alt-tab to.

## Features

### Clipboard core
- **Global hotkey** `Ctrl+Shift+V` opens the popup centered on the monitor with the cursor.
- **Captures** text, RTF, HTML, images (PNG, ≤ 5 MB), and file lists via OS-native clipboard events (no polling). Image-before-files priority on macOS so Finder image-copies land as bitmaps, not paths.
- **Fuzzy search** (`fuse.js`, threshold 0.4) ranks matches as you type. Virtualized list, per-content-type preview pane.
- **Auto-paste** — Enter pastes via `enigo`-simulated `Ctrl+V` / `Cmd+V` into the previously focused app. Shift+Enter overrides the plain-text setting and pastes with original formatting.
- **SQLite store** at `%APPDATA%\ClipSnap\history.db` / `~/Library/Application Support/ClipSnap/history.db`. SHA-256 deduped, 1 000-entry cap.
- **AES-256-GCM at rest** since v0.6.0 — text/HTML/RTF/image bodies, snippet bodies, note bodies. Key in OS keychain (Keychain / Credential Manager / Secret Service), 0600 keyfile fallback. Full reference: [`docs/encryption.md`](./docs/encryption.md).
- **Time chip** (v0.10.3) — the relative-time hint on each row (`just now`, `1h ago`) becomes a tiny clickable button: hover shows both `Captured` and `Last used` absolute timestamps in a tooltip; click toggles the chip itself between relative and absolute display.

### Text expander (snippets, v0.2 — system-wide v0.2.7)
- **In-popup expansion** — type an abbreviation in the search bar; matching snippets surface above clipboard entries; Enter pastes the body.
- **System-wide expansion** — type the abbreviation in *any* text field, press the configured hotkey (default `Alt+Backquote`, opt-in via Settings), ClipSnap replaces it in place. Primary path reads the focused field via OS accessibility (`AXUIElement` / `IUIAutomation`) — no clipboard touch, no keystroke flicker. Falls back to clipboard+keystroke when the field doesn't expose AX/UIA. Diagnose button in Settings reports which path was used.
- **Snippets tab** for create/edit/delete with a two-column form. **JSON import** via Snippets → Import (`docs/snippets-import.md`, themed samples in `docs/examples/snippets/`).
- Caveat: terminals & legacy GUI toolkits sometimes don't expose AX/UIA cleanly → falls back to keystrokes; image/file snippets aren't expanded (text only).
- Full reference: [`docs/text-expander.md`](./docs/text-expander.md).

### 25 bundled AI prompt snippets (v0.5.0)
First-launch seeds your snippet table with `ai*`-prefixed prompts across programming, web, IT security, business, data, and API design (`aiplan`, `aireview`, `airefactor`, `airegex`, `aisql`, `aitest`, `aimigration`, `aithumb`, `aithreat`, `aipentest`, `aibrief`, `aiml`, `aiapi`, …). Each prompt is a structured brief, ready to hand to an LLM. Idempotent (deleted prompts stay deleted), restorable from the Snippets sidebar. Full list: [`docs/ai-prompts.md`](./docs/ai-prompts.md).

### Inline calculator (v0.2.5)
Type a math expression in the search field, the result appears as the top list item — Alfred-style. Press Enter to paste it.

- Operators `+ - * / % ^`, unary `+/-`, parens. Numbers: int/decimal/scientific/`1_000`-grouped. Constants: `pi`/`π`, `tau`, `e`. Functions: `sqrt`, `cbrt`, `abs`, `sign`, `floor`/`ceil`/`round`, `ln`/`log`/`log2`, `exp`, trig + hyperbolic + inverse, `min`/`max`/`pow`/`mod`.
- Gated to expressions with at least one operator/function/constant — plain numbers and text don't trigger. Force-evaluate a literal with `=` prefix (`=pi`).
- Safe recursive-descent parser in [`calc.ts`](./core/frontend/src/lib/calc.ts), no `eval`. 27 tests.

### Color tools (v0.4.0 → v0.5.2)
- **Inline hex preview** — type `#3366FF` (also `3366ff`, `#abc`, `#abcdef12`) → swatch + hex + RGB row at top → Enter pastes uppercase `#RRGGBB`.
- **HSV picker modal** — hue slider, big swatch, output tabs for hex / RGB / HSL, two-click selection (no silent default), copy via Tauri clipboard plugin (sidesteps WKWebView restrictions).
- **Pick from screen** — sample any pixel on the desktop. macOS: Apple's `NSColorSampler` magnifier loupe. Windows: fullscreen overlay + `GetPixel`. Module: [`screen_picker.rs`](./core/rust-lib/src/screen_picker.rs).
- Frontend in [`colors.ts`](./core/frontend/src/lib/colors.ts) + [`ColorPickerModal.tsx`](./core/frontend/src/components/ColorPickerModal.tsx). 24 tests. Reference: [`docs/colors.md`](./docs/colors.md).

### Screen-region OCR (v0.9.0, macOS)
Press `Cmd+Shift+O` (or use the tray's **OCR Region** entry) → drag a marquee over any text on screen → ClipSnap runs Apple Vision over the selection and writes the recognized text straight to your clipboard. The text also lands in the History tab and the source PNG is kept as a separate image entry so you can re-OCR a different region without rescreenshotting.

- **Region picker** — uses `screencapture -i` (the same binary as Cmd+Shift+4), so the marquee UX is the polished one users already know. Esc cancels cleanly.
- **Engine** — Vision's `VNRecognizeTextRequest` with accuracy=Accurate + language correction; same engine that powers Apple Live Text. No model bundling, no network.
- **Languages** — whatever your macOS Vision install supports (Latin + CJK + Arabic + Cyrillic on macOS 13+).
- **Windows** — implementation pending (will use `Windows.Media.Ocr`).
- Modules: [`region_picker.rs`](./core/rust-lib/src/region_picker.rs), [`ocr.rs`](./core/rust-lib/src/ocr.rs).

### Image tools — recolor + ML cutout + save (v0.7.0 → v0.10.1)
On selected image entries, the preview pane exposes three actions:

- **Recolor** (v0.7.0) — for mostly-grayscale PNGs (logos / icons / silhouettes), 9 preset swatches + custom hex tint the image. RGB lerps from target → white by per-pixel luminance, alpha preserved. Saturated photos are auto-hidden from the toolbar (chromaticity gate). Adds the tinted version as a new history entry; original stays.
- **Cut out background** (v0.10.0) — runs the **U2Netp ONNX model** (~4.5 MB embedded) over the image to detect the foreground subject; output is a transparent PNG saved to `~/Downloads/<name>-cutout-<ts>.png`. Shortcut `Cmd/Ctrl+B`. Works on real photos (airplane in sky, person against cluttered background, …) — same architecture as Python's `rembg`, just without Python. Inference runs via `ort` (ONNX Runtime, statically linked).
- **Save to Downloads** (v0.10.1) — drop the selected image entry to disk as `~/Downloads/clipsnap-image-<ts>.png` unchanged. Shortcut `Cmd/Ctrl+S`. Companion to recolor: select the freshly-tinted history entry, hit `Cmd+S`, your file is in Downloads.
- **Inputs:** PNG, JPEG, WebP, GIF, BMP — for clipboard image entries *and* single-file Files entries (so a JPG copied from Finder works too). Output is always RGBA PNG.
- Modules: [`recolor.rs`](./core/rust-lib/src/recolor.rs), [`cutout_ml.rs`](./core/rust-lib/src/cutout_ml.rs). 16 MP cap on inputs.

### Notes (v0.2.6)
Persistent, categorized clipboard items in a separate SQLite table — **not** subject to the 1 000-entry pruning.

- **Bookmark from history** — hover any row → bookmark icon → entry lands in Notes/`Uncategorized`. Decoupled from the source clip; survives pruning.
- **Notes tab** — three panes: categories sidebar (with counts; virtual `All` / `Uncategorized`), list, detail/edit. Free-form categories (`<datalist>` autocomplete). Editable bodies for text/HTML/RTF; image/files notes are read-only. Per-row delete + Clear All with confirm.
- **+ New Note** for from-scratch entries. Tray shortcut: **Manage Notes** opens the popup directly here.
- Reference: [`docs/notes.md`](./docs/notes.md).

### Backup — single-file JSON export/import (v0.2.6+)
Settings tab → *Backup & restore* → tick history / snippets / notes individually → Export to a JSON file. Import merges back: snippets upsert by abbreviation, history upserts by SHA-256, notes append. Versioned schema — newer backups are refused rather than silently truncated. Reference: [`docs/backup.md`](./docs/backup.md).

### Plain-text paste (default on, v0.4.0)
HTML / RTF clipboard entries are stripped to their text preview at paste time, so copy-from-Word / browser / mail no longer leaks styling into other apps. Toggle in Settings → Paste. Shift+Enter in the popup overrides for one paste.

### System tray + multi-monitor
- **Tray menu:** Open · Manage Snippets · Manage Notes · Pause Capture · Clear History · Start with Windows / Start at Login · Quit.
- **Multi-monitor placement:** popup opens on the monitor with the cursor, horizontally centered, ~⅓ from the top, clamped to the active monitor's bounds (matters on mixed-DPI setups).

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
│   ├── colors.md            # Inline hex preview + custom HSV picker + system eyedropper
│   ├── ai-prompts.md        # 25 bundled default AI prompt snippets
│   ├── encryption.md        # AES-256-GCM at-rest encryption — threat model, key storage, migration
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

> Why the `install-macos.sh` helper? Without an Apple Developer ID, every fresh `pnpm build:macos` gets a new random signing identifier, which makes macOS TCC treat each rebuild as a new app and prompt for Accessibility again. The script forces a stable ad-hoc bundle identifier so the grant survives across rebuilds. Full background: [`macos/README.md` — Accessibility permission](./macos/README.md#why-the-dialog-re-appears-after-every-rebuild-and-how-to-stop-that).

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
pnpm test               # frontend unit tests (vitest + happy-dom) — 86 tests
cargo test --workspace  # Rust unit tests — 107 tests (db, snippets, notes, backup, settings, expander, text_field, seed, hotkey parser, clipboard_watcher, models, recolor, cutout, cutout_ml)
```

The same commands run in [GitHub Actions CI](./.github/workflows/ci.yml) on every push and PR.

### Static analysis

```bash
pnpm check            # cargo clippy (workspace) + tsc --noEmit + eslint
```

## Known limitations

| Limitation | Detail |
|------------|--------|
| **At-rest encryption scope** | Sensitive content (clipboard text/HTML/RTF/images, snippet bodies, note bodies) is AES-256-GCM encrypted at rest with a per-install random 256-bit key (v0.6.0+). Key lives in the OS keychain; falls back to a 0600 keyfile in the data dir if the keychain is unavailable. **Not encrypted:** timestamps, content-type tags, dedup hashes, snippet abbreviations, note titles/categories — none of those reveal clipboard content. Full reference: [`docs/encryption.md`](./docs/encryption.md). |
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

A private open-source side project — built on weekends and evenings, made with ❤️.
