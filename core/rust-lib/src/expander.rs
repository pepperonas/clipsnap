//! Trigger-based text expander.
//!
//! Workflow when the user presses the configured hotkey from any focused
//! text field:
//!
//! 1. Save the current clipboard text (best-effort — image / file
//!    clipboards are *not* preserved across the expansion in this version).
//! 2. Synthesize the platform's "select previous word" shortcut
//!    (`Option+Shift+Left` on macOS; `Ctrl+Shift+Left` elsewhere) followed
//!    by the platform copy shortcut.
//! 3. Read the now-selected word back out of the clipboard.
//! 4. Look it up in the snippets table via
//!    [`snippets::find_by_exact_abbreviation`].
//! 5. **Hit:** write the snippet body to the clipboard, synthesize paste —
//!    overwrites the still-active selection in the source app.
//! 6. **Miss:** do nothing (the selection stays visible so the user
//!    notices the failed match).
//! 7. Restore the original clipboard text after a small delay.
//!
//! All of step 2-5 happens while the popup is hidden — the source app
//! retains key focus the whole time.

use anyhow::{anyhow, Result};
use clipboard_rs::{Clipboard, ClipboardContext};
use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use serde::Serialize;
use std::thread;
use std::time::Duration;

use crate::db::DbHandle;
use crate::snippets;
use crate::text_field::{default_field_access, native_path, CapturePath, ReplaceOutcome};

/// enigo `Settings` with `open_prompt_to_get_permissions = false` —
/// see paste.rs for the full rationale. Every `Enigo::new()` here uses
/// this so untrusted-process calls fail silently rather than firing
/// the macOS dialog as a side effect.
fn enigo_settings() -> Settings {
    Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    }
}

// Whether the OS has granted ClipSnap permission to synthesize keyboard
// events (macOS Accessibility / "Privacy & Security" → Accessibility).
// `enigo` silently no-ops without it on macOS — the hotkey fires, the
// `expand_at_cursor` cycle runs, but `Cmd+Shift+←` / `Cmd+C` / `Cmd+V`
// never reach the source app, so the abbreviation never gets selected
// or replaced. Knowing this state up-front lets the UI surface it
// instead of leaving the user puzzled.
#[cfg(target_os = "macos")]
mod macos_ax {
    use std::ffi::c_void;

    type CFTypeRef = *const c_void;
    type CFAllocatorRef = *const c_void;
    type CFDictionaryRef = *const c_void;
    type CFIndex = isize;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub fn AXIsProcessTrusted() -> bool;
        pub fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
        pub static kAXTrustedCheckOptionPrompt: CFTypeRef;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub static kCFBooleanTrue: CFTypeRef;
        pub static kCFAllocatorDefault: CFAllocatorRef;
        pub static kCFTypeDictionaryKeyCallBacks: c_void;
        pub static kCFTypeDictionaryValueCallBacks: c_void;

        pub fn CFDictionaryCreate(
            allocator: CFAllocatorRef,
            keys: *const CFTypeRef,
            values: *const CFTypeRef,
            num_values: CFIndex,
            key_call_backs: *const c_void,
            value_call_backs: *const c_void,
        ) -> CFDictionaryRef;

        pub fn CFRelease(cf: CFTypeRef);
    }
}

/// Whether the OS-level synthetic-input permission is active.
#[cfg(target_os = "macos")]
pub fn accessibility_granted() -> bool {
    unsafe { macos_ax::AXIsProcessTrusted() }
}

/// Whether the OS-level synthetic-input permission is active.
#[cfg(not(target_os = "macos"))]
pub fn accessibility_granted() -> bool {
    // Other platforms either don't gate synthetic input behind a TCC-style
    // permission (Windows, X11) or do so through an entirely different
    // mechanism (Wayland portals). Optimistic default.
    true
}

/// Trigger the macOS "would like to control this computer" dialog and
/// add ClipSnap to **System Settings → Privacy & Security → Accessibility**
/// so the user can flip the toggle there. Returns the *current* trusted
/// status (which is almost always `false` immediately after the prompt
/// appears — the user still has to grant it).
///
/// On non-macOS this is a no-op that returns the same as
/// [`accessibility_granted`].
#[cfg(target_os = "macos")]
pub fn request_accessibility_grant() -> bool {
    use macos_ax::*;
    use std::ffi::c_void;

    unsafe {
        let key = kAXTrustedCheckOptionPrompt;
        let value = kCFBooleanTrue;
        let dict = CFDictionaryCreate(
            kCFAllocatorDefault,
            &key as *const _,
            &value as *const _,
            1,
            &kCFTypeDictionaryKeyCallBacks as *const _ as *const c_void,
            &kCFTypeDictionaryValueCallBacks as *const _ as *const c_void,
        );
        let trusted = AXIsProcessTrustedWithOptions(dict);
        CFRelease(dict);
        trusted
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_accessibility_grant() -> bool {
    accessibility_granted()
}

/// Open **System Settings → Privacy & Security → Accessibility** at the
/// right pane via the macOS preference URL scheme. No-op on other OSes.
#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> anyhow::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("open System Settings: {e}"))
}

#[cfg(not(target_os = "macos"))]
pub fn open_accessibility_settings() -> anyhow::Result<()> {
    Ok(())
}

/// Wipe stale TCC grants for ClipSnap (Accessibility + PostEvent), then
/// fire the standard "would like to control" prompt with the *current*
/// cdhash. Solves the common "toggle says on but ClipSnap still sees
/// untrusted" state that occurs after every real source-change rebuild
/// — the previous toggle was for an older cdhash, the new binary needs
/// a fresh grant. Runs `tccutil reset` for our own bundle id, which
/// doesn't require sudo.
#[cfg(target_os = "macos")]
pub fn force_reset_and_request_grant() -> anyhow::Result<bool> {
    // 1) Wipe whatever stale entries exist. tccutil exits 0 even if
    //    there's nothing to reset, so we don't need to check.
    let _ = std::process::Command::new("tccutil")
        .args(["reset", "Accessibility", "io.celox.clipsnap"])
        .status();
    let _ = std::process::Command::new("tccutil")
        .args(["reset", "PostEvent", "io.celox.clipsnap"])
        .status();

    // 2) Fire the prompt. This re-adds ClipSnap to System Settings →
    //    Accessibility with the current cdhash, ready to be toggled on.
    Ok(request_accessibility_grant())
}

#[cfg(not(target_os = "macos"))]
pub fn force_reset_and_request_grant() -> anyhow::Result<bool> {
    Ok(accessibility_granted())
}

/// Settings keys.
pub const KEY_HOTKEY: &str = "expander.hotkey";
pub const KEY_ENABLED: &str = "expander.enabled";
/// One-shot flag: set once the legacy `Alt+Backquote` default has been
/// migrated to [`DEFAULT_HOTKEY`]. Prevents re-migrating a value the user
/// deliberately set back to `Alt+Backquote` afterwards.
pub const KEY_HOTKEY_MIGRATED: &str = "expander.hotkey_migrated_v0_12";

/// Default hotkey when no setting has ever been written. `Alt + Digit1`
/// (the `1` row key, **not** the numpad) is layout-stable on every
/// keyboard: it has a fixed `KeyboardEvent.code` everywhere, isn't a
/// dead key on any layout, and isn't reserved by macOS or Windows. The
/// previous default `Alt+Backquote` was unreachable on German ISO Macs
/// (the physical `^` key reports as `IntlBackslash`, not `Backquote`).
pub const DEFAULT_HOTKEY: &str = "Alt+Digit1";

/// The pre-0.12 default, kept only so [`migrate_legacy_default`] can
/// recognise an un-customised old install and bump it.
pub const LEGACY_DEFAULT_HOTKEY: &str = "Alt+Backquote";

/// Error string returned by [`expand_at_cursor`] / [`diagnose_at_cursor`]
/// when synthetic input is needed but the OS hasn't granted it (macOS
/// Accessibility). The hotkey handler turns this sentinel into a popup +
/// `expander-permission-needed` event so the user gets an actionable
/// banner instead of a silent no-op.
pub const ERR_NO_ACCESSIBILITY: &str = "ax.permission_denied";

/// One-time settings migration: if the stored hotkey is still the pre-0.12
/// default `Alt+Backquote` (i.e. the user never changed it), rewrite it to
/// the new layout-stable [`DEFAULT_HOTKEY`]. A migration flag makes this
/// idempotent and prevents clobbering a value the user re-picks later.
/// Returns the hotkey string that should be used from here on.
pub fn migrate_legacy_default(db: &DbHandle) -> String {
    use crate::settings;

    let stored = settings::get_or(db, KEY_HOTKEY, DEFAULT_HOTKEY)
        .unwrap_or_else(|_| DEFAULT_HOTKEY.to_string());

    if settings::get_bool(db, KEY_HOTKEY_MIGRATED, false).unwrap_or(false) {
        return stored;
    }

    let upgraded = if stored == LEGACY_DEFAULT_HOTKEY {
        let _ = settings::set(db, KEY_HOTKEY, DEFAULT_HOTKEY);
        tracing::info!("migrated expander hotkey {LEGACY_DEFAULT_HOTKEY} → {DEFAULT_HOTKEY}");
        DEFAULT_HOTKEY.to_string()
    } else {
        stored
    };
    let _ = settings::set(db, KEY_HOTKEY_MIGRATED, "true");
    upgraded
}

/// Diagnostic outcome of an expand-cycle attempt: what got captured,
/// whether it matched a snippet, and a preview of what would be pasted.
/// Used by the Settings panel's "Test now" button so the user can see
/// the *exact* reason an expansion fails (no abbreviation matched, the
/// captured text was empty, …) instead of a silent no-op.
#[derive(Debug, Serialize)]
pub struct DiagnoseResult {
    /// The whitespace-trimmed text captured before the cursor. Empty
    /// string if nothing was selectable (no text before the cursor).
    pub captured: String,
    /// Set when `find_by_exact_abbreviation` returned a row.
    pub matched_abbreviation: Option<String>,
    /// First ~80 characters of the matched snippet body — gives the user
    /// confidence the right snippet would be pasted.
    pub paste_preview: Option<String>,
    /// Which capture mechanism actually succeeded:
    /// - `ax` — macOS Accessibility API (`AXUIElement`).
    /// - `uia` — Windows UI Automation (`IUIAutomation`).
    /// - `clipboard` — fell back to `Cmd/Ctrl+Shift+←` + `Cmd/Ctrl+C`.
    pub path: CapturePath,
}

/// Run the capture half of expansion (read the word before the cursor,
/// look it up) **without** pasting. The caller is responsible for hiding
/// the popup *before* this runs so the AX/UIA call targets the source
/// app, not ClipSnap itself.
///
/// Capture-path policy:
/// 1. **Try AX (macOS) / UIA (Windows) first.** No keystroke synthesis,
///    no clipboard touch — the cleanest path. Works in any app that
///    exposes its focused field through accessibility.
/// 2. **Fall back to the clipboard roundtrip** (`Cmd/Ctrl+Shift+←` +
///    `Cmd/Ctrl+C`) only when AX/UIA returns `None`. The user's
///    clipboard is saved and restored around the operation.
pub fn diagnose_at_cursor(db: &DbHandle) -> Result<DiagnoseResult> {
    // 1) AX/UIA first — clean read, no side effects.
    // BUT: on macOS, calling AX functions on an *untrusted* process
    // triggers the system "would like to control" prompt as a side
    // effect — even when we just want to silently fall back to the
    // clipboard path. So short-circuit when we know the process isn't
    // trusted yet.
    let access = default_field_access();
    let access_ok = accessibility_granted();

    // On macOS, without the Accessibility grant we can neither read the
    // focused field (AX) nor synthesize the clipboard-roundtrip keystrokes
    // (enigo no-ops) — the diagnosis would just report an empty capture
    // and leave the user guessing. Surface the real reason instead.
    #[cfg(target_os = "macos")]
    if !access_ok {
        return Err(anyhow!(
            "Accessibility permission isn't granted — ClipSnap can't read the \
             focused field or synthesize keystrokes without it. Grant it in the \
             section above, then relaunch ClipSnap."
        ));
    }

    let (captured, path) = match if access_ok {
        access.read_word_before_cursor()
    } else {
        Ok(None)
    } {
        Ok(Some(word)) => (word, native_path()),
        Ok(None) | Err(_) => {
            // 2) Fall back to the clipboard roundtrip.
            let saved = read_clipboard_text();
            select_previous_word()?;
            thread::sleep(Duration::from_millis(30));
            send_copy()?;
            thread::sleep(Duration::from_millis(80));
            let captured_raw = read_clipboard_text().unwrap_or_default();
            restore_clipboard(saved.as_deref());
            (
                trim_abbreviation(&captured_raw).to_string(),
                CapturePath::Clipboard,
            )
        }
    };

    let mut result = DiagnoseResult {
        captured: captured.clone(),
        matched_abbreviation: None,
        paste_preview: None,
        path,
    };

    if !captured.is_empty() {
        if let Some(snippet) = snippets::find_by_exact_abbreviation(db, &captured)? {
            // First 80 chars of the body, single-line preview.
            let preview: String = snippet
                .body
                .replace('\n', " ")
                .chars()
                .take(80)
                .collect();
            result.matched_abbreviation = Some(snippet.abbreviation);
            result.paste_preview = Some(preview);
        }
    }

    Ok(result)
}

/// Run a full expand-at-cursor cycle. Errors are returned for logging but
/// the orchestration layer (the hotkey handler) treats them as recoverable
/// — the next press starts a fresh attempt.
///
/// Tries AX (macOS) / UIA (Windows) **first** — the clean path with no
/// clipboard touch and no flickering selection. Falls back to the
/// keystroke + clipboard roundtrip only when the focused element doesn't
/// expose accessibility info.
pub fn expand_at_cursor(db: &DbHandle) -> Result<()> {
    // The hotkey itself is `Alt+<key>` — when this runs (queued onto the
    // main thread, a few ms after key-down) the user may still be
    // physically holding that `Alt`. Give it a beat to come up before we
    // synthesize our own modifier chords; otherwise enigo's press/release
    // can race the user's still-down key and produce a stuck-modifier
    // state in the source app. Invisible: the popup is hidden the whole
    // time anyway.
    thread::sleep(Duration::from_millis(40));

    // ── Path 1: native accessibility (AX / UIA) ────────────────────────────
    // Skip AX entirely when the process isn't trusted — calling AX from
    // an untrusted process fires the macOS permission prompt as a side
    // effect, which is exactly the noise we want to avoid for users in
    // the post-rebuild stale-cdhash state. Fall straight through to the
    // clipboard path; the SettingsPanel banner / Force re-grant button
    // is the right place to surface the underlying permission issue.
    let access = default_field_access();
    if accessibility_granted() {
        if let Ok(Some(word)) = access.read_word_before_cursor() {
            if let Some(snippet) = snippets::find_by_exact_abbreviation(db, &word)? {
                // Try the in-place replace via the same accessibility layer.
                match access.try_replace_word_before_cursor(&snippet.body) {
                    Ok(ReplaceOutcome::Replaced) => return Ok(()),
                    Ok(ReplaceOutcome::SelectionActive) => {
                        // The AX layer *selected* the abbreviation but the
                        // in-place text set was a no-op — the typical
                        // Electron / Chromium / Mac-Catalyst case (WhatsApp,
                        // Slack, Discord, VS Code, …): those expose `AXValue`
                        // read-only and return success for the set anyway.
                        // The abbreviation is highlighted right now, so just
                        // paste the body over it — do NOT re-select.
                        tracing::debug!(
                            "AX selected the abbreviation but in-place replace \
                             was a no-op; pasting body over the live selection"
                        );
                        return paste_over_selection(&snippet.body);
                    }
                    Ok(ReplaceOutcome::Unsupported) => {
                        tracing::debug!(
                            "focused element exposes no settable text attrs; \
                             falling back to keystroke-select + paste"
                        );
                        // Reuse the abbreviation we already captured.
                        return expand_via_clipboard(db, Some(&word), Some(&snippet.body));
                    }
                    Err(e) => {
                        tracing::warn!("AX/UIA replace errored: {e:#}; falling back");
                        return expand_via_clipboard(db, Some(&word), Some(&snippet.body));
                    }
                }
            }
            // No snippet matched. We still want the user to *see* the
            // failure — leaving the field untouched is the right move;
            // no fallback needed for the no-match case.
            return Ok(());
        }
    }

    // ── Path 2: clipboard roundtrip (legacy) ───────────────────────────────
    // This path synthesizes `Cmd/Ctrl+Shift+←` + `Cmd/Ctrl+C` + `Cmd/Ctrl+V`
    // via enigo. On macOS that needs the Accessibility (AXIsProcessTrusted)
    // grant — and if we got here on macOS, the `if accessibility_granted()`
    // above was false, so this would silently no-op. Bail with the
    // sentinel instead; the caller turns it into a "grant Accessibility"
    // banner. On Windows/Linux `accessibility_granted()` is always true,
    // so this never fires and the keystroke path runs normally.
    #[cfg(target_os = "macos")]
    if !accessibility_granted() {
        return Err(anyhow!(ERR_NO_ACCESSIBILITY));
    }

    expand_via_clipboard(db, None, None)
}

/// Pre-AX/UIA expand path. Used as a fallback when the focused element
/// doesn't expose accessibility, and (with `prefetched_*` set) when
/// AX/UIA gave us the abbreviation + body but couldn't perform the
/// replace itself — we still need keystroke synthesis to actually
/// replace the word.
fn expand_via_clipboard(
    db: &DbHandle,
    prefetched_word: Option<&str>,
    prefetched_body: Option<&str>,
) -> Result<()> {
    let saved = read_clipboard_text();

    // Select-prev-word + copy is only needed when we don't already know
    // the abbreviation. With prefetched data the cursor is still right
    // after the abbreviation but no selection exists yet — re-select.
    select_previous_word()?;
    thread::sleep(Duration::from_millis(30));
    send_copy()?;
    thread::sleep(Duration::from_millis(80));

    let abbr_raw = read_clipboard_text().unwrap_or_default();
    let abbr = if let Some(w) = prefetched_word {
        // Prefer the AX-captured word — guards against the clipboard
        // capturing the wrong region in apps that mistreat Shift+Left.
        w
    } else {
        trim_abbreviation(&abbr_raw)
    };
    if abbr.is_empty() {
        restore_clipboard(saved.as_deref());
        return Ok(());
    }

    // 4) Look it up — unless the AX/UIA path already gave us the body.
    let body = if let Some(b) = prefetched_body {
        b.to_string()
    } else {
        let hit = snippets::find_by_exact_abbreviation(db, abbr)?;
        let Some(snippet) = hit else {
            // Selection stays highlighted in the source app — visual cue
            // that nothing matched. Restore clipboard before bailing.
            restore_clipboard(saved.as_deref());
            return Ok(());
        };
        snippet.body
    };

    // 5) Replace selection: write the body, paste over the highlight.
    write_clipboard_text(&body)?;
    thread::sleep(Duration::from_millis(50));
    send_paste()?;

    // 6) Restore the user's original clipboard after the paste has
    //    consumed the snippet body. The delay is generous — too short and
    //    the source app may end up pasting the *restored* clipboard.
    thread::sleep(Duration::from_millis(180));
    restore_clipboard(saved.as_deref());

    Ok(())
}

/// Paste `body` over whatever is currently selected in the focused app,
/// then restore the user's clipboard text. Used when the accessibility
/// layer managed to *select* the abbreviation but couldn't replace the
/// text in place — the typical Electron / Mac-Catalyst case. No
/// re-selection here: the selection is already on the abbreviation, so a
/// `Cmd/Ctrl+Shift+←` would only extend it onto the previous word.
fn paste_over_selection(body: &str) -> Result<()> {
    let saved = read_clipboard_text();
    write_clipboard_text(body)?;
    // Give the pasteboard write time to propagate before we paste —
    // Catalyst / Electron apps can be sluggish about observing it.
    thread::sleep(Duration::from_millis(50));
    send_paste()?;
    // Generous restore delay — too short and the source app ends up
    // pasting the *restored* clipboard instead of the body.
    thread::sleep(Duration::from_millis(180));
    restore_clipboard(saved.as_deref());
    Ok(())
}

/// Trim common boundary characters that the platform may include in the
/// "previous word" selection (trailing space the user typed after the
/// abbreviation, NBSP, newlines, …).
fn trim_abbreviation(raw: &str) -> &str {
    raw.trim_matches(|c: char| c.is_whitespace() || c == '\u{00A0}')
}

fn read_clipboard_text() -> Option<String> {
    let ctx = ClipboardContext::new().ok()?;
    ctx.get_text().ok()
}

fn write_clipboard_text(text: &str) -> Result<()> {
    let ctx = ClipboardContext::new()
        .map_err(|e| anyhow!("clipboard ctx init failed: {e:?}"))?;
    ctx.set_text(text.to_string())
        .map_err(|e| anyhow!("set_text failed: {e:?}"))?;
    Ok(())
}

fn restore_clipboard(saved: Option<&str>) {
    if let Some(text) = saved {
        let _ = write_clipboard_text(text);
    }
}

#[cfg(target_os = "macos")]
fn word_modifier() -> Key {
    // On macOS Option == Alt, both physically and in enigo's mapping.
    Key::Alt
}
#[cfg(not(target_os = "macos"))]
fn word_modifier() -> Key {
    Key::Control
}

#[cfg(target_os = "macos")]
fn cmd_modifier() -> Key {
    Key::Meta
}
#[cfg(not(target_os = "macos"))]
fn cmd_modifier() -> Key {
    Key::Control
}

fn select_previous_word() -> Result<()> {
    let mut e = Enigo::new(&enigo_settings())
        .map_err(|err| anyhow!("enigo init failed: {err:?}"))?;
    let modifier = word_modifier();
    e.key(modifier, Press)
        .map_err(|err| anyhow!("modifier press: {err:?}"))?;
    e.key(Key::Shift, Press)
        .map_err(|err| anyhow!("shift press: {err:?}"))?;
    e.key(Key::LeftArrow, Press)
        .map_err(|err| anyhow!("left press: {err:?}"))?;
    e.key(Key::LeftArrow, Release)
        .map_err(|err| anyhow!("left release: {err:?}"))?;
    e.key(Key::Shift, Release)
        .map_err(|err| anyhow!("shift release: {err:?}"))?;
    e.key(modifier, Release)
        .map_err(|err| anyhow!("modifier release: {err:?}"))?;
    Ok(())
}

fn send_copy() -> Result<()> {
    send_modified_letter('c')
}

fn send_paste() -> Result<()> {
    send_modified_letter('v')
}

fn send_modified_letter(letter: char) -> Result<()> {
    let mut e = Enigo::new(&enigo_settings())
        .map_err(|err| anyhow!("enigo init failed: {err:?}"))?;
    let m = cmd_modifier();
    e.key(m, Press)
        .map_err(|err| anyhow!("modifier press: {err:?}"))?;
    e.key(Key::Unicode(letter), Press)
        .map_err(|err| anyhow!("letter press: {err:?}"))?;
    e.key(Key::Unicode(letter), Release)
        .map_err(|err| anyhow!("letter release: {err:?}"))?;
    e.key(m, Release)
        .map_err(|err| anyhow!("modifier release: {err:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings;

    #[test]
    fn trim_abbreviation_strips_surrounding_whitespace() {
        assert_eq!(trim_abbreviation("  mfg  "), "mfg");
        assert_eq!(trim_abbreviation("mfg\n"), "mfg");
        assert_eq!(trim_abbreviation("\u{00A0}mfg"), "mfg");
        assert_eq!(trim_abbreviation("mfg"), "mfg");
        assert_eq!(trim_abbreviation(""), "");
        assert_eq!(trim_abbreviation("   "), "");
    }

    #[test]
    fn settings_constants_match_documented_keys() {
        // Sanity check — these strings are referenced from the frontend
        // settings UI, so they're effectively part of our public API.
        assert_eq!(KEY_HOTKEY, "expander.hotkey");
        assert_eq!(KEY_ENABLED, "expander.enabled");
        assert_eq!(DEFAULT_HOTKEY, "Alt+Digit1");
        assert_eq!(LEGACY_DEFAULT_HOTKEY, "Alt+Backquote");
        // The new default must be a string the shortcut parser accepts.
        crate::hotkey::parse_shortcut(DEFAULT_HOTKEY).expect("default hotkey must parse");
    }

    #[test]
    fn migrate_legacy_default_upgrades_untouched_install() {
        use crate::settings;
        use parking_lot::Mutex;
        use rusqlite::Connection;
        use std::sync::Arc;

        let db = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        settings::init_table(&db).unwrap();

        // Untouched old install: hotkey row holds the legacy default.
        settings::set(&db, KEY_HOTKEY, LEGACY_DEFAULT_HOTKEY).unwrap();
        assert_eq!(migrate_legacy_default(&db), DEFAULT_HOTKEY);
        assert_eq!(
            settings::get(&db, KEY_HOTKEY).unwrap().as_deref(),
            Some(DEFAULT_HOTKEY)
        );
        // Idempotent — and won't clobber a value re-picked afterwards.
        settings::set(&db, KEY_HOTKEY, LEGACY_DEFAULT_HOTKEY).unwrap();
        assert_eq!(migrate_legacy_default(&db), LEGACY_DEFAULT_HOTKEY);
    }

    #[test]
    fn migrate_legacy_default_leaves_custom_hotkey_alone() {
        use crate::settings;
        use parking_lot::Mutex;
        use rusqlite::Connection;
        use std::sync::Arc;

        let db = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        settings::init_table(&db).unwrap();
        settings::set(&db, KEY_HOTKEY, "Ctrl+Shift+E").unwrap();
        assert_eq!(migrate_legacy_default(&db), "Ctrl+Shift+E");
        assert_eq!(
            settings::get(&db, KEY_HOTKEY).unwrap().as_deref(),
            Some("Ctrl+Shift+E")
        );
    }

    #[test]
    fn error_sentinel_is_stable() {
        // The hotkey handler matches on this exact string.
        assert_eq!(ERR_NO_ACCESSIBILITY, "ax.permission_denied");
    }

    #[test]
    fn settings_module_round_trip_for_expander_keys() {
        // Belt-and-braces: ensure the keys we export are usable with the
        // settings store.
        use parking_lot::Mutex;
        use rusqlite::Connection;
        use std::sync::Arc;

        let conn = Connection::open_in_memory().unwrap();
        let db = Arc::new(Mutex::new(conn));
        settings::init_table(&db).unwrap();

        assert_eq!(
            settings::get_or(&db, KEY_HOTKEY, DEFAULT_HOTKEY).unwrap(),
            DEFAULT_HOTKEY
        );
        assert!(settings::get_bool(&db, KEY_ENABLED, false).unwrap() == false);

        settings::set(&db, KEY_HOTKEY, "Ctrl+Shift+E").unwrap();
        settings::set(&db, KEY_ENABLED, "true").unwrap();
        assert_eq!(
            settings::get_or(&db, KEY_HOTKEY, DEFAULT_HOTKEY).unwrap(),
            "Ctrl+Shift+E"
        );
        assert!(settings::get_bool(&db, KEY_ENABLED, false).unwrap());
    }
}
