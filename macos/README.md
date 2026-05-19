# ClipSnap — macOS bundle

This directory contains the macOS-specific Tauri shell for ClipSnap. The shared app logic lives in [`../core`](../core); this shell only owns the bundle config (DMG / `.app`), entitlements, and a thin `main.rs` that boots the shared lib.

## Prerequisites (macOS)

- [Rust toolchain](https://rustup.rs/) (Apple Silicon: `aarch64-apple-darwin`; Intel: `x86_64-apple-darwin`) — with `clippy`: `rustup component add clippy`
- [Node.js](https://nodejs.org/) 20+ and [pnpm](https://pnpm.io/) 10+
- **Xcode Command Line Tools** — `xcode-select --install`
- macOS 10.15 (Catalina) or newer

No DMG-specific tooling is required — Tauri's `bundle_dmg.sh` runs out of the box on macOS.

## Build

From the repository root:

```bash
pnpm install                                 # workspace install
pnpm dev:macos                               # tauri dev — live-reload
pnpm build:macos                             # produces .app + .dmg
```

Outputs:

```
target/release/bundle/macos/ClipSnap.app
target/release/bundle/dmg/ClipSnap_x.x.x_<arch>.dmg
```

If only the `.app` is needed (no DMG), build directly:

```bash
cd macos && pnpm tauri build --bundles app
```

## Install

Drag the `.app` from the DMG to `/Applications`, or copy directly:

```bash
cp -R target/release/bundle/macos/ClipSnap.app /Applications/
xattr -dr com.apple.quarantine /Applications/ClipSnap.app   # unsigned dev build
open /Applications/ClipSnap.app
```

### Gatekeeper (unsigned builds)

Local builds are not Apple-signed. On first launch macOS will refuse to open the app:

- **Right-click** → **Open** → confirm **Open** in the dialog, **or**
- **System Settings → Privacy & Security → "Open Anyway"** at the bottom.

The `xattr -dr com.apple.quarantine …` command above sidesteps this for development builds.

### Accessibility permission (required for paste)

Auto-paste uses synthesized `Cmd+V` keystrokes via [`enigo`](https://docs.rs/enigo). macOS will prompt for **Accessibility** access on first paste:

1. **System Settings → Privacy & Security → Accessibility**
2. Enable **ClipSnap**
3. **Quit and relaunch** ClipSnap (the permission only takes effect on the next process start)

Without Accessibility access the popup still opens and you can read entries, but `Enter` will not paste into the previous app.

#### Why the dialog re-appears after every rebuild — and how it's mitigated

macOS TCC binds **every** grant (Accessibility, Screen Recording, PostEvent) to the app's **(bundle id, cdhash)** tuple. On macOS Sequoia (15) and Tahoe (26) this binding is *strict* — when the cdhash changes, every grant for that bundle id is dropped (the System Settings toggle may keep reading "on", but the running process is denied). Without an Apple Developer ID, ClipSnap is ad-hoc-signed; re-signing is needed for a stable bundle id, but each `codesign` invocation embeds a fresh CMS timestamp **and** Rust release builds aren't byte-reproducible, so naïvely re-signing on every install gives a new cdhash every time → all grants invalidated.

The `scripts/install-macos.sh` helper handles this in two ways:

1. **Idempotent install.** It SHA-256 compares the freshly built binary to whatever is currently installed at `/Applications/ClipSnap.app`. If they're identical (and the bundle identifier already matches), the script **skips both `cp` and `codesign`** — your install is preserved verbatim, the cdhash stays stable, and your TCC grant survives. Net effect: rebuilding without source changes never asks you to re-grant.
2. **Auto-restart prompt.** When real source changes do produce a new cdhash, the in-app **Settings tab** detects the missing grant, walks you through enabling ClipSnap in System Settings, and automatically prompts to relaunch ClipSnap with one click as soon as it sees the toggle flip on. Total round-trip: ~30 seconds.

```bash
# Build (or rebuild) and install. Re-grant only required if the binary
# actually changed.
bash scripts/install-macos.sh

# Wipe stale TCC entries first (Accessibility + PostEvent + ScreenCapture)
# — useful after multiple zombie ClipSnap entries pile up in System
# Settings from old builds, or when the toggle reads "on" but a feature
# (OCR / paste / expander) still acts denied because the OS-saved grant
# is bound to the previous cdhash.
bash scripts/install-macos.sh --reset
```

The script prints the cdhash at the end so you can confirm at a glance whether your grant will survive.

The honest, *permanent* fix to this re-grant churn is an Apple Developer ID (~$99/year). With a Developer-ID signature, TCC matches on (Team ID + bundle id) — Apple's signature anchors trust independently of cdhash, so source changes don't invalidate the grant. ClipSnap is currently distributed as ad-hoc-signed builds, so this is left as a personal choice.

## Usage

| Action                                | Keys                       |
|---------------------------------------|----------------------------|
| Open popup                            | `Ctrl` + `Shift` + `V`     |
| Screen-region OCR (drag a marquee)    | `Ctrl` + `Shift` + `O`     |
| Screen-region screenshot (no OCR)     | `Ctrl` + `Shift` + `S`     |
| Expand snippet abbreviation in place  | `Alt` + `1` (default, configurable) |
| Direct-slot hotkeys (paste snippet)   | configurable in Settings — work even in terminals |
| Navigate list                         | `↑` / `↓`                  |
| Paste selected                        | `Enter` (or double-click)  |
| Close popup                           | `Esc` (or click outside)   |

ClipSnap runs as a **menu-bar background app** — there is no Dock icon. The activation policy is set to `Accessory` on launch (see [`core/rust-lib/src/lib.rs`](../core/rust-lib/src/lib.rs)).

### Tray menu

- **Open (Ctrl+Shift+V)** — show the popup
- **Manage Snippets** — open the popup directly on the Snippets tab
- **Manage Notes** — open the popup directly on the Notes tab
- **OCR Region (⌃⇧O)** — drag a marquee → recognised text on the clipboard (literal Control, not Cmd — v0.14.1+)
- **Screenshot Region (⌃⇧S)** — drag a marquee → PNG on the clipboard + history, no OCR step (v0.15.0+)
- **Pause Capture** — stop recording new clipboard items
- **☑ Start at Login** — toggle macOS LaunchAgent registration (`~/Library/LaunchAgents/ClipSnap.plist`); checkmark reflects current state (v0.14.0)
- **Clear History…** — wipe all stored entries
- **Quit ClipSnap**

## Data location

```
~/Library/Application Support/ClipSnap/history.db
```

SQLite database holding both clipboard history (capped at 1 000, deduped on SHA-256) and snippets.

## Files in this directory

```
macos/
├── package.json              # Tauri CLI entry + frontend proxy scripts
├── README.md                 # (this file)
└── src-tauri/
    ├── Cargo.toml            # bin crate; pulls in `tauri/macos-private-api`
    ├── build.rs              # tauri-build
    ├── tauri.conf.json       # bundle = ["dmg","app"], frontendDist = ../../core/frontend/dist
    ├── entitlements.plist    # macOS sandbox/entitlement declaration
    ├── capabilities/         # default + desktop capability permissions
    └── src/
        └── main.rs           # thin entrypoint: clipsnap_core::run(generate_context!())
```

## Multi-monitor placement

The popup opens on the monitor that contains the mouse cursor at hotkey time. The window is horizontally centered and placed roughly ⅓ from the top of the active monitor; placement is clamped to the monitor's bounds so the popup can never extend past a screen edge — important for mixed-DPI setups (e.g., MacBook Retina + external display). See `core/rust-lib/src/hotkey.rs::clamp_into_monitor`.

## Troubleshooting

- **App opens, hotkey does nothing.** Another app may already hold `Ctrl+Shift+V` (some launchers, IDEs). Close suspected conflicts and relaunch.
- **Popup opens but `Enter` does not paste.** Accessibility permission is missing — see "Accessibility permission" above. After granting, **quit and relaunch** ClipSnap.
- **`failed to bundle project … bundle_dmg.sh` during `pnpm build:macos`.** The DMG step occasionally fails on busy disks (FileVault background indexing, Time Machine snapshot, etc.). The `.app` itself is already built — install it directly with `cp -R target/release/bundle/macos/ClipSnap.app /Applications/`. Or rebuild only the `.app` with `pnpm tauri build --bundles app`.
- **Tray icon missing after launch.** macOS sometimes hides menu-bar icons when there's no room. Click and drag in the menu bar with `Cmd` held, or use [Bartender](https://www.macbartender.com/) / [Hidden Bar](https://github.com/dwarvesf/hidden) to pin it.
