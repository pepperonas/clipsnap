# Changelog

All notable changes to ClipSnap are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] — 2026-05-09

### Added — Screen-region OCR (macOS)

- **`Cmd+Shift+O` triggers an interactive screen-region picker.** Drag a marquee over any text on screen, ClipSnap runs Apple Vision OCR on the selection, writes the recognized text to the system clipboard, and pushes it into history. The source PNG is kept as a separate image entry so the user can re-OCR a different region without rescreenshotting. Tray menu also exposes an **OCR Region (⌘⇧O)** entry for discoverability. — *#feat(ocr)*
  - **Region picker** ([`region_picker.rs`](./core/rust-lib/src/region_picker.rs)) shells out to `/usr/sbin/screencapture -i -x -t png`, the same binary backing Cmd+Shift+4 — battle-tested marquee UX (Esc cancels, Space drags the rect, etc.) without reinventing an `objc2` overlay window. Captured PNG read from a temp file then deleted.
  - **OCR engine** ([`ocr.rs`](./core/rust-lib/src/ocr.rs)) uses Vision's `VNRecognizeTextRequest` (accuracy=Accurate, `usesLanguageCorrection=true`) via raw `objc2` `msg_send`. Joins one `\n` between observations (Vision returns one observation per visual line). Empty results are surfaced as `OcrResult { chars: 0 }` rather than an error so the UI can differentiate "engine ran but found nothing" from "engine failed".
  - **Build** — new `core/rust-lib/build.rs` emits `cargo:rustc-link-lib=framework=Vision` on macOS so the framework is linked. No new crate dependencies.
  - **IPC:** `ocr_region() -> { text, cancelled, chars }`. Both the global shortcut and the tray menu route through the shared `commands::run_ocr_pipeline(app)` helper, which dispatches the screencapture wait to a worker thread.
  - **Watcher integration:** the OCR pipeline calls `mark_self_write` before writing, so the clipboard watcher doesn't double-capture the result as a fresh user copy.
  - **Windows:** stubbed — both `region_picker::capture` and `ocr::recognize` return "not yet implemented on Windows" so the workspace still builds. Implementation will use `Windows.Media.Ocr` + a snipping overlay in a follow-up release.

## [0.8.0] — 2026-05-09

### Added — Image cutout / Freistellen

- **Background-removal action** in the image preview pane. Selecting an image entry shows a "Cut out background" button (plus `Cmd/Ctrl+B` shortcut); clicking it chroma-keys the image and saves the transparent PNG to `~/Downloads/clipsnap-cutout-<timestamp>.png`. — *#feat(image)*
  - **Algorithm.** Sample the four corners of the image (8×8 patches per corner, median per channel — robust to subject pixels bleeding into the corner regions), treat that as the background colour, and replace each pixel with `alpha = 0` if its colour is within 30 RGB units of the background, `alpha = original` if beyond 50 units, with linear feathering in the band between (smooth cutout edge).
  - **Sweet spot.** Subjects on uniform backgrounds — sky, studio backdrops, solid logo fields. Cluttered / busy backgrounds hit the limit of chroma-keying; pro-grade results would need ML (rembg / U2Net), which is out of scope for a clipboard utility.
  - **Bounds & safety.** Hard cap at 16 megapixels. Output goes to `~/Downloads` (or `$HOME` if that doesn't resolve); the source history entry is left untouched.
  - **Module:** [`core/rust-lib/src/cutout.rs`](./core/rust-lib/src/cutout.rs) (~210 LOC). 5 unit tests cover background detection, subject preservation, oversize rejection, the all-background degenerate case, and transparent-corner handling.
  - **IPC:** `cut_out_image_entry(id) → saved_path`. Frontend wrapper in [`ipc.ts`](./core/frontend/src/lib/ipc.ts), UI in `CutoutButton` inside [`PreviewPanel.tsx`](./core/frontend/src/components/PreviewPanel.tsx).

### Added — About dialog + footer credit

- **About dialog** behind a button in **Settings → About**. Shows version, developer, license, year, target-audience pitch, and a tabular tech-stack overview (Tauri 2 / Wry / Rust / SQLite + AES-256-GCM / React 19 / TypeScript 5 / Vite 7 / Tailwind v4 / `image` 0.25). Esc / backdrop / X all close. — *#feat(ui)*
- **Author credit** ("made with ♥ by Martin Pfeffer") added to the popup footer next to the version chip and entry counter. — *#feat(ui)*

### Changed — Documentation

- **README rewrite.** Subtitle now reads "The keyboard-first clipboard toolkit for power users — Windows 11 & macOS"; new **Workflow** section frames the `Ctrl+Shift+V → type → Enter` loop; **Features** section reorganised by theme (Clipboard core / Text expander / AI prompts / Calculator / Color tools / Image tools / Notes / Backup / Plain-text paste / Tray + multi-monitor) with each block tightened to a scannable header + 3–6 bullets. Encryption (v0.6.0) promoted from "Limitations" into the Clipboard core feature list where it belongs.
- **Tauri bundle metadata** (`copyright`, `shortDescription`, `longDescription`) updated to drop the `celox.io` chatter and reflect the broader feature set / power-user audience. Bundle id stays `io.celox.clipsnap` — that's a stable technical identifier the keychain & TCC depend on.
- **Snippet example signatures** anonymised to use `Your Name` / `https://example.com` placeholders so they're useful as templates for any user.

## [0.7.0] — 2026-05-08

### Added — Image recolor

- **Recolor toolbar in the image preview pane.** Selecting a mostly-grayscale image entry (logo, icon, silhouette) reveals a row of 9 preset swatches plus a hex input below the preview. Clicking a swatch or pressing Enter on a hex tints the image and stores the result as a new history entry — the original stays put. — *#feat(image)*
  - **Algorithm.** Decode PNG → for each RGBA pixel, replace RGB with `lerp(target, white, BT.601_luminance)`, preserve alpha → re-encode. Equivalent to ImageMagick's `+level-colors target,white`. Pure Rust via the `image` 0.25 crate (PNG-only feature set, no other format codecs pulled in).
  - **Photo guard.** Chromaticity sampling (`max((max-min)/max)` over up to 4096 opaque pixels) gates the UI: ≥ 0.12 hides the toolbar so saturated photos can't get accidentally tinted into Photoshop disasters.
  - **Bounds.** Hard cap at 16 megapixels to keep the synchronous recolor on the UI thread responsive on slower hardware.
  - **Module:** [`core/rust-lib/src/recolor.rs`](./core/rust-lib/src/recolor.rs) (~140 LOC). 6 unit tests cover dark→target mapping, white→white anchor, alpha preservation, oversize rejection, and chromaticity probe edges (pure-grayscale → ~0, pure-red → > 0.9).
  - **IPC:** `recolor_image_entry(id, hex) → new_id`, `image_chromaticity(id) → 0..1`. Frontend wrapper in [`core/frontend/src/lib/ipc.ts`](./core/frontend/src/lib/ipc.ts); UI in `RecolorToolbar` inside [`PreviewPanel.tsx`](./core/frontend/src/components/PreviewPanel.tsx).
  - **Deps added:** `image` 0.25 with `default-features = false, features = ["png"]` (avoids BMP/GIF/HDR/EXR/etc. baggage).

### Fixed — Clipboard capture priority

- **Image-before-files in the watcher.** macOS puts both the bitmap *and* the file path on the pasteboard when you copy an image file (PNG / JPG / HEIC) from Finder or use "Share → Copy Image" in many apps. The previous priority order (`files → image → …`) meant ClipSnap stored only the path — users would see `/Users/.../foo.png` in history instead of the actual picture. Order is now `image → files → html → rtf → text`; pure file copies (PDFs, ZIPs, …) still capture as Files exactly as before. — *#fix(watcher)*

## [0.6.1] — 2026-05-07

### Fixed

- **Clear all confirmation** — replaced unreliable `window.confirm` (silent in Tauri's WebView2) with an inline "Delete N clips? Yes / Cancel" prompt in the history toolbar. — *#fix(ui)*
- **Bookmark visual feedback** — clicking the bookmark icon now shows a filled `BookmarkCheck` icon in accent color for 1.5 s so the user can see the note was saved. — *#fix(ui)*
- **Color picker modal height** — reduced SVPicker height (`h-44 → h-32`), swatch height (`h-16 → h-10`), and tightened margins so the modal fits inside the 500 px popup on Windows without scrolling. — *#fix(color-picker)*

## [0.6.0] — 2026-05-06

### Added — At-rest encryption for sensitive content

- **The SQLite database now encrypts every sensitive content field with AES-256-GCM.** Closes the long-standing "Unencrypted storage" limitation row in the README — passwords, tokens, snippet bodies, and note bodies are no longer readable to anyone who can `cat` the DB file. — *#feat(security)*
  - **Encrypted columns:** `entries.content_text`, `entries.content_data`, `snippets.body`, `notes.content_text`, `notes.content_data`. **Not encrypted:** timestamps, content-type tags, dedup `hash`, snippet abbreviations, note titles/categories — those are metadata that doesn't reveal clipboard content.
  - **Storage format.** Each encrypted value is stored as TEXT prefixed with `v1:` followed by base64 of `12-byte random nonce ‖ ciphertext+tag`. Legacy plaintext rows (no `v1:` prefix) are detected on read and returned as-is, then re-encrypted in place by the migration step at next startup. The migration is idempotent — already-encrypted rows are skipped.
  - **Key storage.** Per-install random 256-bit key kept in the **OS keychain** (macOS Keychain / Windows Credential Manager / Linux Secret Service) under service `io.celox.clipsnap`, account `history-db-key-v1`. Falls back to a 0600 keyfile (`<data-dir>/.dbkey`) if the keychain is unavailable so the app stays usable instead of crashing. The fallback is strictly weaker — file-system access gets you the key — but matches the previous threat model floor.
  - **Roundtrip-safe across paths.** `save_from_clip` (Notes ← Clipboard) passes the already-encrypted ciphertext straight into the notes row instead of decrypt-then-reencrypt — same key, same scheme, ~free. `append_imported` from a JSON backup re-encrypts on the way in (backups stay plaintext for portability).
  - **Module:** [`core/rust-lib/src/crypto.rs`](./core/rust-lib/src/crypto.rs) (~280 LOC). 6 unit tests cover roundtrip, legacy plaintext passthrough, empty strings, fresh-nonce-per-encrypt, tampered-ciphertext rejection, wrong-key rejection.
  - **Deps added:** `aes-gcm` 0.10, `rand` 0.8, `keyring` 3 (cross-platform OS-keychain crate).

### Why 0.6.0

This is a feature with security implications and a one-time data migration on first launch — not a bug fix. Per `docs/RELEASING.md`'s 0.x.0-vs-0.x.y rule, that earns a minor bump.

## [0.5.2] — 2026-05-06

### Added — System-wide screen color picker (eyedropper)

- **The Color picker modal now has a "Pick from screen" button** that lets you sample a color from anywhere on the desktop, not just inside ClipSnap's own UI. The picked hex is automatically inserted into the modal — ready to copy as HEX / RGB / HSL. — *#feat(colors)*
  - **macOS:** uses Apple's own `NSColorSampler` (AppKit, 10.15+) — the same magnifier-loupe used by Pages, Keynote, and Sketch. Clicking outside the loupe cancels.
  - **Windows:** spawns a fullscreen layered overlay; click anywhere on screen to sample (`GetPixel` on the desktop DC). Press Esc to cancel.
  - **Async architecture.** The `pick_screen_color` IPC returns immediately; the result arrives later via the `color-picked` Tauri event with `string | null` payload. Keeps the UI responsive while the user is targeting their click.
  - New module `core/rust-lib/src/screen_picker.rs` (≈180 lines, fully `#[cfg(target_os = …)]`-gated). Adds `objc2` 0.6 + `block2` 0.6 as macOS-only deps for the Objective-C runtime calls; Windows reuses the existing `windows` 0.61 crate with extra features (`Win32_UI_WindowsAndMessaging`, `Win32_Graphics_Gdi`, `Win32_UI_Input_KeyboardAndMouse`).
  - **Tahoe quirk worth knowing.** macOS Tahoe's `NSColorSampler` only renders its loupe when the calling app is a *Regular* (Dock-visible) NSApplication. ClipSnap normally runs as `Accessory` (Dock-hidden tray app), so the picker briefly promotes the activation policy to Regular while the loupe is up, then demotes back 500 ms after the popup is restored. The popup itself stays visible during the pick — hiding it kills the loupe rendering ("no key window → no loupe").

### Docs

- README tagline updated to "Windows 11 & macOS"; previously said Windows 11 only.
- New / refreshed badges: separate Windows / macOS / Apple Silicon platform badges, plus Vite 7, ESLint flat-config, Vitest 3, cargo-test count, last-commit, repo-size, code-size, top-language.
- `docs/colors.md` rewritten end-to-end to describe the v0.5.x custom HSV modal, the click-to-select UX, and the screen eyedropper. The old "OS-native NSColorPanel / Win32 ChooseColor / GTK ColorChooser" copy was outdated since v0.5.0.

## [0.5.1] — 2026-05-06

### Fixed — Accessibility prompt fired on every paste

- **The actual root cause of "permission keeps re-prompting" is finally identified and fixed.** `enigo`'s `Settings::default()` ships with `open_prompt_to_get_permissions = true` on macOS — meaning every `Enigo::new()` call internally invokes `AXIsProcessTrustedWithOptions` *with the prompt option enabled*. So **every paste action on an untrusted process fired the standard "ClipSnap would like to control this computer" dialog as a side effect** — even though we just wanted to silently fall back. — *#fix(macos)*
  - **Fix:** new `enigo_settings()` helper in `paste.rs`, `expander.rs`, and `text_field/windows.rs` constructs `Settings { open_prompt_to_get_permissions: false, ..Settings::default() }`. Every `Enigo::new()` now uses it. enigo silently returns `NoPermission` when the process is untrusted; the dialog never fires as a paste-time side effect.
  - **Plus AX guard at the top of every paste IPC.** `paste_entry`, `paste_entry_formatted`, `paste_text`, `paste_snippet`, `paste_note`, `paste_note_formatted` all start with `require_accessibility()?` — short-circuits before even touching enigo and returns the structured `ax.permission_denied` error string to the frontend.
  - **Frontend toast.** `App.tsx` catches paste errors and renders an amber sticky banner: *"Paste failed — macOS Accessibility access not granted. Open the Settings tab and click Force re-grant…"* with an **Open Settings** button. Auto-dismisses after 8 s. The user finally has clear feedback instead of a silent failure or a recurring system dialog.
- **Live-debug methodology** documented in the commit history (kept in `git log` rather than the codebase): a temporary background AX-poller revealed that `AXIsProcessTrusted()` does *not* cache per-process on Tahoe — it re-queries TCC on every call. So our SettingsPanel polling has always been correct; the `ax.permission_denied` toast is the right user-facing complement.

### Changed — Color picker UX

- **Modal opens in a "no selection yet" state.** v0.5.0 default-filled the picker with `#3366FF` so the toolbar-button click felt like it had already selected a color. Now the modal opens with: empty hex input, dashed-border placeholder swatch reading "Click in the picker above (or type a hex) to select a color", and Copy disabled. **The first click in the SV picker is the selection** — matching the user's mental model of "1st click opens, 2nd click selects". — *#fix(colors)*
  - SV-picker crosshair indicator hidden until first click.
  - Hue-slider drag and hex-input typing also count as "selection" once the user engages with them.
  - Closing & re-opening the modal resets to the no-selection state.

## [0.5.0] — 2026-05-05

### Added — 25 default AI prompt snippets, working color picker

- **Bundled default snippet library — 25 curated AI prompts.** First-launch seeds your snippet table with `ai*`-prefixed prompts covering programming (`aiplan`, `aireview`, `airefactor`, `airegex`, `aisql`, `aitest`, `aimigration`, `aibench`), web/frontend (`aithumb`, `aimobile`, `aia11y`, `aiseo`, `aicomponent`), IT security (`aithreat`, `aipentest`, `aiauth`, `aigdpr`), business workflows (`aibrief`, `airfp`, `aiokr`, `aichange`), data analysis (`aidataq`, `aiml`, `aidashboard`), and architecture (`aiapi`). Each prompt is a structured, opinionated brief — sections, bullets, output-format directives — written to be handed straight to an LLM without further massaging. Type the abbreviation in the search field, press Enter (or use the text expander), get the full prompt. — *#feat(snippets)*
  - **Idempotent seeding.** Tracked via `seed.default_snippets_v1` in the settings table. Runs once on first install; user-deleted prompts stay deleted on subsequent launches.
  - **Restore defaults button** in the Snippets-tab sidebar (rotate-counter-clockwise icon, next to Import). Re-imports all 25 prompts, upsert-by-abbreviation — your custom snippets with different abbreviations are untouched, but a deleted/edited `aiplan` *is* reset to the bundled version.
  - Embedded via `include_str!` so no external file is needed at runtime.
  - 3 new Rust unit tests (`embedded_json_parses_and_has_25_prompts`, `maybe_seed_inserts_on_first_run_and_skips_after`, `restore_defaults_re_imports_explicitly`).
- **Working cross-platform color picker.** v0.4.0's HTML5 `<input type="color">` was unreliable in WKWebView (Tauri's macOS renderer) — the OS picker often didn't open, and even when it did, `navigator.clipboard.writeText` got blocked because the `change` event fires outside the user-gesture context. Replaced with a **custom modal** that runs entirely in the WebView. — *#fix(colors)*
  - Hue slider + 2D saturation/value picker + live hex input + format tabs (HEX/RGB/HSL) + WCAG-readable preview swatch + Copy button.
  - Clipboard write goes through `@tauri-apps/plugin-clipboard-manager`'s `writeText` (no browser-API restrictions).
  - Esc / backdrop-click closes; copy feedback flashes "Copied!" for 2s.
  - Capabilities updated: `clipboard-manager:allow-write-text` added to both `macos/src-tauri/capabilities/default.json` and `win/src-tauri/capabilities/default.json`.

### Why 0.5.0 (not 0.4.3)

The 25-prompt seed is a real new feature surface, AND first-run behavior changes (new users automatically get a populated snippet library — that's an opinion, not a fix). Bumping minor signals it.

### Tests

`cargo test --workspace`: **84 → 87 green** (+3 seed). `pnpm test`: **77 → 85 green** (+8 HSV/HSL/hex helpers).

## [0.4.2] — 2026-05-05

### Fixed

- **No more duplicate history entries from plain-text paste.** v0.4.0's plain-text-paste downgrade for HTML / RTF clips was leaking back into the watcher: ClipSnap wrote the plain-text version of an HTML clip to the OS clipboard → the clipboard watcher saw the change → recorded a *new* Text-type entry `just now`, sitting next to the original HTML clip from earlier. Hash-based dedup didn't catch it because `hash(Html, "<p>foo</p>") ≠ hash(Text, "foo")`. — *#fix(watcher)*
  - **Fix:** `WatcherState` gets a one-shot `self_written: Mutex<Option<String>>` fuse holding the SHA-256 of the most recent payload we wrote ourselves. The watcher checks this hash before storing and consumes-and-skips any matching event. Every paste IPC (`paste_entry`, `paste_entry_formatted`, `paste_text`, `paste_snippet`, `paste_note`, `paste_note_formatted`) calls `watcher.mark_self_write(content_type, payload)` immediately before triggering the OS clipboard write. Net effect: pasting from history never creates a duplicate entry, regardless of the plain-text setting.
- **Macros prompt no longer fires as an unwanted side effect.** When `expand_at_cursor` (hotkey trigger) or `diagnose_at_cursor` (Test button) call `AXUIElementCopyAttributeValue` on the system-wide element while ClipSnap is **untrusted** (typical post-rebuild stale-cdhash state), macOS triggers the standard "would like to control this computer" prompt as a side effect — even when we just want to silently fall back to the clipboard path. — *#fix(macos)*
  - **Fix:** both functions now check `accessibility_granted()` *before* calling any AX function. When `false`, they go straight to the clipboard fallback (or return an empty diagnose result), and the macOS prompt isn't triggered as a no-op cost. The Settings panel's amber banner + **Force re-grant** button remain the right place to surface the underlying permission issue.

## [0.4.1] — 2026-05-05

### Changed

- **`paste_note` now respects `paste.plain_text_only`.** v0.4.0 added the plain-text-paste toggle for clipboard history, but notes (a separate paste path via `paste_note`) kept their old behaviour — HTML / RTF notes always pasted with formatting. The user's original ask was "always plain text in all OSes" which implicitly covers notes too. Now: HTML / RTF notes get downgraded to their plain-text preview when the toggle is on; image / files notes remain unaffected. — *#fix(paste)*
- New `paste_note_formatted` IPC command mirrors `paste_entry_formatted` — bypasses the setting and uses the note's original content type. Wires up symmetrically; the NotesPanel UI doesn't surface a Shift+click override yet but the IPC is ready when we add one.

### Docs

- `docs/notes.md` paste-behaviour table updated to call out which content types respect the plain-text-only toggle and which are unaffected.

## [0.4.0] — 2026-05-05

### Added — Plain-text paste, hex color preview, color picker

- **Plain-text paste mode (default on).** Settings → Paste section gets a new toggle. When on, HTML and RTF clipboard entries are stripped to their plain-text preview at paste time — so copy-from-Word / browser / mail and paste-into-anything no longer leaks the source app's font / colour / hyperlink styling. The original formatted content is preserved in the history (preview pane still renders it; the type icon still shows HTML / RTF), only the *paste action* downgrades. Image / Files entries are unaffected. — *#feat(paste)*
  - **Per-row override:** hold <kbd>Shift</kbd> while pressing <kbd>Enter</kbd> in the popup to paste *with* original formatting, regardless of the toggle. New IPC `paste_entry_formatted` bypasses the setting; `useKeyboardNav` forwards `event.shiftKey` to the activate handler.
  - Backend: `paste.plain_text_only` setting key (default `true`); `paste_entry` reads it and routes Html / Rtf entries to `paste::paste_text(content_text)`. `paste_entry_formatted` always uses `paste::paste_entry` for original-content-type behaviour.
- **Inline hex color preview** in the search input — Alfred-style. — *#feat(colors)*
  - Type `#3366FF` (or `3366FF`, `#abc`, `#abcdef12`, …) and a color row appears as the top list item with a swatch + hex + RGB summary. Press <kbd>Enter</kbd> to paste the canonical `#RRGGBB` (uppercase) into the previously focused app.
  - Heuristic: 3 / 4-digit forms require the `#` prefix (too ambiguous with search otherwise — `abc`, `f00d`, …); 6 / 8-digit forms accept either form.
  - Preview pane shows a full 128 px swatch with the hex overlaid (foreground auto-picked black/white via WCAG luminance for readability), plus copy-to-clipboard buttons for hex / `rgb(…)` / `hsl(…)` strings.
  - Pure frontend (`core/frontend/src/lib/colors.ts`); 24 vitest cases covering valid / invalid / canonicalisation / RGB-HSL conversion / readable-foreground.
- **OS-native color picker** — new "Color picker" button in the History tab's toolbar. Opens an `<input type="color">` which Tauri renders via the OS-native picker (NSColorPanel on macOS, Win32 ColorDialog on Windows, GTK ColorChooser on Linux). The chosen hex (uppercase) is written to the system clipboard via the Web Clipboard API; the watcher captures it as a fresh history entry within the next event tick. — *#feat(colors)*

### Changed

- `App.tsx` activate handler: signature changes to `activate(i, shiftKey)`. Color-row activation pastes the canonical hex via the existing `paste_text` command. Calc-row activation unchanged.
- `useKeyboardNav.onEnter` callback signature is now `(shiftKey: boolean) => void`.
- `HistoryItem` and `PreviewPanel` learn a fourth row kind (`color`) alongside clip / snippet / calc.
- `ListEntry` discriminated union gains `{ kind: "color"; data: ColorEntryView }`.

### Tests

`pnpm test`: **53 → 77 frontend** (+24 colors tests). `cargo test --workspace`: 84 unchanged (paste-plain-text logic exercises through existing paste tests; the wiring is straightforward enough that integration testing is overkill here).

### Why 0.4.0 (not 0.3.2)

Plain-text-paste-by-default is a **behaviour change**: clipboard entries that *used* to paste with formatting now arrive as plain text, by default, without the user opting in. That's a semver-meaningful flip. Two new user-facing features (hex preview, color picker) compound it. Bumping minor signals the change.

## [0.3.1] — 2026-04-29

### Fixed

- **macOS Accessibility prompt loop after rebuilds.** Common state after a real source-change install: the toggle in System Settings → Accessibility shows ClipSnap as **enabled**, but ClipSnap still asks for permission on every hotkey press. Cause: the toggle's underlying TCC entry is bound to the *previous* binary's cdhash; the new build has a different cdhash and is treated as a new app. The toggle UI just reports the bundle id, which masked the discrepancy.
  - **Fix:** new **Force re-grant (clear stale)** button in the amber Accessibility banner. Shells out to `tccutil reset Accessibility io.celox.clipsnap` + `tccutil reset PostEvent io.celox.clipsnap` (no sudo needed for the user's own bundle), then fires `AXIsProcessTrustedWithOptions(prompt: true)` so macOS re-adds ClipSnap to the Accessibility list with the *current* cdhash. Toggling on again creates a TCC entry that matches what the running process actually is. — *#fix(macos)*
  - The legacy "Try system prompt" button stays as a secondary option (for the rare cases where the entry is sane and just needs a re-prompt).
- New IPC command `force_reset_and_request_grant` (macOS-only meaningful behaviour; no-op elsewhere). Backend in [`core/rust-lib/src/expander.rs`](./core/rust-lib/src/expander.rs); wrapper in [`core/frontend/src/lib/ipc.ts`](./core/frontend/src/lib/ipc.ts).

## [0.3.0] — 2026-04-28

### Added — Accessibility-first text expander

- **The text expander now reads the focused field directly via the OS accessibility layer** instead of synthesising `Cmd/Ctrl+Shift+←` + `Cmd/Ctrl+C` as the *primary* path. macOS uses **`AXUIElement`** (ApplicationServices), Windows uses **`IUIAutomation`** (UIAutomationCore). Same Accessibility permission already required for paste; no new permission added. Native FFI — no objc2/winRT macros needed. — *#feat(expander)*
  - **Why it matters:** the keystroke approach works in 90 % of apps but breaks in terminals (iTerm2, kitty, gnome-terminal — they reinterpret `Cmd/Ctrl+Shift+←` as pane-switch / mark-selection), web apps with custom keyboard handlers (Google Docs, online IDEs), and password fields. The accessibility approach succeeds wherever the focused element exposes its value to assistive tech — which is essentially every text field a screen reader can read.
  - **No more clipboard touch on the happy path.** When AX/UIA succeeds the user's clipboard is left completely untouched and there's no visible selection flicker.
  - **Clipboard fallback retained.** When the focused element doesn't expose the necessary attributes (rare native Carbon, Java/Swing without AccessBridge), ClipSnap falls back to the previous keystroke + clipboard roundtrip seamlessly.
- **`text_field` module** — new abstraction in [`core/rust-lib/src/text_field/`](./core/rust-lib/src/text_field/):
  - `mod.rs` — `FieldAccess` trait + `CapturePath { Ax, Uia, Clipboard }` enum + UTF-16 ↔ char-index helpers + the platform-agnostic `word_start_before_cursor` algorithm. 7 unit tests covering ASCII, German umlauts, emoji (supplementary plane), cursor past end, whitespace-only.
  - `macos.rs` — raw FFI to `AXUIElementCreateSystemWide` / `AXUIElementCopyAttributeValue` / `AXUIElementSetAttributeValue` for the three attributes that matter: `AXFocusedUIElement`, `AXValue`, `AXSelectedTextRange`. UTF-16 helpers because AX reports cursor positions in UTF-16 code units. 3 unit tests.
  - `windows.rs` — `windows` crate bindings to `IUIAutomation`, `IUIAutomationTextPattern`, `IUIAutomationTextRange`. Uses UIA for the *read* (reliable) but deliberately uses Backspace×N + `enigo.text(body)` for the *write*, because UIA's `IUIAutomationTextEditPattern2::Replace` is patchily implemented across real-world Windows controls.
- **`Capture path` row in the Diagnose UI** — Settings → *Text expander* → Diagnose now shows whether the run used `macOS AX (clean — no clipboard touch)`, `Windows UIA (clean — no clipboard touch)`, or fell back to the `Clipboard fallback` path. Lets you tell at a glance whether the app you're testing in has working accessibility.

### Changed

- `expander::expand_at_cursor` and `expander::diagnose_at_cursor` now try AX/UIA first; the legacy clipboard roundtrip is the second-choice fallback. The fallback path can also be invoked with prefetched abbreviation+body so the lookup isn't repeated when AX read succeeded but AX replace didn't.
- `core/rust-lib/Cargo.toml` — added `windows = { version = "0.61", features = ["Win32_Foundation", "Win32_System_Com", "Win32_UI_Accessibility"] }` as a `target.'cfg(target_os = "windows")'` dependency. macOS / Linux builds don't pull it in.
- **`DiagnoseResult`** gains a `path: "ax" | "uia" | "clipboard"` field. Frontend `ipc.ts` interface updated to match.

### Why bump to 0.3.0

This is a real architecture change for the expander — the keystroke path is no longer the default. Bumping the minor signals that the failure modes (and therefore the user-visible behaviour) shift. The fallback path keeps full backward compatibility — every app that worked in 0.2.x still works in 0.3.0, just often via a cleaner mechanism.

### Tests

`cargo test --workspace`: **74 → 84 green** (+7 word-boundary, +3 UTF-16). `pnpm test`: 53 unchanged.

## [0.2.12] — 2026-04-28

### Changed

- **Backup Export / Import moved to the Settings tab.** Lived under the Notes tab's sidebar since v0.2.6, but conceptually belonged with the rest of the app-level configuration. The Notes tab keeps **+ New Note** and **Clear All**; everything backup-related is now under the new **Backup & restore** section in Settings. — *#feat(settings)*
- **Selective export.** Three checkboxes — *Clipboard history*, *Snippets*, *Notes* — let you choose which sections land in the file. All checked by default; unchecking any of them writes an empty array for that section in the JSON. Intended use: share snippets without leaking your clipboard history.
  - Backend: new `backup::ExportOptions { include_history, include_snippets, include_notes }` with `::all()` / `::default()` constructors. Both `export_backup` and `save_backup_to_file` IPC commands take three optional flags (default `true`). Existing callers stay backward-compatible.
  - Frontend: `BackupExportOptions` interface in `ipc.ts`. `exportBackup()` / `saveBackupToFile(path, opts)` accept the same fields.
  - 3 new Rust unit tests (`export_with_only_snippets…`, `export_with_all_off…`, `export_options_default…`). Backend total: 71 → **74 green**.

### Fixed

- After an Import, the Notes / Snippets / History tabs now refresh immediately. The Settings panel takes an `onBackupImported` prop from `App.tsx` that re-fires the three list hooks (`refreshHistory`, `refreshSnippets`, `refreshNotes`) once the merge returns.

## [0.2.11] — 2026-04-26

### Fixed

- **Crash on hotkey / Test now: `EXC_BREAKPOINT` from `_dispatch_assert_queue_fail`.** The text-expander dispatched `enigo` work onto a worker thread (`std::thread::spawn` in `register_expander`, plus the IPC handler thread for `trigger_expand_at_cursor` / `diagnose_expand_at_cursor`). On macOS, enigo's `Key::Unicode(...)` mapping calls `TSMGetInputSourceProperty` (Text Services Manager) which **asserts main-thread**. Calling it from any other thread fires a libdispatch assertion and aborts the process with SIGTRAP. Confirmed by three crash reports today: `clipsnap-2026-04-26-070927.ips`, `…-070931.ips`, etc — all ended at `enigo::macos_impl::keycode_to_string` from a worker thread.
  - **Fix:** all three call sites now dispatch the expand cycle to the main thread via `AppHandle::run_on_main_thread`. The hotkey path is fire-and-forget; `diagnose_expand_at_cursor` ferries the result back through an `mpsc::channel`. The popup is hidden during the cycle, so the ~290 ms main-thread block is invisible to the user.

## [0.2.10] — 2026-04-26

### Fixed

- **macOS Accessibility re-grant loop is finally broken.** Real root cause this time, not symptoms: macOS Tahoe (26.x) binds the TCC Accessibility grant to the tuple `(bundle id, cdhash)`. `scripts/install-macos.sh` previously ran `codesign --force` on every install — even when the user re-installed an *unchanged* binary — which embedded a fresh CMS timestamp into the signature blob and produced a new cdhash. macOS then dropped the prior grant, prompting again. — *#fix(macos)*
  - **Idempotent install.** The script now SHA-256 compares the freshly built binary at `target/release/bundle/macos/ClipSnap.app/Contents/MacOS/clipsnap` against the currently installed binary at `/Applications/ClipSnap.app/Contents/MacOS/clipsnap`. If they're identical (and the bundle identifier already matches), the script **skips both `cp` and `codesign`** entirely — your install is preserved verbatim, the cdhash stays stable, and your TCC grant survives. Net effect: rebuilds without source changes never ask you to re-grant.
  - **Cleaner re-sign output.** When source *did* change, the script now prints both old and new SHA-256 prefixes plus the resulting cdhash, with an explicit "TCC grant must be re-given" warning so you know what to expect.
- **Wrong entitlement removed.** `com.apple.security.automation.apple-events` was misleadingly attached "for enigo to simulate paste" but actually covers AppleScript automation (NSAppleEvent / OSAScript), not `CGEventPost`-style synthetic input. Worse, on macOS Tahoe with Hardened Runtime its presence can trigger an unrelated TCC "Automation" prompt and confuse the permission flow. Removed from `macos/src-tauri/entitlements.plist`. The remaining three entitlements (`allow-jit`, `allow-unsigned-executable-memory`, `disable-library-validation`) correctly cover WebKit / Tauri plugin loading.

### Added

- **Auto-restart prompt after grant detected.** The Settings panel's polling loop now distinguishes the false→true transition of `accessibility_granted`. When it fires, an inline emerald-bordered prompt appears: **"Access detected — one more step"** with a **Restart now** button. Click → ClipSnap spawns a fresh `/Applications/ClipSnap.app` process via `open -n` and exits cleanly. The new instance picks up the just-granted TCC state correctly (the running process couldn't, because macOS caches `AXIsProcessTrusted()` per-process). Total post-grant flow: ~30 seconds, one click. — *#feat(settings)*
  - New `relaunch_app` IPC command in `core/rust-lib/src/commands.rs`.
  - `relaunchApp()` wrapper in `core/frontend/src/lib/ipc.ts`.
- **"Why does this keep happening?" disclosure** in the amber banner of the Settings panel, explaining the cdhash binding in plain language so users understand the constraint instead of feeling gaslit by the OS.

### Changed

- **`[profile.release]`** at the workspace root: `codegen-units = 1`, `lto = true`, `strip = "debuginfo"`, `opt-level = 3`. Won't make Rust release builds fully byte-reproducible, but reduces non-determinism so the SHA-256 idempotency check has a fighting chance for trivial source changes.
- **`scripts/install-macos.sh`** — full restructure with helper functions (`bin_sha256`, `cdhash`, `current_identifier`, `kill_running`, `resign_app`, `reset_tcc`) and clearer printed status. The script's docstring at the top now accurately describes the cdhash binding and how the idempotent path works.
- **`macos/README.md`** "Why the dialog re-appears" section rewritten with the honest truth instead of the previous wishful "Sequoia and earlier accept this; later releases may still re-prompt." Now says: every meaningful rebuild requires re-grant on Tahoe; the script + auto-restart prompt make it bearable; the only permanent fix is an Apple Developer ID.

### Verification recipe

```bash
# 1) idempotent rebuild preserves grant
bash scripts/install-macos.sh        # initial install
# … grant Accessibility once via Settings panel banner …
bash scripts/install-macos.sh        # re-run with no source changes
#   ⇒ prints "Binary unchanged — keeping existing install"
#   ⇒ green banner stays green; Diagnose works without intervention

# 2) source change triggers single re-grant
echo "// touch" >> core/rust-lib/src/lib.rs
bash scripts/install-macos.sh
#   ⇒ prints "Binary changed — full reinstall"
#   ⇒ amber banner appears in Settings tab
#   ⇒ click Open System Settings → enable toggle → switch back
#   ⇒ green "Restart now" prompt appears within 1 s
#   ⇒ one click → app relaunches → Diagnose works
```

## [0.2.9] — 2026-04-26

### Added

- **Accessibility status badge in the Settings panel** — green when ClipSnap has macOS Accessibility access, amber when it doesn't, with an inline explainer of what to do. Polled once per second while not granted, so the badge flips to green within ~1 s of the user toggling ClipSnap on in System Settings — no panel reload needed. — *#feat(settings)*
- **`Test now` button** in the Settings panel — runs the full expand-at-cursor cycle without using the hotkey after a 2-second grace period (long enough to switch back to the source app and place the cursor after an abbreviation). Lets you tell whether the *hotkey* is the problem or the *expansion logic* is. Wired through the existing `trigger_expand_at_cursor` IPC.
- **`get_accessibility_status` Tauri command** + `ExpanderConfig.accessibility_granted` field — backed by macOS `AXIsProcessTrusted()` via FFI to `ApplicationServices.framework`. Returns `true` unconditionally on Windows / Linux, where synthetic input is either ungated or gated by a different mechanism.

### Fixed

- **`scripts/install-macos.sh`** — new helper that builds + re-signs ClipSnap with a stable ad-hoc identifier (`io.celox.clipsnap`) before copying into `/Applications`. Without an Apple Developer ID, every fresh `pnpm build:macos` produced a *random* identifier (e.g. `clipsnap-c64f925d…`); macOS TCC then treated the rebuild as a brand-new app and discarded the previous Accessibility grant. The script's stable identifier lets the grant survive across rebuilds (where macOS allows bundle-id matching), and `--reset` runs `tccutil reset` to wipe stale carcass entries when needed.
- **macOS README** — new "Why the dialog re-appears after every rebuild" section explaining TCC binding to code-signature, plus how to use `install-macos.sh`.

## [0.2.8] — 2026-04-26

### Fixed

- **Expander hotkey capture failed for the `^` key on German ISO macOS keyboards.** WebKit reports the top-left key (`^`/`°`) as `event.code = "IntlBackslash"`, but the Tauri `tauri-plugin-global-shortcut` parser (`Shortcut::from_str`) maintains a hand-written allow-list that doesn't include any `Intl…` codes — the captured combo `Alt+IntlBackslash` was rejected with `UnsupportedKey("IntlBackslash")`. Two-part fix: — *#fix(expander)*
  - **Frontend** (`HotkeyCapture.tsx`) — new `normalizeCode()` maps WebKit's `IntlBackslash` back to `Backquote` (the layout-stable W3C name; same Carbon virtual keycode `kVK_ANSI_Grave` = 0x32 the OS will see at hotkey time).
  - **Backend** (`hotkey::parse_shortcut`) — replaces the plugin's narrow parser with our own. Routes the code token through `keyboard_types::Code::from_str`, which understands the **full** W3C `KeyboardEvent.code` spec. Future-proofs against other gaps in the plugin's allow-list (`IntlBackquote`, `IntlRo`, `IntlYen`, less-common media keys, …).
  - 9 new unit tests for the parser (modifier aliases, `IntlBackslash` accept, single-key, error cases). Backend tests: 62 → **71 green**.
- **HotkeyCapture button never recorded on macOS.** Safari/WebKit does **not** focus a `<button>` on click, so the button-level `onKeyDown` never fired. The capture indicator stayed at "Press a key combination…" forever. Fix: while capturing, attach a window-level keydown listener in *capture phase* — wins over the global keyboard-nav hook (which would otherwise consume Esc as "close popup"). — *#fix(settings)*
- **Search bar placeholder + Notes/Snippets/Settings titles ran behind the absolutely-positioned tab strip.** With four tabs (after Settings was added in 0.2.7) the strip overlapped the input. Fix: reserve `pr-[260px]` on the search bar and on the inactive-tab title row, tighten tab buttons to `px-2 whitespace-nowrap`, shorten the placeholder to `Search or calculate…`. — *#fix(ui)*

### Added

- **Per-row delete + Clear all** for clipboard history. Hover any clip row in the History tab → trash icon appears next to the bookmark icon → one click removes that single entry. A new toolbar at the top of the history list shows the clip count and a **Clear all** button (with `window.confirm` guard) for nuking everything at once. Wired through the existing `delete_entry` / `clear_history` IPC commands. — *#feat(history)*

### Changed

- `useClipboardHistory` now exposes its `refresh` callback to `App.tsx` so the list refetches immediately after delete/clear-all instead of waiting for the next `clipboard-changed` event.

## [0.2.7] — 2026-04-25

### Added

- **System-wide text expander.** Type a snippet abbreviation in any text field — code editor, browser, mail client, Slack — then press the configured hotkey, and ClipSnap replaces the abbreviation in place with the snippet body. Default hotkey is `Alt+Backquote` (the `^` key on a German keyboard, ` on US). Disabled by default — opt in from the new **Settings** tab. — *#feat(expander)*
  - **How it works:** the popup stays out of the way. ClipSnap synthesizes `Cmd/Ctrl+Shift+←` (select previous word) → `Cmd/Ctrl+C` (copy), looks the captured word up in the snippets table via the new `find_by_exact_abbreviation` (case-sensitive first, case-insensitive fallback), writes the body to the clipboard, and synthesizes `Cmd/Ctrl+V`. The user's clipboard is saved before the cycle and restored after.
  - **Trigger semantics, not silent watch.** No global keylogger — you decide when to expand.
  - **Configurable hotkey.** New **Settings** tab → click the hotkey field → press your combination (Backspace clears, Esc cancels). The string is stored in the new `settings` SQLite table and re-registered with the OS via `tauri-plugin-global-shortcut`. Bad combinations are rejected before the previous registration is touched, so you can't accidentally lose your hotkey to a typo.
  - **Cross-platform.** macOS / Windows / Linux X11 work the same. Linux Wayland depends on the compositor's global-shortcut portal (GNOME/KDE OK; sway-flavoured stacks may not).
  - Full reference: [`docs/text-expander.md`](./docs/text-expander.md).
- **Settings tab** in the popup, alongside History · Snippets · Notes. Designed to grow — first home for the expander toggle + hotkey capture; future settings (capture pause defaults, image-size cap, …) will land here.
- **`settings` SQLite table** — new key/value store via `core/rust-lib/src/settings.rs`. Idempotent migration; created on first launch of v0.2.7.
- **`HotkeyCapture` React component** that converts a `KeyboardEvent` into the W3C-code shortcut format the global-shortcut plugin's parser expects (`Modifier+...+Code`).
- **14 new Rust unit tests** — settings store roundtrip (6), `snippets::find_by_exact_abbreviation` semantics (5), expander helpers (3). `cargo test --workspace`: 48 → **62**.

### Changed

- IPC surface gains `get_expander_config`, `set_expander_config`, `trigger_expand_at_cursor`. The latter is a programmatic alternative to the hotkey — useful for testing and for any future tray-menu entry.
- `hotkey.rs` gains `ExpanderShortcutState` (Tauri-managed) and `register_expander(...)`, which idempotently swaps the previously-registered expander shortcut. Runs the actual expansion on a worker thread so the global-shortcut callback returns instantly (avoids platform-specific deadlocks).

### Caveats — what *won't* work cleanly

These are documented in [`docs/text-expander.md`](./docs/text-expander.md), surfaced in the Settings panel's "How it works" disclosure:

- **Terminals** (iTerm2, kitty, gnome-terminal) sometimes interpret `Cmd/Ctrl+Shift+←` as a pane-switch / mark-selection — the expander may grab the wrong "word" or nothing at all.
- **Password fields** in many apps refuse synthetic paste; the abbreviation gets selected but the body never lands.
- **Linux Wayland** in restrictive compositors blocks global shortcuts entirely.
- **Image / files snippets** are not supported by the expander (the orchestration only handles text). This is intentional for v1.

## [0.2.6] — 2026-04-25

### Added

- **Notes — a third tab for persistent, categorized clipboard items.** Notes live in their own SQLite table and are *not* affected by the 1 000-entry pruning of the clipboard history, so they're the right place for things you want to keep. — *#feat(notes)*
  - Three-pane layout: **Categories sidebar** (with note counts per category, plus virtual `All` and `Uncategorized` groups), **note list**, and **detail/edit pane**.
  - **Free-form categories** — typing a new category name in the edit form auto-creates it; the input has a `<datalist>` for autocomplete from existing categories.
  - **Editable bodies** for `text`, `html`, `rtf` notes; `image` and `files` notes are read-only (you can still rename them and change category). The detail pane renders images inline and shows file paths as a list.
  - **Paste from a note** preserves the original content type — image notes paste as images, HTML notes paste as HTML, etc.
- **Star button on history rows** — hover any clipboard entry in the History tab and the bookmark icon appears next to the timestamp; one click promotes the entry to a note in the `Uncategorized` bucket. The note is decoupled from the clip thereafter, so even if the clip gets pruned out of history, the note stays.
- **Full-app backup** — Notes tab toolbar gets `Export…` and `Import…` actions wired through `tauri-plugin-dialog`. Export writes a single pretty-printed JSON file (`{ version, exported_at, history, snippets, notes }`); import merges that file back into the live database with sensible per-table semantics:
  - **Snippets** — upsert by `abbreviation` (existing rows are overwritten).
  - **History** — upsert by SHA-256 hash; duplicates just bump `last_used_at`, new rows respect the existing 1 000-entry cap.
  - **Notes** — appended verbatim with original timestamps preserved (no natural dedup key, so re-importing the same backup creates duplicates — use Clear All first if you want a clean replace).
- **`Clear All` for notes**, with a `window.confirm` guard.
- **Tray menu entry “Manage Notes”** — opens the popup directly on the Notes tab via a new `open-notes-tab` event.
- **15 new Rust unit tests** for the notes module (CRUD, categories, save_from_clip, image-note read-only update) and the backup module (roundtrip into empty db, merge into populated db, version-rejection guard, replace-all). `cargo test --workspace` is now **48 → was 33**.

### Changed

- `paste.rs::write_to_clipboard` was refactored to take primitives `(content_type, data, text)` instead of a `&ClipEntry`, exposed via the new public `paste::paste_payload(...)`. This lets the `paste_note` IPC command paste any content type without needing to construct a fake `ClipEntry`.
- New IPC commands wired into `invoke_handler`: `list_notes`, `list_note_categories`, `save_clip_as_note`, `create_note`, `update_note`, `delete_note`, `clear_notes`, `paste_note`, `export_backup`, `save_backup_to_file`, `import_backup`.
- New permissions in both shells' `capabilities/default.json`: `dialog:allow-save` (for the export file picker).

### Database

- New table on first launch (idempotent `CREATE TABLE IF NOT EXISTS`):
  ```sql
  CREATE TABLE notes (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      content_type TEXT NOT NULL,
      content_text TEXT NOT NULL DEFAULT '',
      content_data TEXT NOT NULL DEFAULT '',
      title        TEXT NOT NULL DEFAULT '',
      category     TEXT NOT NULL DEFAULT '',
      byte_size    INTEGER NOT NULL DEFAULT 0,
      created_at   INTEGER NOT NULL,
      updated_at   INTEGER NOT NULL
  );
  ```
  Indexed on `category` and `updated_at DESC`.

## [0.2.5] — 2026-04-25

### Added

- **Inline calculator in the search field** — Alfred-style. As you type, ClipSnap evaluates the input as a math expression and shows the result as the top list item; press Enter to paste the result into the previously active app. Bare numbers (`42`) and plain text (`hello`) are ignored; only inputs with at least one operator, function call, or named constant trigger calc mode. A leading `=` forces evaluation (so `=42` or `=pi` displays a result for a single literal). — *#feat(calc)*
  - Supported operators: `+ - * / % ^` (power is right-associative), unary `+`/`-`, parens.
  - Supported numbers: integers, decimals (`0.5`, `.5`), scientific (`1e3`, `1.5e-2`), digit grouping (`1_000`).
  - Constants: `pi` / `π`, `tau`, `e`.
  - Functions: `sqrt`, `cbrt`, `abs`, `sign`, `floor`, `ceil`, `round`, `ln`, `log` (base 10), `log2`, `exp`, `sin`/`cos`/`tan` (radians), `asin`/`acos`/`atan`/`atan2`, `sinh`/`cosh`/`tanh`, `min`, `max`, `pow`, `mod`.
- **`paste_text(text)` Tauri command** — generic "compute & paste" entry point used by the calculator (and available for future flows like unit-conversion / date-math). Hides the popup, writes `text` to the clipboard, and synthesizes Cmd+V / Ctrl+V via `enigo`, same as the existing snippet-paste path.
- **27 new vitest cases** for `tryEvaluate` and `formatResult` covering precedence, right-associative power, parens, decimals + scientific notation, every supported function/constant, `=`-forced evaluation, and rejection of plain numbers / malformed input. (`pnpm test`: 24 → 51 frontend tests.)

### Changed

- **Search field rebranded as a general input.** Placeholder is now `Search history or type an expression (2+2, sqrt(16), …)`. The leading icon is a chevron by default and switches to a calculator glyph the moment the input parses as a math expression — making the field read as an entry box, not just a search box.
- New `CalcEntry` variant in `ListEntry`; `HistoryItem` renders calc rows with a `calc` chip and `expr = result` formatting in monospace, `PreviewPanel` shows a centered large `= result` view.

## [0.2.4] — 2026-04-25

### Fixed

- **Paste did not land in the previously active app on macOS.** Hiding only the popup window left ClipSnap (an `Accessory`-policy app) in a state where the OS could not reliably hand key focus back to the prior frontmost app, so `enigo`'s synthesized `Cmd+V` either dropped on the floor or arrived back at ClipSnap. — *#fix(paste)*

### Changed

- `hotkey::hide_popup` now also calls `AppHandle::hide()` on macOS (no-op on other platforms), which invokes `NSApplication.hide(nil)` and forces the OS to restore the prior frontmost app as key window. The popup window is hidden first, then the app.
- The settle delay between clipboard write and the synthesized paste keystroke is now platform-specific: **120 ms on macOS** (was 50 ms — `NSApp.hide()` takes a frame or two), unchanged 50 ms on Windows / Linux.

## [0.2.3] — 2026-04-25

### Fixed

- **Import button appeared to crash the app on macOS.** When the native file dialog (`NSOpenPanel`) opened, the popup window lost focus, which fired our existing `Focused(false)` window event → `hide_popup()` ran → the popup vanished. The dialog often stayed half-up but with its parent gone, the user perceived the whole app as having crashed. — *#fix(snippets)*

### Added

- New `UiState { suppress_hide: AtomicBool }` shared state and IPC command `set_suppress_hide(suppress: bool)`. The Snippets-tab Import handler now wraps the `dialog.open()` call in `setSuppressHide(true) … finally setSuppressHide(false)` so the popup stays put while NSOpenPanel owns focus.
- `core/rust-lib/src/ui_state.rs` — new module owning the shared UI flag.

### Changed

- The popup's `Focused(false)` handler in `lib.rs` consults the suppress flag before calling `hide_popup`. Default behaviour (auto-hide on click-outside, Esc, alt-tab) is unchanged.

## [0.2.2] — 2026-04-25

### Fixed

- **JSON snippet import was broken on macOS.** The 0.2.1 implementation used a hidden `<input type="file">` triggered by `.click()` from React. WKWebView (Tauri's macOS renderer) does not reliably surface a native file picker for hidden inputs in this pattern, so the Import button appeared to do nothing on macOS. — *#fix(snippets)*

### Changed

- **Switched the snippet-import file picker to `tauri-plugin-dialog`.** The Import button now opens the native NSOpenPanel / Win32 OpenFileDialog via `@tauri-apps/plugin-dialog`'s `open()`, with a `.json` filter and a localized "Select snippets JSON file" title. Selected path is read in Rust (`std::fs::read_to_string`) and parsed by the existing `import_from_json` pipeline.

### Added

- New IPC command `import_snippets_from_file(path: String) -> ImportResult` (in addition to the existing `import_snippets(json: String)` which is still used by tests).
- `tauri-plugin-dialog` workspace dep + capability permission `dialog:allow-open` in both the Windows and macOS shells.
- Import button shows "Importing…" while the dialog/import is in flight.
- **5 themed example JSON files** under `docs/examples/snippets/` — `getting-started.json` (3 entries), `signatures.json` (4), `dev.json` (8), `markdown.json` (5), `wrapped-form.json` (2, demonstrates the `{ snippets: [...] }` shape). Each is a stand-alone, ready-to-import file; the folder has its own `README.md` indexing them and showing how to merge multiple files via `jq -s 'add'`.
- `docs/snippets-import.md` extended with a Tips & anti-patterns section.
- Root `README.md` Snippet-import section now lists all example files in a table instead of a placeholder code block.

## [0.2.1] — 2026-04-25

### Added

- **JSON snippet import** — bulk-load snippets from a `.json` file via **Snippets → Import** in the popup. Existing abbreviations are upserted in place, so re-importing the same file is idempotent. Both `[…]` (bare array) and `{ "snippets": [...] }` (wrapped) shapes are accepted; per-row failures are collected in the result without aborting the whole import. See [`docs/snippets-import.md`](./docs/snippets-import.md) for the schema and [`docs/snippets-example.json`](./docs/snippets-example.json) for a sample. — *#feat(snippets)*
- **`macos/README.md`** with installation, Gatekeeper bypass, Accessibility-permission setup, and troubleshooting (DMG bundle failures, missing tray icon).
- **`docs/snippets-import.md`** — full reference: file format, field semantics, sample-file walkthrough, manual export recipe via `sqlite3` + `jq`, IPC surface, test matrix.
- **`CHANGELOG.md`** (this file).
- **6 new Rust unit tests** for the snippet import path (`cargo test --workspace`: 27 → 33).

### Fixed

- **CI was failing** with `ERR_PNPM_OUTDATED_LOCKFILE` because `macos/package.json` (added in 0.2.0) declared `@tauri-apps/cli` without a lockfile refresh. The lockfile is now in sync. — *#fix(ci)*
- **macOS build was broken** in 0.2.0:
  - `tauri.conf.json` declared `macOSPrivateApi: true` but the corresponding `tauri/macos-private-api` cargo feature was not enabled — `tauri-build` aborted. — *#fix(build)*
  - `app.set_activation_policy(...)` was wrapped in `if let Err(e) = …`, but the function returns `()`, not `Result`. The whole crate failed to typecheck on macOS. — *#fix(build)*
- **Multi-monitor popup placement** — the popup occasionally opened in the bottom-right of the active monitor and could even extend past the screen edge, most reliably reproducible on mixed-DPI setups (MacBook Retina + external display). The show/position pipeline was restructured: pick cursor monitor first, park the hidden window onto it, **then** `show()` + `set_focus()` (so `outer_size()` returns a real value), then re-resolve the monitor and finally call new helper `clamp_into_monitor()` which hard-clamps `x`/`y` to the monitor's bounds so the window can never overflow. — *#fix(hotkey)*

### Changed

- **`README.md`** — added a Multi-monitor placement subsection, surfaced the JSON-import feature, refreshed the repo layout to include `macos/` and the new docs, bumped test counts (24 frontend, 33 Rust).
- **`.gitignore`** — ignore `.claude/` (per-machine agent session state).

### Known issues

- The macOS DMG bundling step (`bundle_dmg.sh`) occasionally fails on busy disks (FileVault background indexing, Time Machine snapshot in progress). The `.app` itself is built first and is unaffected — see [`macos/README.md` § Troubleshooting](./macos/README.md#troubleshooting).
- macOS builds are **arm64 only** (Apple Silicon). Intel-Mac users need to build from source with `--target x86_64-apple-darwin`.
- Bundles are **not Apple-signed** — Gatekeeper will refuse to open on first launch. Workarounds documented in `macos/README.md`.

## [0.2.0] — 2026-04-24

### Added

- **macOS bundle shell** under [`macos/`](./macos) — DMG + `.app` targets, `entitlements.plist`, capabilities, thin `main.rs` reusing `clipsnap-core`.
- **Text expander** ("snippets") — abbreviations (e.g. `mfg`) with optional title and body. Matching snippets appear at the top of the History list when you type their abbreviation; Enter pastes the body. Dedicated **Snippets** tab for create/edit/delete, **Manage Snippets** entry in the tray menu.
- **GitHub Actions CI** — Rust + frontend tests on every push/PR ([`ci.yml`](./.github/workflows/ci.yml)).
- **GitHub Actions release** — builds Windows MSI/EXE and publishes a GitHub Release on `v*` tags ([`release.yml`](./.github/workflows/release.yml)).
- **Frontend unit tests** — vitest + happy-dom + @testing-library/react (`Footer`, `format` helpers — 24 tests).
- **Rust unit tests** — in-memory SQLite tests for `db` (insert/dedupe/list/touch/prune — 27 tests).
- README badges, icon header, polished layout.

### Known issues (resolved in 0.2.1)

- macOS build broken (`macos-private-api` cargo feature missing, `set_activation_policy` type mismatch). Fixed in 0.2.1.
- CI failing due to stale `pnpm-lock.yaml`. Fixed in 0.2.1.

## [0.1.0] — 2026-04-23

### Added

- Initial release. Windows-first clipboard history manager.
- Global hotkey `Ctrl+Shift+V` opens a frameless, always-on-top popup centered on the cursor's monitor.
- Captures **text**, **RTF**, **HTML**, **images** (≤ 5 MB, base64 PNG), and **file lists** via real OS clipboard change events (no polling).
- Fuzzy search (`fuse.js`, threshold 0.4) over preview text.
- Auto-paste with `enigo` (simulates `Ctrl+V` after the popup hides).
- SQLite history at `%APPDATA%\ClipSnap\history.db`, deduped on SHA-256, capped at 1 000 entries.
- System tray menu: Open · Pause Capture · Clear History · Start with Windows · Quit.
- pnpm + Cargo workspaces with shared [`core/`](./core) and [`win/`](./win) bundle shell.

[0.5.1]: https://github.com/pepperonas/clipsnap/releases/tag/v0.5.1
[0.5.0]: https://github.com/pepperonas/clipsnap/releases/tag/v0.5.0
[0.4.2]: https://github.com/pepperonas/clipsnap/releases/tag/v0.4.2
[0.4.1]: https://github.com/pepperonas/clipsnap/releases/tag/v0.4.1
[0.4.0]: https://github.com/pepperonas/clipsnap/releases/tag/v0.4.0
[0.3.1]: https://github.com/pepperonas/clipsnap/releases/tag/v0.3.1
[0.3.0]: https://github.com/pepperonas/clipsnap/releases/tag/v0.3.0
[0.2.12]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.12
[0.2.11]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.11
[0.2.10]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.10
[0.2.9]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.9
[0.2.8]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.8
[0.2.7]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.7
[0.2.6]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.6
[0.2.5]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.5
[0.2.4]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.4
[0.2.3]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.3
[0.2.2]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.2
[0.2.1]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.1
[0.2.0]: https://github.com/pepperonas/clipsnap/releases/tag/v0.2.0
[0.1.0]: https://github.com/pepperonas/clipsnap/commits/main
