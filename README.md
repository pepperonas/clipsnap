<div align="center">
  <img src="docs/inspector-rust.png?v=1" alt="Built with Rust" width="600" />

  # Inspector Rust

  **The keyboard-first clipboard hyper-toolkit for people who think `Cmd+V` should be a verb, a noun, and a way of life — Windows 11 & macOS, native, no Electron, no cloud, no bullshit.**

  This is the clipboard manager that ate its own scope statement.

  Press **`Ctrl+Shift+V`** anywhere on your system → a transparent, frameless popup ghosts into existence over the monitor your cursor lives on, auto-focuses a search bar, fuzzy-searches **1 000 deduped clipboard entries** by content (text, RTF, HTML preview, PNG bitmaps base64'd into SQLite, file path lists), **AES-256-GCM-encrypted at rest** with the key tucked into your **OS keychain** (macOS Keychain / Windows Credential Manager / Linux Secret Service, with a 0600 keyfile fallback for the paranoid). Hit **Enter** — it writes the entry back to the clipboard, hides itself, waits for focus to settle, synthesizes **`Cmd/Ctrl+V`** into whichever app you were just in. The whole loop is under 200 ms cold and under 50 MB RAM resident.

  But that's just the *first* tab. There are four.

  The **search bar doubles as an inline calculator** (`2+2`, `sqrt(144)`, `sin(pi/2)`, hex literals, bit shifts — anything `mathjs` can parse), **a colour converter** (paste `#0078d4` or `rgb(0,120,212)` and get every other format with one click), **a snippet matcher** (type `aiplan` → the *aiplan* AI-prompt body floats to the top of the list, Enter pastes it), and **a Color Picker** button that fires `NSColorSampler` on macOS or a GDI screen-overlay on Windows for system-wide eyedropping. **Image entries** get a preview pane with **Recolor** (9-swatch logo tint via per-pixel luminance lerp), **Cut out background** (a 4.5 MB U²-Net ONNX model statically linked into the binary via the `ort` crate — real photo segmentation, not chroma key, comparable to Python's `rembg` but without Python), and **Save to Downloads** (`⌘S` writes the PNG to disk, perfect for the freshly recoloured logo you just produced).

  **`Ctrl+Shift+O`** fires the **screen-region OCR**: drag a marquee (`screencapture -i` — the same overlay as `Cmd+Shift+4`), and Apple's **Vision** framework runs `VNRecognizeTextRequest` over the selection at `recognitionLevel = Accurate` with language correction. The recognised text lands on your clipboard, in your history (at the top — *fixed in v0.14.2*), and the source PNG is preserved one slot below so you can re-OCR a different region without rescreenshotting. Latin, CJK, Arabic, Cyrillic — whatever your macOS Vision install supports. **`Ctrl+Shift+S`** (new in **v0.15.0**) does the same marquee but **skips the OCR step** — pure region screenshot, PNG straight to clipboard + history, so charts, buttons, photos, and silent UI mockups land where you need them too. Same TCC gate, same threading model, zero text-required.

  The **text expander** has *three* expansion modes living side by side. The **search-based** one (always on, zero permissions): type `mfg` in the popup → matching snippets bubble to the top → Enter pastes. The **abbreviation hotkey** (default `Alt+1`, opt-in via Settings, configurable to anything): type the abbreviation in *any* text field, press the hotkey, Inspector Rust replaces it in place via macOS Accessibility API or Windows UIA (with an AX-select-then-paste fallback for Electron / Chromium / Mac-Catalyst apps that don't expose writable text — WhatsApp, Slack, Discord, VS Code — and a clipboard+keystroke last resort for everything else; the *Diagnose* button in Settings reports which path was used). And the **direct hotkey → snippet slots** (added v0.13.0): bind a hotkey straight to a snippet — `Alt+2` → the *aiplan* body — and pressing it pastes the body with **no abbreviation typed**. Reads nothing, so it works **in any app including terminals** (iTerm2, Terminal.app, kitty, Alacritty), where the abbreviation expander can't see the input line.

  Ships with **25 bundled AI prompt snippets** prefixed `ai*` across programming, web, IT security, business, data, and API design (`aiplan`, `aireview`, `airefactor`, `airegex`, `aisql`, `aitest`, `aimigration`, `aithumb`, `aithreat`, `aipentest`, `aibrief`, `aiml`, `aiapi`, `aiux`, `aimarketing`, …), each a structured-instruction half you append to your own prompt or code. Idempotent seeding (deleted prompts stay deleted), one-click *Restore defaults*, and live-editable from the **Snippets tab**. The **Notes tab** turns any clipboard entry into a permanent, categorised bookmark with no history cap. The **Backup tab** exports the full database (history + snippets + notes) into a versioned JSON file you can re-import on another machine; per-section tickboxes let you share snippets-only with a colleague without leaking your history. **Backup is plaintext JSON by design** so it's portable across machines — re-encryption happens at the destination's keychain on import.

  All of it runs as a **menu-bar / tray-resident background process** (no dock icon on macOS via `Accessory` activation policy, no taskbar icon on Windows via `skipTaskbar`). **`Autostart on login`** (added v0.14.0) is a tray-visible check menu item + Settings toggle — macOS writes `~/Library/LaunchAgents/InspectorRust.plist`, Windows uses the run-key registry entry. The popup is **per-monitor aware** (it opens on the monitor your cursor is on, not always the primary), **focus-loss-cancellable** (click outside or Esc to dismiss, with a `suppress_hide` flag during native file dialogs so they don't bounce the popup), and **fuzzy-search-as-you-type** via `fuse.js` with a virtualised list (`@tanstack/react-virtual`) that stays snappy at 1 000 entries.

  **Zero telemetry. Zero network calls. Zero account.** Your data lives at `~/Library/Application Support/InspectorRust/history.db` (macOS) or `%APPDATA%\InspectorRust\history.db` (Windows) and nowhere else. The 4.5 MB ONNX model is *bundled* — even cutouts run offline. The Vision OCR is *local* — Apple's on-device ML, no API key, no rate limit. The encryption keys never leave your machine, the snippets sync nowhere, the history is yours.

  Built with **Tauri 2** (WebView2 / WKWebView), **Rust** (workspace: `core/rust-lib` is the single shared library, `win/src-tauri` + `macos/src-tauri` are two-line bundle shells), **React 19** + **TypeScript 5** + **Tailwind v4** + **Vite 7**, packaged into a **~5 MB MSI** (Windows) or **~5 MB DMG** (macOS Apple Silicon). **110 Rust unit tests + 86 frontend vitest tests** keep it honest. **MIT-licensed**, hackable, and unapologetically built for the kind of person who already has muscle memory for three different clipboard managers and is tired of every one of them.

  <!-- ── Status / release ─────────────────────────────────────── -->
  [![Version](https://img.shields.io/badge/version-0.16.1-blue?style=flat-square)](https://github.com/pepperonas/inspector-rust/releases)
  [![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](./LICENSE)
  [![CI](https://img.shields.io/github/actions/workflow/status/pepperonas/inspector-rust/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/pepperonas/inspector-rust/actions/workflows/ci.yml)
  [![Release](https://img.shields.io/github/actions/workflow/status/pepperonas/inspector-rust/release.yml?branch=main&style=flat-square&label=release)](https://github.com/pepperonas/inspector-rust/actions/workflows/release.yml)
  [![Latest Release](https://img.shields.io/github/v/release/pepperonas/inspector-rust?style=flat-square&label=download)](https://github.com/pepperonas/inspector-rust/releases/latest)
  [![Maintenance](https://img.shields.io/badge/maintained-yes-brightgreen?style=flat-square)](https://github.com/pepperonas/inspector-rust/commits/main)
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
  [![cargo test](https://img.shields.io/badge/cargo%20test-110%20passing-success?style=flat-square&logo=rust&logoColor=white)](#)
  [![vitest](https://img.shields.io/badge/vitest-86%20passing-success?style=flat-square&logo=vitest&logoColor=white)](#)
  [![cargo clippy](https://img.shields.io/badge/cargo%20clippy-D%20warnings-success?style=flat-square&logo=rust&logoColor=white)](#)
  [![tsc strict](https://img.shields.io/badge/tsc-strict-3178C6?style=flat-square&logo=typescript&logoColor=white)](#)
  [![Prettier](https://img.shields.io/badge/code%20style-Prettier-F7B93E?style=flat-square&logo=prettier&logoColor=black)](https://prettier.io)

  <!-- ── Community ───────────────────────────────────────────── -->
  [![Issues](https://img.shields.io/github/issues/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/issues)
  [![Closed issues](https://img.shields.io/github/issues-closed/pepperonas/inspector-rust?style=flat-square&color=success)](https://github.com/pepperonas/inspector-rust/issues?q=is%3Aissue+is%3Aclosed)
  [![PRs](https://img.shields.io/github/issues-pr/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/pulls)
  [![Stars](https://img.shields.io/github/stars/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/stargazers)
  [![Forks](https://img.shields.io/github/forks/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/network/members)
  [![Watchers](https://img.shields.io/github/watchers/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/watchers)
  [![Contributors](https://img.shields.io/github/contributors/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/graphs/contributors)
  [![Last commit](https://img.shields.io/github/last-commit/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/commits/main)
  [![Commit activity](https://img.shields.io/github/commit-activity/m/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/commits/main)
  [![Repo size](https://img.shields.io/github/repo-size/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Code size](https://img.shields.io/github/languages/code-size/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Top language](https://img.shields.io/github/languages/top/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Languages](https://img.shields.io/github/languages/count/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Lines](https://img.shields.io/tokei/lines/github/pepperonas/inspector-rust?style=flat-square&label=lines%20of%20code)](https://github.com/pepperonas/inspector-rust)
  [![Made with love](https://img.shields.io/badge/made%20with-%E2%99%A5-red?style=flat-square)](#)

  <!-- ── Architecture & build ────────────────────────────────── -->
  [![Monorepo](https://img.shields.io/badge/repo-pnpm%20workspace-F69220?style=flat-square&logo=pnpm&logoColor=white)](./pnpm-workspace.yaml)
  [![Workspace crates](https://img.shields.io/badge/cargo%20workspace-3%20crates-CE422B?style=flat-square&logo=rust&logoColor=white)](./Cargo.toml)
  [![Single binary](https://img.shields.io/badge/distribution-single%20binary-blue?style=flat-square)](#)
  [![Native](https://img.shields.io/badge/no-Electron-success?style=flat-square)](#)
  [![Memory](https://img.shields.io/badge/memory-%3C50%20MB-blue?style=flat-square)](#)
  [![Cold start](https://img.shields.io/badge/cold%20start-%3C200%20ms-blue?style=flat-square)](#)
  [![MSI size](https://img.shields.io/badge/MSI-~5%20MB-blue?style=flat-square&logo=windows&logoColor=white)](#)
  [![DMG size](https://img.shields.io/badge/DMG-~5%20MB-blue?style=flat-square&logo=apple&logoColor=white)](#)
  [![exe size](https://img.shields.io/badge/.exe-~14%20MB-blue?style=flat-square&logo=windows&logoColor=white)](#)

  <!-- ── Features (numerical) ────────────────────────────────── -->
  [![IPC commands](https://img.shields.io/badge/IPC%20commands-59-blueviolet?style=flat-square)](./core/rust-lib/src/commands.rs)
  [![Tauri events](https://img.shields.io/badge/events-10-blueviolet?style=flat-square)](#)
  [![Rust modules](https://img.shields.io/badge/Rust%20modules-22-CE422B?style=flat-square&logo=rust&logoColor=white)](./core/rust-lib/src)
  [![Snippets](https://img.shields.io/badge/AI%20prompts-25%20bundled-blueviolet?style=flat-square)](./docs/ai-prompts.md)
  [![Tabs](https://img.shields.io/badge/popup%20tabs-4-blueviolet?style=flat-square)](#)
  [![DB tables](https://img.shields.io/badge/SQLite%20tables-4-003B57?style=flat-square&logo=sqlite&logoColor=white)](./docs/encryption.md)
  [![Global shortcuts](https://img.shields.io/badge/global%20hotkeys-3-blueviolet?style=flat-square)](#)
  [![Snippet expansion modes](https://img.shields.io/badge/expansion%20modes-3-blueviolet?style=flat-square)](./docs/text-expander.md)
  [![Image formats](https://img.shields.io/badge/image%20formats-5-blueviolet?style=flat-square)](#)

  <!-- ── Standards / conventions ─────────────────────────────── -->
  [![SemVer](https://img.shields.io/badge/semver-2.0-blue?style=flat-square)](https://semver.org)
  [![Keep a Changelog](https://img.shields.io/badge/changelog-Keep%20a%20Changelog-orange?style=flat-square)](https://keepachangelog.com)
  [![Conventional Commits](https://img.shields.io/badge/commits-conventional-orange?style=flat-square)](https://www.conventionalcommits.org)
  [![ARIA](https://img.shields.io/badge/a11y-keyboard%20first-blueviolet?style=flat-square)](#)
  [![ADRs in CHANGELOG](https://img.shields.io/badge/ADRs-in%20CHANGELOG-orange?style=flat-square)](./CHANGELOG.md)

  <!-- ── Permissions / OS surfaces ───────────────────────────── -->
  [![macOS TCC: Accessibility](https://img.shields.io/badge/macOS%20TCC-Accessibility-000000?style=flat-square&logo=apple&logoColor=white)](./docs/text-expander.md)
  [![macOS TCC: Screen Recording](https://img.shields.io/badge/macOS%20TCC-Screen%20Recording-000000?style=flat-square&logo=apple&logoColor=white)](#)
  [![Autostart](https://img.shields.io/badge/autostart-LaunchAgent%20%2F%20RegRun-blue?style=flat-square)](./CHANGELOG.md)
  [![Tray icon](https://img.shields.io/badge/UI-tray%20resident-blue?style=flat-square)](#)

  <!-- ── Tech (extended) ─────────────────────────────────────── -->
  [![rusqlite](https://img.shields.io/badge/rusqlite-bundled-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/rusqlite)
  [![enigo](https://img.shields.io/badge/enigo-paste%20sim-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/enigo)
  [![clipboard-rs](https://img.shields.io/badge/clipboard--rs-event%20driven-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/clipboard-rs)
  [![ort](https://img.shields.io/badge/ort-ONNX%20Runtime-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/ort)
  [![ring](https://img.shields.io/badge/ring-AES--256--GCM-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/ring)
  [![objc2](https://img.shields.io/badge/objc2-Vision%20FFI-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/objc2)
  [![Fuse.js](https://img.shields.io/badge/fuse.js-fuzzy%20search-005571?style=flat-square)](https://www.fusejs.io)
  [![lucide-react](https://img.shields.io/badge/icons-lucide--react-F56565?style=flat-square)](https://lucide.dev)
  [![react-virtual](https://img.shields.io/badge/list-react--virtual-FF4154?style=flat-square&logo=react&logoColor=white)](https://tanstack.com/virtual)

  <!-- ── Vibes ───────────────────────────────────────────────── -->
  [![Inspired by Alfred](https://img.shields.io/badge/inspired%20by-Alfred-blueviolet?style=flat-square)](#)
  [![Mouse-free](https://img.shields.io/badge/mouse-not%20required-brightgreen?style=flat-square)](#)
  [![Self-hosted](https://img.shields.io/badge/data-on%20your%20disk-brightgreen?style=flat-square)](#)
  [![Free forever](https://img.shields.io/badge/free-forever-brightgreen?style=flat-square)](./LICENSE)
  [![Made in Germany](https://img.shields.io/badge/made%20in-Germany-FFCE00?style=flat-square)](#)

  Press `Ctrl+Shift+V` → search → paste. Inspired by Alfred's clipboard viewer on macOS, scoped to one tool you can keep on every machine.
</div>

---

## Download

**Latest release:** [![Latest Release](https://img.shields.io/github/v/release/pepperonas/inspector-rust?style=flat-square&label=latest&color=green)](https://github.com/pepperonas/inspector-rust/releases/latest) — see the [CHANGELOG](./CHANGELOG.md) for what's new.

| Platform | File | Notes |
|----------|------|-------|
| **Windows 11 / 10** | [`InspectorRust_<ver>_x64_en-US.msi`](https://github.com/pepperonas/inspector-rust/releases/latest) | MSI installer — adds Start-menu entry & uninstaller |
| **Windows 11 / 10** | [`inspector-rust.exe`](https://github.com/pepperonas/inspector-rust/releases/latest) | Standalone exe — no install needed |
| **macOS 10.15+ (Apple Silicon)** | [`InspectorRust_<ver>_aarch64.dmg`](https://github.com/pepperonas/inspector-rust/releases/latest) | DMG for arm64 Macs |
| **macOS Intel** | — | Build from source: [`macos/README.md`](./macos/README.md) |
| **Linux** | — | Planned for a later release |

> **macOS Gatekeeper note.** Local-build releases are **not Apple-signed**. On first launch macOS will refuse to open the app — right-click → **Open** → confirm, or **System Settings → Privacy & Security → "Open Anyway"**. Then grant **two** TCC permissions:
> - **Accessibility** — required for paste (`enigo` synthesizes Cmd+V) and the system-wide text expander (Cmd+Shift+← / Cmd+C / Cmd+V cycle).
> - **Screen Recording** — required for the OCR shortcut (`screencapture -i` is attributed to Inspector Rust and macOS denies it without this grant). The Settings tab surfaces both with collapsible amber banners and one-click jumps to the right Privacy pane.
>
> Full setup in [`macos/README.md`](./macos/README.md).

---

## Platform support

| Platform   | Status         | Location                |
|------------|----------------|-------------------------|
| Windows 11 | ✅ implemented | [`win/`](./win)         |
| macOS      | ✅ implemented | [`macos/`](./macos)     |
| Linux      | 🟡 planned     | `linux/` (not yet)      |

All app logic lives in [`core/`](./core) — a single frontend (`core/frontend`) and a single Rust lib (`core/rust-lib`) shared across platforms. Each OS has its own thin bundle shell that owns platform-specific details (installer config, icons, capabilities). To add a new platform, see [`CONTRIBUTING.md`](./CONTRIBUTING.md#adding-a-new-platform-shell-linux-etc).

## Workflow

Inspector Rust is built for one workflow: **`Ctrl+Shift+V` → type → Enter**. The hotkey opens a frameless popup over the active monitor; whatever you type is fuzzy-searched across clipboard history, snippets, calc results, and color values; Enter pastes the top match into the previously focused app. No mouse, no menu trees, no per-app integrations.

Two more global shortcuts fire from anywhere — Inspector Rust's window doesn't need to be open or focused:

- **`Ctrl+Shift+O`** — screen-region **OCR**. Drag a marquee, Apple Vision recognises the text in the region, the text lands on your clipboard + at the top of History.
- **`Ctrl+Shift+S`** *(v0.15.0+)* — screen-region **screenshot**. Same marquee, but no OCR step: the captured PNG goes straight to the clipboard and into History. Use this when the region has no text (chart, button, photo, UI mockup) or you want the image itself.

Literal Control on every OS — same key on Windows and macOS. Both require the macOS **Screen Recording** TCC grant.

Everything else (snippets management, notes, settings, image tools) lives in the same popup behind tabs in the top-right — there's no separate window to alt-tab to. **Settings → Keyboard shortcuts** carries the full cheat sheet.

## Features

### Clipboard core
- **Global hotkey** `Ctrl+Shift+V` opens the popup centered on the monitor with the cursor.
- **Captures** text, RTF, HTML, images (PNG, ≤ 5 MB), and file lists via OS-native clipboard events (no polling). Image-before-files priority on macOS so Finder image-copies land as bitmaps, not paths.
- **Fuzzy search** (`fuse.js`, threshold 0.4) ranks matches as you type. Virtualized list, per-content-type preview pane.
- **Auto-paste** — Enter pastes via `enigo`-simulated `Ctrl+V` / `Cmd+V` into the previously focused app. Shift+Enter overrides the plain-text setting and pastes with original formatting.
- **SQLite store** at `%APPDATA%\InspectorRust\history.db` / `~/Library/Application Support/InspectorRust/history.db`. SHA-256 deduped, 1 000-entry cap.
- **AES-256-GCM at rest** since v0.6.0 — text/HTML/RTF/image bodies, snippet bodies, note bodies. Key in OS keychain (Keychain / Credential Manager / Secret Service), 0600 keyfile fallback. Full reference: [`docs/encryption.md`](./docs/encryption.md).
- **Time chip** (v0.10.3) — the relative-time hint on each row (`just now`, `1h ago`) becomes a tiny clickable button: hover shows both `Captured` and `Last used` absolute timestamps in a tooltip; click toggles the chip itself between relative and absolute display.

### Text expander (snippets, v0.2 — system-wide v0.2.7, hotkey overhaul v0.12.0, direct slots v0.13.0)
- **In-popup expansion** — type an abbreviation in the search bar; matching snippets surface above clipboard entries; Enter pastes the body.
- **Abbreviation expander** — type the abbreviation in *any* text field, press the configured hotkey (default `Alt+1`, opt-in via Settings; one-click presets `Alt+1` / `Alt+2` / `Alt+3`, or record any combination), Inspector Rust replaces it in place. Three paths: AX/UIA in-place replace (native apps — no clipboard touch, no flicker, verified by re-reading the value); AX-select-then-paste-over-selection for Electron / Chromium / Mac-Catalyst apps that expose `AXValue` read-only (WhatsApp, Slack, Discord, VS Code — v0.12.0); and a clipboard+keystroke fallback for everything else. Diagnose button in Settings reports which path was used.
  - *Why `Alt+1` and not `Alt+Backquote`?* The old default was unreachable on German ISO MacBooks (the physical `^` key reports as `IntlBackslash`). Digit-row keys are layout-stable everywhere. An un-customised old install is migrated to `Alt+1` once on upgrade (won't clobber a value you deliberately re-pick).
- **Direct hotkey → snippet slots (v0.13.0)** — bind a hotkey straight to a snippet (Settings → *Direct hotkey → snippet*); pressing it pastes the body at the cursor with **no abbreviation typed**. Reads nothing from the focused field — just writes the body to the clipboard, synthesizes paste, restores the clipboard — so it works in **any** app, **including terminals** (iTerm2, Terminal.app, …) where the abbreviation expander can't see the input line. Collisions with the popup / OCR / abbreviation hotkeys are rejected.
- **Loud on permission failure (macOS, v0.12.0)** — if Accessibility isn't granted, pressing the hotkey no longer silently no-ops: Inspector Rust opens its popup, switches to Settings, and shows an amber banner with `Force re-grant` → `Restart now`. (Same pattern as the OCR / paste banners. Direct slots use the same gate + banner.)
- **Snippets tab** for create/edit/delete with a two-column form. **JSON import** via Snippets → Import (`docs/snippets-import.md`, themed samples in `docs/examples/snippets/`).
- Caveat: the **abbreviation** expander can't work on a terminal command line (no AX-exposed input line, no GUI "select previous word" on a shell prompt — use a *Direct hotkey → snippet* slot, or the popup, in terminals). Image/file snippets aren't expanded (text only).
- Full reference: [`docs/text-expander.md`](./docs/text-expander.md).

### 25 bundled AI prompt snippets (v0.5.0, reworked v0.12.0)
First-launch seeds your snippet table with `ai*`-prefixed prompts across programming, web, IT security, business, data, and API design (`aiplan`, `aireview`, `airefactor`, `airegex`, `aisql`, `aitest`, `aimigration`, `aithumb`, `aithreat`, `aipentest`, `aibrief`, `aiml`, `aiapi`, …). Each prompt is the **structured-instruction half only** — no `[REQUIREMENT]`-style fill-in slots (removed in v0.12.0). You append it to your own prompt / code / context and the LLM picks up the subject from there. Idempotent (deleted prompts stay deleted), restorable from the Snippets sidebar — existing installs click *Restore defaults* to pick up the v0.12.0 style. Full list: [`docs/ai-prompts.md`](./docs/ai-prompts.md).

### Inline calculator (v0.2.5)
Type a math expression in the search field, the result appears as the top list item — Alfred-style. Press Enter to paste it.

- Operators `+ - * / % ^`, unary `+/-`, parens. Numbers: int/decimal/scientific/`1_000`-grouped. Constants: `pi`/`π`, `tau`, `e`. Functions: `sqrt`, `cbrt`, `abs`, `sign`, `floor`/`ceil`/`round`, `ln`/`log`/`log2`, `exp`, trig + hyperbolic + inverse, `min`/`max`/`pow`/`mod`.
- Gated to expressions with at least one operator/function/constant — plain numbers and text don't trigger. Force-evaluate a literal with `=` prefix (`=pi`).
- Safe recursive-descent parser in [`calc.ts`](./core/frontend/src/lib/calc.ts), no `eval`. 27 tests.

### Color tools (v0.4.0 → v0.5.2)
- **Inline hex preview** — type `#3366FF` (also `3366ff`, `#abc`, `#abcdef12`) → swatch + hex + RGB row at top → Enter pastes uppercase `#RRGGBB`.
- **HSV picker modal** — hue slider, big swatch, output tabs for hex / RGB / HSL, two-click selection (no silent default), copy via Tauri clipboard plugin (sidesteps WKWebView restrictions).
- **Pick from screen** — sample any pixel on the desktop. macOS: Apple's `NSColorSampler` magnifier loupe. Windows: fullscreen overlay + `GetPixel`. Module: [`screen_picker.rs`](./core/rust-lib/src/screen_picker.rs).
- Frontend in [`colors.ts`](./core/frontend/src/lib/colors.ts) + [`ColorPickerModal.tsx`](./core/frontend/src/components/ColorPickerModal.tsx). 32 tests. Reference: [`docs/colors.md`](./docs/colors.md).

### Screen-region OCR (v0.9.0, macOS)
Press `Ctrl+Shift+O` (or use the tray's **OCR Region** entry) → drag a marquee over any text on screen → Inspector Rust runs Apple Vision over the selection and writes the recognized text straight to your clipboard. The text also lands at the top of History; the source PNG is kept as a separate image entry just below, so you can re-OCR a different region without rescreenshotting and pressing Enter on the auto-selected top entry pastes the **text**, not the screenshot (ordering fixed in v0.14.2). The hotkey is **literal Control** on macOS too (v0.14.1+ — earlier builds used `⌘⇧O` which collided with IDE bindings).

- **Region picker** — uses `screencapture -i` (the same binary as Cmd+Shift+4), so the marquee UX is the polished one users already know. Esc cancels cleanly.
- **Engine** — Vision's `VNRecognizeTextRequest` with accuracy=Accurate + language correction; same engine that powers Apple Live Text. No model bundling, no network.
- **Languages** — whatever your macOS Vision install supports (Latin + CJK + Arabic + Cyrillic on macOS 13+).
- **Windows** — implementation pending (will use `Windows.Media.Ocr`).
- Modules: [`region_picker.rs`](./core/rust-lib/src/region_picker.rs), [`ocr.rs`](./core/rust-lib/src/ocr.rs).

### Image tools — recolor + ML cutout + save (v0.7.0 → v0.10.x)
On selected image entries, the preview pane exposes three actions:

- **Recolor** (v0.7.0) — for mostly-grayscale PNGs (logos / icons / silhouettes), 9 preset swatches + custom hex tint the image. RGB lerps from target → white by per-pixel luminance, alpha preserved. Saturated photos are auto-hidden from the toolbar (chromaticity gate). Adds the tinted version as a new history entry; original stays.
- **Cut out background** (v0.10.0) — runs the **U²-Net (U2Netp) ONNX model** (~4.5 MB embedded) over the image to detect the foreground subject; output is a transparent PNG saved to `~/Downloads/<name>-cutout-<ts>.png`. Shortcut `Cmd/Ctrl+B`. Works on real photos (airplane in sky, person against cluttered background, …) — same architecture as Python's `rembg`, just without Python. Inference runs via `ort` (ONNX Runtime, statically linked into the binary).
- **Save to Downloads** (v0.10.1) — drop the selected image entry to disk as `~/Downloads/inspector-rust-image-<ts>.png` unchanged. Shortcut `Cmd/Ctrl+S`. Companion to recolor: select the freshly-tinted history entry, hit `Cmd+S`, your file is in Downloads.
- **Inputs:** PNG, JPEG, WebP, GIF, BMP — for clipboard image entries *and* single-file Files entries (so a JPG copied from Finder works too). Output is always RGBA PNG.
- Modules: [`recolor.rs`](./core/rust-lib/src/recolor.rs), [`cutout_ml.rs`](./core/rust-lib/src/cutout_ml.rs). Legacy chroma-key cutout in [`cutout.rs`](./core/rust-lib/src/cutout.rs) is kept as a fast-path option but unused by default. 16 MP cap on inputs. Bundled model: [`core/rust-lib/models/u2netp.onnx`](./core/rust-lib/models/u2netp.onnx) (Apache-2.0).

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

### Permissions UX (v0.11.0)
Inspector Rust needs **two** independent macOS TCC grants — Accessibility (paste + text expander) and Screen Recording (OCR + screenshot region). The Settings tab surfaces each as a collapsible amber banner that:

- Stays loud (border + warning icon + primary `Open System Settings` button) when missing, but collapses to a single row by default so the page isn't cluttered.
- Pre-checks before invoking the relevant native call. OCR returns a `screen.permission_denied` sentinel rather than failing silently when Screen Recording is denied; a Tauri event opens the popup + flips an in-app toast banner pointing at the right pane.
- Polls the grant once per second while not granted, so the badge flips green ~1 s after the user toggles the System Settings switch — no panel reload needed.
- Each banner has a `tccutil reset` recovery button for the "toggle says on but the running process still sees denied" stale-cdhash state.

### Discoverability (v0.10.7)
- **Footer hints** — `⌃⇧O OCR` + `⌃⇧S Shot` rendered next to the `⏎ Paste · ↑↓ Navigate · Esc Close` strip so users see the global shortcuts every time they open the popup.
- **Settings → Keyboard shortcuts** — three-group cheat sheet (Global / Popup nav / Image actions) covering every shortcut the app binds. Modifier glyphs (`⌘` vs `Ctrl`, `⇧` vs `Shift`, `⌥` vs `Alt`) adapt to the running OS via the `IS_MAC` helper in [`core/frontend/src/lib/platform.ts`](./core/frontend/src/lib/platform.ts).
- **About dialog** — Settings → About opens a modal with version, license, year, target audience, and a tabular tech-stack overview.

### System tray + multi-monitor
- **Tray menu:** Open · Manage Snippets · Manage Notes · **OCR Region (Ctrl+Shift+O)** · **Screenshot Region (Ctrl+Shift+S)** *(v0.15.0+)* · Pause Capture · ☑/☐ Start with Windows / Start at Login (checkmark reflects state since v0.14.0) · Clear History · Quit.
- **Autostart on login** (v0.14.0) — toggle in Settings → Startup, or from the tray menu. macOS writes `~/Library/LaunchAgents/InspectorRust.plist`; Windows uses the run-key registry entry. App launches hidden in the tray so it's ready when the popup hotkey hits.
- **Multi-monitor placement:** popup opens on the monitor with the cursor, horizontally centered, ~⅓ from the top, clamped to the active monitor's bounds (matters on mixed-DPI setups).

## Repository layout

```
inspector-rust/
├── core/
│   ├── frontend/            # React 19 + TS + Tailwind v4 (cross-platform)
│   │   └── src/
│   │       ├── components/  # SearchBar, HistoryList/Item, PreviewPanel, SnippetsPanel, NotesPanel, …
│   │       ├── hooks/       # useClipboardHistory, useFuzzySearch, useSnippets, useNotes, useKeyboardNav
│   │       └── lib/         # ipc.ts, types.ts, calc.ts (Alfred-style evaluator), format.ts
│   └── rust-lib/            # Shared Rust app logic
│       ├── build.rs         # Links the macOS Vision framework for OCR
│       ├── models/
│       │   └── u2netp.onnx  # U²-Net cutout model (~4.5 MB, Apache-2.0)
│       └── src/
│           ├── lib.rs                # Tauri builder, plugin/tray setup, invoke_handler
│           ├── commands.rs           # all #[tauri::command] wrappers
│           ├── models.rs             # ContentType / ClipEntry / NewClip + caps
│           ├── db.rs                 # entries table, hash-dedup, prune
│           ├── crypto.rs             # AES-256-GCM at-rest encryption + OS-keychain key
│           ├── snippets.rs           # snippets table, JSON upsert, exact-abbreviation lookup
│           ├── seed.rs               # default AI-prompt snippets bundled at first launch
│           ├── notes.rs              # notes table, categories, save_from_clip
│           ├── backup.rs             # full-app export/import (versioned JSON)
│           ├── settings.rs           # key/value store (expander hotkey + future prefs)
│           ├── ui_state.rs           # suppress_hide flag for native-modal interaction
│           ├── expander.rs           # trigger-based text expander (AX/UIA primary, clipboard fallback)
│           ├── text_field/           # FieldAccess trait + macOS AX + Windows UIA implementations
│           ├── paste.rs              # write_to_clipboard + enigo paste shortcut
│           ├── hotkey.rs             # global Ctrl+Shift+V + Ctrl+Shift+O + Ctrl+Shift+S + expander hotkey + direct slots
│           ├── clipboard_watcher.rs  # event-driven capture, RTF stripping (image > files priority)
│           ├── recolor.rs            # image tint (lerp target ↔ white by per-pixel luminance)
│           ├── cutout.rs             # legacy chroma-key cutout (kept as fast-path option)
│           ├── cutout_ml.rs          # U²-Net-based subject cutout via `ort` (ONNX Runtime)
│           ├── screen_picker.rs      # color eyedropper (NSColorSampler / GDI overlay)
│           ├── region_picker.rs      # screencapture -i wrapper — shared by OCR + screenshot pipelines
│           ├── ocr.rs                # Apple Vision (VNRecognizeTextRequest) wrapper
│           └── screen_recording.rs   # macOS Screen Recording TCC permission API — gates OCR + screenshot
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
└── package.json             # Root scripts: dev:{win,macos}, build:{win,macos}, lint, typecheck, format, test, check
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
pnpm build:win        # → target/release/bundle/msi/InspectorRust_x.x.x_x64_en-US.msi

# macOS
pnpm dev:macos                      # tauri dev — live-reload
pnpm build:macos                    # → target/release/bundle/{macos/InspectorRust.app, dmg/InspectorRust_x.x.x_<arch>.dmg}
bash scripts/install-macos.sh       # build + re-sign + install into /Applications + launch
bash scripts/install-macos.sh --reset  # …also tccutil-reset stale Accessibility grants (use after first run)
```

> Why the `install-macos.sh` helper? Without an Apple Developer ID, every fresh `pnpm build:macos` gets a new random signing identifier, which makes macOS TCC treat each rebuild as a new app and prompt for Accessibility again. The script forces a stable ad-hoc bundle identifier so the grant survives across rebuilds. Full background: [`macos/README.md` — Accessibility permission](./macos/README.md#why-the-dialog-re-appears-after-every-rebuild-and-how-to-stop-that).

> Each platform must be built on its native host (Windows for MSI, macOS for DMG/`.app`). Cross-compilation is not supported.

### Snippet import

In Inspector Rust: open the popup (`Ctrl+Shift+V`) → **Snippets** tab → **Import** → pick a `.json` file. The native file picker opens (NSOpenPanel on macOS, OpenFileDialog on Windows); existing abbreviations are upserted in place so re-importing the same file is idempotent.

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
- **Export full backup:** Settings tab → **Backup & restore** → tick what to export (Clipboard history / Snippets / Notes — all default on) → **Export…** → choose a path. Inspector Rust writes a single JSON file (default name `inspector-rust-backup-<timestamp>.json`); unticked sections are written as empty arrays so you can share snippets without leaking your clipboard.
- **Import a backup:** Settings tab → **Backup & restore** → **Import…** → pick the JSON file. Snippets and history merge by their natural keys (abbreviation / SHA-256 hash); notes are appended. Notes / Snippets / History tabs auto-refresh.

Full feature reference: [`docs/notes.md`](./docs/notes.md). Backup file schema and merge semantics: [`docs/backup.md`](./docs/backup.md).

### Tests

```bash
pnpm test               # frontend unit tests (vitest + happy-dom) — 86 tests
cargo test --workspace  # Rust unit tests — 110 tests (db, snippets, notes, backup, settings, expander, text_field, seed, hotkey parser, clipboard_watcher, models, recolor, cutout, cutout_ml)
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
| **No sensitive-app detection** | Inspector Rust captures everything without filtering. |
| **No cloud sync** | No automatic sync or multi-device support — but the [Backup](./docs/backup.md) export/import gives you a portable JSON file you can move between machines manually. |
| **File paste fallback** | Setting file-list clipboard payloads from Rust is not universally supported; Inspector Rust falls back to pasting the newline-joined list of paths as text. |
| **Expander in terminals: use a direct slot** | The *abbreviation* expander does nothing on a terminal command line (Terminal.app, iTerm2, kitty, …) — terminals don't expose the input line via accessibility and a shell prompt has no GUI "select previous word". Use a **Direct hotkey → snippet** slot there (v0.13.0 — pastes without reading anything, works everywhere) or the popup (`Ctrl+Shift+V` → search → Enter). Electron / Chromium / Mac-Catalyst apps (WhatsApp, Slack, VS Code, …) *are* supported by the abbreviation expander as of v0.12.0, via an AX-select-then-paste path. |
| **macOS Accessibility** | Paste simulation (`enigo`) and the system-wide text expander require Accessibility access. Grant it once in System Settings → Privacy & Security → Accessibility. If missing, Inspector Rust shows an amber banner with an `Open Settings` button on the next paste attempt — and, since v0.12.0, also when the expander hotkey is pressed — instead of silently failing or re-firing the system dialog (v0.5.1 / v0.12.0). |
| **macOS Screen Recording** | OCR (`Ctrl+Shift+O`) **and** screenshot region (`Ctrl+Shift+S`, v0.15.0+) both require Screen Recording access — `screencapture -i` is attributed to Inspector Rust and macOS denies it without the grant. Pre-checked via `CGPreflightScreenCaptureAccess`; missing permission opens the popup + shows an amber banner pointing to the right Privacy pane (v0.11.0). |
| **macOS unsigned build** | Release builds are not notarized. macOS may warn "unidentified developer" — right-click the app and choose **Open** to bypass Gatekeeper on first launch. |
| **macOS rebuild ⇒ re-grant** | `cdhash` changes on every source-affecting rebuild, which invalidates the previous TCC grants. `scripts/install-macos.sh` skips re-signing when the source hash is unchanged so casual rebuilds survive; real source changes still require re-granting. |
| **OCR + Screenshot are macOS-only for now** | Region capture (shared by OCR and the v0.15.0 screenshot pipeline) plus Vision OCR ship only on macOS. Windows / Linux invocations of `ocrRegion()` / `screenshotRegion()` return a structured `"not implemented on this platform"` error; the workspace builds cross-platform via stubs. Windows region capture will use `Graphics.Capture` and Vision-equivalent OCR via `Windows.Media.Ocr` in a follow-up release. |

## Contributing

Contributions welcome — see [`CONTRIBUTING.md`](./CONTRIBUTING.md) for the dev workflow, code style, and how to add IPC commands or new platform shells.

## Releasing

Push a `v*` tag to trigger the [release workflow](https://github.com/pepperonas/inspector-rust/actions/workflows/release.yml), which builds the Windows and macOS bundles and attaches them to a GitHub Release. Full procedure (version bumps, pre-flight checks, troubleshooting) in [`docs/RELEASING.md`](./docs/RELEASING.md).

## Changelog

See [`CHANGELOG.md`](./CHANGELOG.md) — every release is documented with what was added, fixed, and any known issues at the time.

## License

[MIT](./LICENSE) — © 2026 Martin Pfeffer

A private open-source side project — built on weekends and evenings, made with ❤️.
