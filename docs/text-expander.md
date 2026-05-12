# Text expander

ClipSnap's **text expander** lets you trigger snippet expansion *outside* the popup — from any text field in any app. Type a snippet abbreviation, press your configured hotkey, ClipSnap replaces it in place with the snippet body. Trigger-based, not silent: no keylogger, you stay in control of when expansion happens.

Expander introduced in **v0.2.7**. Default hotkey changed from `Alt+Backquote` to `Alt+1` in **v0.12.0** (the old default was unreachable on German ISO MacBooks — see [Hotkey format](#hotkey-format)).

## Quick start

1. Open the popup (`Ctrl+Shift+V`) → **Settings** tab.
2. Toggle **Enable** on.
3. Keep the default hotkey `Alt+1`, click one of the `Alt+1` / `Alt+2` / `Alt+3` presets, or click the hotkey field and record your own combination.
4. Click **Save & re-register**.
5. Type an abbreviation (e.g. `mfg`) in any other app's text field, then press the hotkey.

> **macOS:** if Accessibility access isn't granted, pressing the hotkey now opens the ClipSnap popup, switches to Settings, and shows an amber banner — instead of silently doing nothing (v0.12.0). Grant it (Force re-grant → Restart now) and try again.

The abbreviation is replaced with the snippet body in place.

## How it works

The expander has **three paths**, tried in order; each later one is a fallback for the case the earlier one can't handle.

### Path 1 (default): Accessibility API — no clipboard touch

When you press the hotkey from a focused text field, ClipSnap asks the OS's accessibility layer directly:

- **macOS:** `AXUIElementCreateSystemWide` → `kAXFocusedUIElement` → `kAXValue` (the field's text) + `kAXSelectedTextRange` (cursor position). Compute the word before the cursor in pure code. Replace via `AXUIElementSetAttributeValue(kAXSelectedTextRange + kAXSelectedText)`, then **re-read `kAXValue` to verify** the replace actually landed (see Path 1b — many apps lie about this).
- **Windows:** `IUIAutomation::GetFocusedElement` → `IUIAutomationTextPattern::GetSelection` → `MoveEndpointByUnit(TextUnit_Word, -1)` to expand the start backwards by one word → `GetText` for the abbreviation. Replace via Backspace × char_count(word) + `enigo.text(body)` (UIA's `Replace` is patchily implemented; SendInput-based Backspace+type is more reliable on Windows).
- ClipSnap looks the captured word up in the `snippets` table via [`snippets::find_by_exact_abbreviation`](../core/rust-lib/src/snippets.rs) — exact case match preferred, case-insensitive fallback.
- **No clipboard read or write at all.** The user's clipboard is left untouched, no selection flickers in the source app.

This path requires the same macOS Accessibility permission already needed for paste; on Windows no extra permission is required.

### Path 1b (macOS): AX selected, but couldn't replace — paste over the selection

Electron / Chromium / Mac-Catalyst text views (WhatsApp Desktop, Slack, Discord, VS Code, …) expose `AXValue` **read-only**: setting `AXSelectedTextRange` *does* select the abbreviation, but setting `AXSelectedText` returns `kAXErrorSuccess` and silently does nothing. The verify step in Path 1 catches this (the field's text didn't change) and ClipSnap then pastes the snippet body over the now-selected abbreviation — exactly one clipboard write + `Cmd+V` + restore, **without** re-selecting (a `Cmd+Shift+←` here would swallow the previous word too). So expansion works in those apps; it just touches the clipboard briefly (saved & restored) instead of being a pure AX operation. Internally this is the `ReplaceOutcome::SelectionActive` branch in [`text_field`](../core/rust-lib/src/text_field/mod.rs) / `expander::paste_over_selection`.

### Path 2 (fallback): keystroke + clipboard roundtrip

When the focused element doesn't expose AX / UIA attributes at all (rare native Carbon controls, Java/Swing without AccessBridge, niche custom widgets) ClipSnap falls back to the original v0.2.x flow:

1. Save the current clipboard text.
2. Synthesize *select previous word*: **`Option+Shift+←`** on macOS, **`Ctrl+Shift+←`** elsewhere.
3. Synthesize *copy* (`Cmd/Ctrl+C`) and read the just-selected word out of the clipboard.
4. Look up the snippet.
5. **Hit:** write the body to the clipboard; synthesize paste (`Cmd/Ctrl+V`); after ~150 ms restore the original clipboard text.
6. **Miss:** nothing is pasted; the selection remains visible as a cue that the abbreviation didn't match.

The popup is *not* shown during either path — focus stays in the source app the entire time.

### How to tell which path was used

Settings → Text expander → **Diagnose** now reports the path it took:

- 🟢 *macOS AX (clean — no clipboard touch)*
- 🟢 *Windows UIA (clean — no clipboard touch)*
- 🟡 *Clipboard fallback — focused app didn't expose accessibility info*

If you see the amber fallback in an app you'd expect to support AX/UIA, file an issue with the app name and OS version.

## Threading model — must run on the main thread (macOS)

The entire expand cycle dispatches to the **main thread** via `AppHandle::run_on_main_thread`. Three call sites observe this rule:

- The global-shortcut handler in [`hotkey.rs::register_expander`](../core/rust-lib/src/hotkey.rs)
- The IPC command [`commands.rs::trigger_expand_at_cursor`](../core/rust-lib/src/commands.rs)
- The IPC command [`commands.rs::diagnose_expand_at_cursor`](../core/rust-lib/src/commands.rs) — uses an `mpsc::channel` to ferry the result back from the main-thread closure to the IPC handler thread.

**Why this matters (a real bug we hit in v0.2.10).** `enigo`'s macOS `Key::Unicode(...)` mapping calls `TSMGetInputSourceProperty` (Text Services Manager) to look up the layout-dependent keycode for `'c'` and `'v'`. TSM hard-asserts that it's invoked from the main thread; calling it from a worker thread fires `_dispatch_assert_queue_fail` and aborts the process with `EXC_BREAKPOINT` / `SIGTRAP`. Three crash reports under `~/Library/Logs/DiagnosticReports/clipsnap-2026-04-26-070*.ips` confirmed this stack:

```
_dispatch_assert_queue_fail
dispatch_assert_queue
TSMGetInputSourceProperty
enigo::macos_impl::keycode_to_string
enigo::Keyboard::key                  ← Key::Unicode('c') / ('v')
expander::send_modified_letter
expander::expand_at_cursor
std::sys::thread::unix::Thread::new::thread_start  ← worker thread!
```

The fix landed in v0.2.11. The ~330 ms main-thread block during the cycle (includes a 40 ms settle delay added in v0.12.0 so a still-held `Alt` from the hotkey itself is released before `enigo` synthesizes its own modifier chords) is invisible to the user because the popup is hidden the whole time.

If you ever extend the expander or add new IPC paths that call enigo on macOS: **dispatch to the main thread**. There is no escape route — even `Key::Other(keycode)` won't help, because users would still need a layout-aware keycode lookup, which routes back through TSM.

## Hotkey format

The hotkey is stored as a Tauri global-shortcut string of the form

```
<Modifier>+<Modifier>+<Code>
```

- **Modifiers:** `Ctrl`, `Shift`, `Alt`, `Meta` (alias `Cmd`/`Super`/`Command`), and the cross-platform `CmdOrCtrl`.
- **Code:** a W3C `KeyboardEvent.code` name. Examples: `Digit1`, `KeyE`, `Backquote`, `F5`, `ArrowLeft`. The plugin also accepts the literal characters when unambiguous (`` ` ``, `1`, `=`).

The Settings panel's hotkey-capture button records exactly the right format from a single keypress — you should never need to type this string by hand. The `Alt+1` / `Alt+2` / `Alt+3` preset buttons set `Alt+Digit1` / `Alt+Digit2` / `Alt+Digit3` directly. Stored codes render in the friendly form (`Alt+Digit1` → `Alt+1`) in tooltips and status text.

The default is **`Alt+Digit1`** (shown as `Alt+1`) — the `1`-row digit key, *not* the numpad. Digit-row keys have a fixed `KeyboardEvent.code` on every keyboard layout, aren't dead keys anywhere, and aren't reserved by macOS or Windows. The pre-0.12 default `Alt+Backquote` was a poor choice: on German ISO MacBooks the physical `^`/`°` key under Esc reports as `IntlBackslash` (and `Backquote` lands on a different/unreachable position), so the registered shortcut never matched the key the user pressed and the expander appeared dead. On upgrade, an un-customised `Alt+Backquote` install is migrated to `Alt+Digit1` exactly once — see `expander::migrate_legacy_default`; the `expander.hotkey_migrated_v0_12` flag prevents re-migrating a value you deliberately re-pick afterwards.

## Per-OS feasibility

| OS               | Hotkey | Selection roundtrip | Paste | Verdict |
|------------------|--------|---------------------|-------|---------|
| **macOS**        | ✅      | `Option+Shift+←` then `Cmd+C` | `Cmd+V` via `enigo`. Accessibility permission required (already needed for the popup paste flow). | ✅ |
| **Windows**      | ✅      | `Ctrl+Shift+←` then `Ctrl+C`  | `Ctrl+V` via `enigo`. No extra permission. | ✅ |
| **Linux X11**    | ✅      | `Ctrl+Shift+←` then `Ctrl+C`  | `Ctrl+V` via `enigo`. No extra permission. | ✅ |
| **Linux Wayland**| 🟡     | same                          | same  | ⚠️ Compositor-dependent. GNOME ≥ 41 and KDE Plasma expose the global-shortcut portal — works there. Sway/`wlroots`-based niche WMs may block global shortcuts entirely. |

ClipSnap is Windows-first; the Wayland gap is intentionally tolerated. If you hit it, run the X11 session of your distro.

## Caveats — what won't work cleanly

The expander is a **trigger-based macro**, not a deeply integrated input-method. There are situations where it falls short:

- **Terminal command lines — not supported.** Terminal.app, iTerm2, kitty, Alacritty, WezTerm, gnome-terminal: pressing the hotkey there does **nothing**. Terminals don't expose the input line through accessibility (Path 1/1b can't see it), and a shell prompt has no GUI-style "select previous word" — `Cmd/Ctrl+Shift+←` either does nothing on the input line or selects *screen* text, so Path 2's select+copy+paste grabs the wrong region or comes back empty. There's no clean fix short of per-shell readline integration, which is out of scope. **Workaround:** popup paste (`Ctrl+Shift+V` → search the abbreviation → Enter).
- **Electron / Chromium / Mac-Catalyst apps** (WhatsApp Desktop, Slack, Discord, VS Code, …) — *supported* since v0.12.0, but via Path 1b: the AX `AXSelectedText` set is a no-op there, so ClipSnap selects the abbreviation via AX and pastes the body over it (brief clipboard touch, saved & restored). If you ever see the abbreviation get *highlighted but not replaced*, that's the verify step working and the paste failing — usually a timing fluke; press the hotkey again.
- **Password fields** in many browsers and apps refuse synthetic paste — the abbreviation gets selected (visible) but the body never lands. Workaround: not appropriate to use the expander in password fields anyway. Use the popup.
- **Image / files snippets are not supported** by the expander. The orchestration is text-only on purpose: the previous-word selection is a single text run, and replacing it with an image / file-list payload doesn't make sense in most editors. Use the popup for those.
- **Web apps with custom keyboard handlers** (Google Docs, some IDE web frontends) intercept `Ctrl+Shift+←` for their own shortcuts. Same workaround — popup paste.
- **Linux Wayland** in restrictive compositors blocks global shortcuts entirely.

## Settings storage

Persisted in the `settings` table (introduced in v0.2.7):

| Key                            | Default        | Notes                                     |
|--------------------------------|----------------|-------------------------------------------|
| `expander.enabled`             | `false`        | Opt-in. Stored as the literal string `"true"`/`"false"`. |
| `expander.hotkey`              | `Alt+Digit1`   | Tauri shortcut string format (above).     |
| `expander.hotkey_migrated_v0_12` | *(unset)*    | Set to `"true"` once the one-time `Alt+Backquote` → `Alt+Digit1` migration has run. Idempotency guard. |

`settings` is a key/value table:

```sql
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

The settings module ([`core/rust-lib/src/settings.rs`](../core/rust-lib/src/settings.rs)) exposes `get`, `set`, `get_or`, `get_bool`. Future settings should land here too.

## macOS Accessibility — granting and surviving rebuilds

The expander uses the AX API to read/replace the focused field, and `enigo`'s `CGEventPost` (synthesizing `Cmd+Shift+←`/`Cmd+C`/`Cmd+V`) for the fallback path. Both are gated behind the **Accessibility** TCC permission. ClipSnap surfaces the state up-front and walks you through granting it:

- **Loud failure on hotkey press (v0.12.0).** If the permission is missing, pressing the expander hotkey no longer runs a doomed cycle that silently no-ops — `expand_at_cursor` returns the `ax.permission_denied` sentinel and the hotkey handler instead pops the popup, switches to the Settings tab, and emits `expander-permission-needed` (the frontend turns it into an amber banner). Same pattern as the OCR `screen.permission_denied` path.
- **Status badge.** The Settings panel shows `Accessibility access granted` (emerald) or `Accessibility access required` (amber) at the top of the expander section. Probed via FFI to `AXIsProcessTrusted()` from `ApplicationServices.framework`.
- **Auto-detect.** When the banner is amber, the panel polls `get_accessibility_status` once a second. As soon as you flip the toggle on in System Settings, the banner switches state without a panel reload.
- **Auto-restart.** macOS caches `AXIsProcessTrusted` per-process — so the *running* ClipSnap can't actually use a freshly granted permission until it's relaunched. On the false→true edge the panel surfaces a one-click **Restart now** button which calls the `relaunch_app` IPC: it spawns `open -n /Applications/ClipSnap.app` and exits, leaving the new process to inherit the granted state. Total post-grant flow: ~30 seconds.
- **Diagnose** (new in v0.2.9). The panel's "Diagnose" button captures the word before your cursor in the previously focused app, looks it up in the snippets table, and reports back `Captured`, `Snippet match`, and `Would paste` — *without* pasting. This isolates the lookup half from the paste half, so you can tell exactly which step is failing when expansion isn't working. Implementation hides the popup first so the synthetic Cmd+Shift+← reaches the source app instead of ClipSnap itself.

The grant is bound to the app's `(bundle id, cdhash)` tuple. Without an Apple Developer ID, ClipSnap is ad-hoc-signed, and *any* code change produces a new cdhash — invalidating the prior grant. The bundled `scripts/install-macos.sh` mitigates this by hashing the source tree and skipping `tauri build` + `codesign --force` entirely when nothing has changed (see [`macos/README.md`](../macos/README.md#why-the-dialog-re-appears-after-every-rebuild--and-how-its-mitigated)). The honest, *permanent* fix is an Apple Developer ID.

## IPC surface

| Command                       | Args                          | Returns                                |
|-------------------------------|-------------------------------|----------------------------------------|
| `get_expander_config`         | —                             | `{ enabled, hotkey, accessibility_granted }` |
| `set_expander_config`         | `enabled, hotkey`             | `ExpanderConfig` (the now-effective config — errors before writing if `hotkey` is malformed) |
| `get_accessibility_status`    | —                             | `boolean` — cheap probe; safe for polling |
| `request_accessibility_grant` | —                             | `boolean` — fires the macOS "would like to control…" prompt via `AXIsProcessTrustedWithOptions` |
| `open_accessibility_settings` | —                             | `void` — `open x-apple.systempreferences:…` |
| `trigger_expand_at_cursor`    | —                             | `void` — programmatic full expand (hides popup, sleeps, runs cycle) |
| `diagnose_expand_at_cursor`   | —                             | `{ captured, matched_abbreviation, paste_preview, path }` — capture half only, no paste. Errors with an explanatory message on macOS when Accessibility isn't granted. |
| `relaunch_app`                | —                             | `void` — `open -n /Applications/ClipSnap.app` then `app.exit(0)` |
| `quit_app`                    | —                             | `void` — `app.exit(0)` (no relaunch) |

```ts
// core/frontend/src/lib/ipc.ts
import { setExpanderConfig, diagnoseExpandAtCursor } from "../lib/ipc";
await setExpanderConfig(true, "Ctrl+Shift+E");
const result = await diagnoseExpandAtCursor();
// result.captured, result.matched_abbreviation, result.paste_preview
```

Backend implementation:

- [`core/rust-lib/src/expander.rs`](../core/rust-lib/src/expander.rs) — orchestration, FFI to `AXIsProcessTrusted`/`AXIsProcessTrustedWithOptions`, the diagnose/expand functions.
- [`core/rust-lib/src/hotkey.rs`](../core/rust-lib/src/hotkey.rs) — `parse_shortcut`, `ExpanderShortcutState`, `register_expander`.
- [`core/rust-lib/src/commands.rs`](../core/rust-lib/src/commands.rs) — every command above.

## Frontend settings UI

The **Settings** tab is the 4th tab next to History · Snippets · Notes. The expander section has:

- The **Accessibility status banner** (sticky amber if missing; inline emerald if granted) with **Open System Settings** + **Try system prompt** + **Re-check** buttons.
- An **Enable** checkbox.
- A **Hotkey** capture button — click to start recording; the next non-modifier keypress wins. Backspace clears, Esc cancels. WebKit on macOS doesn't focus `<button>` on click, so capture uses a window-level listener while recording. WebKit-quirky `IntlBackslash` (the German `^` key) is normalized to `Backquote` before persisting. Next to it: `Alt+1` / `Alt+2` / `Alt+3` preset buttons (the recommended choice — layout-stable).
- A **Reset** button that returns the field to `Alt+1`.
- **Save & re-register** — disabled until something changed.
- The **Diagnose** card — explains the workflow + the `Test now` button that runs `diagnose_expand_at_cursor`.
- A collapsible **Why does this keep happening on rebuild?** disclosure explaining the cdhash binding.

## Testing

Unit tests live in three modules and run as part of `cargo test --workspace`:

| Module             | Tests                                                                               |
|--------------------|--------------------------------------------------------------------------------------|
| `settings::tests`  | Missing key → `None`; set/get roundtrip; overwrite; `get_or` default; `get_bool` parsing (truthy / falsy / unknown / missing). |
| `snippets::tests`  | `find_by_exact_abbreviation`: exact-case wins; case-insensitive fallback; empty → `None`; whitespace trim; unknown → `None`. |
| `expander::tests`  | `trim_abbreviation` strips whitespace + NBSP; settings constants are stable (incl. the new default `Alt+Digit1` parses); expander keys roundtrip through the settings store; `migrate_legacy_default` upgrades an un-customised `Alt+Backquote` install (and is idempotent / leaves a custom hotkey alone); the `ax.permission_denied` sentinel string is stable. |

The selection-and-paste roundtrip itself can't be unit-tested without a real OS input stream — it's exercised by the manual smoke test (open TextEdit, type `mfg`, press the hotkey).

## See also

- [`docs/snippets-import.md`](./snippets-import.md) — bulk-load the snippet library that the expander reads from.
- [`docs/notes.md`](./notes.md) — Notes feature, persistent clipboard items.
- [`docs/backup.md`](./backup.md) — full-app export/import covers history, snippets, and notes. (The `settings` table is *not* included in v1 backups — your hotkey choice is per-machine.)
