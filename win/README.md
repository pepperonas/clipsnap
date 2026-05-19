# ClipSnap — Windows bundle

This directory contains the Windows-specific Tauri shell for ClipSnap. The shared app logic lives in [`../core`](../core); this shell only owns the installer config, icons, capabilities, and the thin `main.rs` that boots the shared lib with a Windows `tauri::Context`.

## Prerequisites (Windows)

- [Rust toolchain](https://rustup.rs/), MSVC target — with `clippy`: `rustup component add clippy`
- [Node.js](https://nodejs.org/) 20+ and [pnpm](https://pnpm.io/) 10+
- Microsoft **WebView2** runtime (preinstalled on Windows 11)
- [**Visual Studio Build Tools**](https://visualstudio.microsoft.com/visual-cpp-build-tools/) — "Desktop development with C++" workload (MSVC + Windows SDK)
- [**WiX Toolset v3**](https://wixtoolset.org/) — required for `.msi` bundling

## Development

From the repository root:

```bash
pnpm install          # installs the entire workspace (frontend + tauri CLI)
pnpm dev:win          # delegates to `pnpm --filter clipsnap-win tauri dev`
pnpm build:win        # produces the .msi installer
```

The `.msi` lands at:

```
win/src-tauri/target/release/bundle/msi/ClipSnap_<version>_x64_en-US.msi
```

Double-click to install. ClipSnap launches into the system tray — there's no Start-menu shortcut that opens a window; the UI is summoned with `Ctrl+Shift+V`.

## Usage

| Action                                | Keys                        |
|---------------------------------------|-----------------------------|
| Open popup                            | `Ctrl` + `Shift` + `V`      |
| Screen-region OCR (planned)           | `Ctrl` + `Shift` + `O` *(macOS-only for now — stub on Windows)* |
| Screen-region screenshot (planned)    | `Ctrl` + `Shift` + `S` *(macOS-only for now — stub on Windows)* |
| Expand snippet abbreviation in place  | `Alt` + `1` (default, configurable) |
| Direct-slot hotkeys (paste snippet)   | configurable in Settings — work in any app |
| Navigate list                         | `↑` / `↓`                   |
| Paste selected                        | `Enter` (or double-click)   |
| Close popup                           | `Esc` (or click outside)    |

### System tray menu

- **Open (Ctrl+Shift+V)** — show the popup
- **Manage Snippets** — open the popup directly on the Snippets tab
- **Manage Notes** — open the popup directly on the Notes tab
- **OCR Region (Ctrl+Shift+O)** — drag a marquee → recognised text on the clipboard
- **Screenshot Region (Ctrl+Shift+S)** — drag a marquee → PNG on the clipboard + history (no OCR, works on text-free regions; v0.15.0+)
- **Pause Capture** — stop recording new clipboard items
- **☑ Start with Windows** — toggle autostart at login; checkmark reflects current state (v0.14.0)
- **Clear History…** — wipe all stored entries
- **Quit ClipSnap** — exit

## Data location

```
%APPDATA%\ClipSnap\history.db
```

Up to 1000 entries; oldest pruned on each insert. Duplicates are deduplicated on SHA-256 hash of the payload — copying the same thing twice just bumps its `last_used_at`.

## Captured formats

| Type   | Stored             | Preview                                                  |
|--------|--------------------|----------------------------------------------------------|
| Text   | UTF-8 string       | `<pre>` plain                                            |
| HTML   | Raw markup         | Sandboxed `<iframe srcdoc>` (no script execution)        |
| RTF    | Raw RTF            | Stripped plain text; original RTF pasted on selection    |
| Image  | Base64 PNG, ≤5 MB  | `<img>` + dimensions + size                              |
| Files  | JSON list of paths | Line-per-path list                                       |

## Files in this directory

```
win/
├── package.json              # Tauri CLI entry + frontend proxy scripts
├── README.md                 # (this file)
└── src-tauri/
    ├── Cargo.toml            # bin crate, depends on ../../core/rust-lib
    ├── build.rs              # tauri-build
    ├── tauri.conf.json       # bundle = msi, frontendDist = ../../core/frontend/dist
    ├── capabilities/         # default + desktop capability permissions
    ├── icons/                # Windows icon set (.ico, .png)
    └── src/
        └── main.rs           # thin entrypoint: clipsnap_core::run(generate_context!())
```

## Troubleshooting

- **`cargo: command not found`** — install Rust via rustup, then restart the shell.
- **`wix: toolset not found`** during `build:win` — install [WiX v3](https://wixtoolset.org/) and ensure it's on your `PATH`.
- **Global hotkey doesn't fire** — another app may already hold `Ctrl+Shift+V`. Close Windows' built-in clipboard history (Win+V) or any third-party clipboard manager, then restart ClipSnap.
- **Paste inserts nothing** — some apps (elevated processes, some Electron apps) reject simulated keystrokes. Copy manually from the preview pane as a workaround.
