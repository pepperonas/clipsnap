use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Monitor, PhysicalPosition, WebviewWindow};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::db::DbHandle;
use crate::expander;

/// Parse a `Modifier+...+Code` shortcut string. We deliberately avoid
/// `Shortcut::from_str` from the Tauri plugin — its lookup table is
/// incomplete (no `IntlBackslash`, `IntlBackquote`, `IntlRo`, `IntlYen`,
/// no media keys beyond a handful, …). `keyboard_types::Code` (which the
/// plugin re-exports as `Code`) implements `FromStr` for the full W3C
/// `KeyboardEvent.code` spec, so we route everything through it.
pub fn parse_shortcut(s: &str) -> Result<Shortcut> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("empty shortcut string"));
    }

    let tokens: Vec<&str> = trimmed.split('+').map(str::trim).collect();
    if tokens.iter().any(|t| t.is_empty()) {
        return Err(anyhow!("empty token in shortcut: {trimmed:?}"));
    }

    let (code_token, mod_tokens) = tokens
        .split_last()
        .ok_or_else(|| anyhow!("missing key code in shortcut: {trimmed:?}"))?;

    let mut mods = Modifiers::empty();
    for raw in mod_tokens {
        match raw.to_ascii_uppercase().as_str() {
            "CTRL" | "CONTROL" => mods |= Modifiers::CONTROL,
            "SHIFT" => mods |= Modifiers::SHIFT,
            "ALT" | "OPTION" | "OPT" => mods |= Modifiers::ALT,
            "META" | "CMD" | "COMMAND" | "SUPER" | "WIN" => mods |= Modifiers::SUPER,
            #[cfg(target_os = "macos")]
            "CMDORCTRL" | "CMDORCONTROL" | "COMMANDORCONTROL" | "COMMANDORCTRL" => {
                mods |= Modifiers::SUPER
            }
            #[cfg(not(target_os = "macos"))]
            "CMDORCTRL" | "CMDORCONTROL" | "COMMANDORCONTROL" | "COMMANDORCTRL" => {
                mods |= Modifiers::CONTROL
            }
            other => return Err(anyhow!("unknown modifier {other:?} in {trimmed:?}")),
        }
    }

    let code = Code::from_str(code_token)
        .map_err(|_| anyhow!("unknown key code {code_token:?} in {trimmed:?}"))?;

    Ok(Shortcut::new(Some(mods), code))
}

pub const POPUP_LABEL: &str = "popup";

/// Holds the currently-registered expander shortcut (if any), so we can
/// unregister it cleanly when the user changes it from the settings panel.
/// Tauri state.
#[derive(Default)]
pub struct ExpanderShortcutState {
    pub current: Arc<Mutex<Option<Shortcut>>>,
}

/// Ctrl+Shift+V global hotkey for the popup.
pub fn register(app: &AppHandle) -> Result<()> {
    let popup = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
    let app_for_popup = app.clone();
    app.global_shortcut()
        .on_shortcut(popup, move |_app, sc, event| {
            if event.state == ShortcutState::Pressed && *sc == popup {
                if let Err(e) = toggle_popup(&app_for_popup) {
                    tracing::warn!("toggle_popup failed: {e:#}");
                }
            }
        })
        .context("failed to register Ctrl+Shift+V")?;

    // OCR region — Cmd/Ctrl+Shift+O. Hard-coded for now (configurable
    // shortcut UI can come later); SUPER on macOS = Cmd, CONTROL
    // elsewhere via the same Shortcut struct on each OS at compile time.
    #[cfg(target_os = "macos")]
    let ocr_mods = Modifiers::SUPER | Modifiers::SHIFT;
    #[cfg(not(target_os = "macos"))]
    let ocr_mods = Modifiers::CONTROL | Modifiers::SHIFT;
    let ocr = Shortcut::new(Some(ocr_mods), Code::KeyO);
    let app_for_ocr = app.clone();
    app.global_shortcut()
        .on_shortcut(ocr, move |_app, sc, event| {
            if event.state == ShortcutState::Pressed && *sc == ocr {
                // Dispatch to a worker — the screencapture wait blocks
                // until the user finishes the marquee; doing it on the
                // global-shortcut callback thread would hang the
                // shortcut subsystem.
                let app = app_for_ocr.clone();
                std::thread::spawn(move || {
                    match crate::commands::run_ocr_pipeline(&app) {
                        Ok(r) if !r.cancelled && r.chars > 0 => {
                            tracing::info!("OCR captured {} chars", r.chars);
                        }
                        Ok(_) => tracing::debug!("OCR cancelled or empty"),
                        Err(e) => tracing::warn!("OCR pipeline: {e}"),
                    }
                });
            }
        })
        .context("failed to register OCR hotkey")?;

    Ok(())
}

/// Register the text-expander hotkey from a string like `"Alt+Backquote"`.
/// If a previous expander shortcut was registered, it is unregistered
/// first. A no-op if `enabled` is false (any prior registration is
/// removed).
pub fn register_expander(
    app: &AppHandle,
    state: &ExpanderShortcutState,
    hotkey: &str,
    enabled: bool,
) -> Result<()> {
    // Always unregister whatever was previously registered first.
    {
        let mut current = state.current.lock();
        if let Some(prev) = current.take() {
            let _ = app.global_shortcut().unregister(prev);
        }
    }

    if !enabled {
        tracing::info!("text expander disabled");
        return Ok(());
    }

    let shortcut = parse_shortcut(hotkey)
        .with_context(|| format!("could not parse expander hotkey {hotkey:?}"))?;

    let app_for_handler = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, sc, event| {
            if event.state == ShortcutState::Pressed && *sc == shortcut {
                // CRITICAL: enigo's `Key::Unicode(...)` mapping calls the
                // macOS TSM (Text Services Manager) APIs, which assert
                // they're called on the main thread. Calling them from a
                // worker thread — as we used to with `std::thread::spawn`
                // — fires `_dispatch_assert_queue_fail` and crashes the
                // process with EXC_BREAKPOINT/SIGTRAP. Dispatch the whole
                // cycle to the main thread instead. ~290 ms blocking is
                // acceptable here: the popup is hidden during the cycle,
                // so the freeze is invisible to the user.
                let app = app_for_handler.clone();
                let app_in_closure = app.clone();
                let _ = app.run_on_main_thread(move || {
                    if let Some(db) = app_in_closure.try_state::<DbHandle>() {
                        if let Err(e) = expander::expand_at_cursor(&db) {
                            tracing::warn!("expand_at_cursor failed: {e:#}");
                        }
                    }
                });
            }
        })
        .with_context(|| format!("failed to register expander hotkey {hotkey:?}"))?;

    *state.current.lock() = Some(shortcut);
    tracing::info!("text expander armed: {hotkey}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shortcut_accepts_alt_backquote() {
        let s = parse_shortcut("Alt+Backquote").expect("should parse");
        // Modifiers tolerate any case — verify via a different mod casing.
        assert_eq!(s, parse_shortcut("ALT+Backquote").unwrap());
        assert_eq!(s, parse_shortcut("alt+Backquote").unwrap());
        // Codes must match the W3C `KeyboardEvent.code` PascalCase exactly,
        // since that's what the frontend always sends.
        assert!(parse_shortcut("Alt+backquote").is_err());
    }

    #[test]
    fn parse_shortcut_accepts_intl_backslash() {
        // The whole reason we don't use the Tauri plugin's parser — its
        // table doesn't include IntlBackslash, but `keyboard_types` does.
        // German ISO macOS keyboards report the top-left `^/°` key as
        // IntlBackslash via WebKit, so this is a real path users hit.
        parse_shortcut("Alt+IntlBackslash").expect("IntlBackslash must parse");
        parse_shortcut("Ctrl+Shift+IntlBackslash").expect("with multiple mods");
    }

    #[test]
    fn parse_shortcut_supports_every_letter_and_function_key() {
        for code in ["KeyA", "KeyZ", "Digit0", "Digit9", "F1", "F12", "F19"] {
            parse_shortcut(&format!("Alt+{code}"))
                .unwrap_or_else(|e| panic!("{code} did not parse: {e:#}"));
        }
    }

    #[test]
    fn parse_shortcut_supports_modifier_aliases() {
        // CmdOrCtrl is the cross-platform alias the Tauri plugin advertises.
        parse_shortcut("CmdOrCtrl+KeyV").expect("alias must parse");
        parse_shortcut("Cmd+KeyV").expect("Cmd alias must parse");
        parse_shortcut("Option+KeyV").expect("Option alias must parse");
    }

    #[test]
    fn parse_shortcut_rejects_unknown_modifier() {
        let err = parse_shortcut("Hyper+KeyA")
            .expect_err("Hyper is not a Tauri modifier")
            .to_string();
        assert!(err.contains("unknown modifier"), "got: {err}");
    }

    #[test]
    fn parse_shortcut_rejects_unknown_code() {
        let err = parse_shortcut("Alt+NotAKey")
            .expect_err("garbage code must error")
            .to_string();
        assert!(err.contains("unknown key code"), "got: {err}");
    }

    #[test]
    fn parse_shortcut_rejects_empty_input() {
        assert!(parse_shortcut("").is_err());
        assert!(parse_shortcut("   ").is_err());
    }

    #[test]
    fn parse_shortcut_rejects_dangling_plus() {
        // "Alt+" or "+KeyA" produce empty tokens — should fail loudly.
        assert!(parse_shortcut("Alt+").is_err());
        assert!(parse_shortcut("+KeyA").is_err());
    }

    #[test]
    fn parse_shortcut_accepts_single_key_no_modifier() {
        // Single-key shortcuts (e.g. F19) are valid. The plugin happily
        // registers them — we should accept them too.
        parse_shortcut("F19").expect("bare F19 must parse");
    }
}

pub fn toggle_popup(app: &AppHandle) -> Result<()> {
    let window = app
        .get_webview_window(POPUP_LABEL)
        .context("popup window not found")?;

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return Ok(());
    }

    show_and_position(&window)?;
    let _ = app.emit("window-shown", ());
    Ok(())
}

/// Show the popup unconditionally and center it on the cursor monitor.
pub fn show_popup(app: &AppHandle) -> Result<()> {
    let window = app
        .get_webview_window(POPUP_LABEL)
        .context("popup window not found")?;
    show_and_position(&window)
}

pub fn hide_popup(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(POPUP_LABEL) {
        let _ = w.hide();
    }
    // Tell the frontend the popup is gone so it can drop transient state
    // (e.g., close any open modal so the next "open popup" shows the
    // default History view, not the modal that was up at hide-time).
    let _ = app.emit("popup-hidden", ());
    // On macOS, hiding the window alone does not reliably return key focus
    // to the previously active app — especially with `ActivationPolicy::
    // Accessory`. Hiding the whole app (NSApp.hide(nil)) makes the OS
    // restore the prior frontmost app, which is what `enigo`'s synthesized
    // Cmd+V needs in order to land in the right window.
    #[cfg(target_os = "macos")]
    let _ = app.hide();
}

/// Move the (currently hidden) window onto the cursor's monitor, **then**
/// show, **then** center. Reading `outer_size()` before `show()` returns
/// stale or zero values on macOS for hidden windows; doing it in this order
/// guarantees the centering math has the real window size.
fn show_and_position(window: &WebviewWindow) -> Result<()> {
    let target = pick_cursor_monitor(window);

    // 1) Park the hidden window somewhere on the target monitor so that
    //    `current_monitor()` reports the right one after `show()`.
    if let Some(m) = &target {
        let mpos = m.position();
        let msize = m.size();
        let parked_x = mpos.x + (msize.width as i32 / 4);
        let parked_y = mpos.y + (msize.height as i32 / 4);
        let _ = window.set_position(PhysicalPosition::new(parked_x, parked_y));
    }

    // 2) Show + focus. After this, `outer_size()` reflects the real size.
    window.show()?;
    window.set_focus()?;

    // 3) Re-resolve the monitor (in case the user moved the cursor between
    //    the parking and the show), then center horizontally + ~⅓ down,
    //    clamped to the monitor's visible area.
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or(target)
        .or_else(|| window.primary_monitor().ok().flatten());
    if let Some(m) = monitor {
        if let Err(e) = clamp_into_monitor(window, &m) {
            tracing::debug!("clamp_into_monitor: {e:#}");
        }
    }
    Ok(())
}

/// Find the monitor that contains the OS cursor; fall back to primary.
fn pick_cursor_monitor(window: &WebviewWindow) -> Option<Monitor> {
    let pos = window.cursor_position().ok()?;
    let monitors = window.available_monitors().ok()?;
    monitors
        .into_iter()
        .find(|m| {
            let p = m.position();
            let s = m.size();
            let x = pos.x as i32;
            let y = pos.y as i32;
            x >= p.x
                && x < p.x + s.width as i32
                && y >= p.y
                && y < p.y + s.height as i32
        })
        .or_else(|| window.primary_monitor().ok().flatten())
}

/// Center horizontally, place ~⅓ down vertically, then clamp so the window
/// can never extend past any edge of the monitor.
fn clamp_into_monitor(window: &WebviewWindow, monitor: &Monitor) -> Result<()> {
    let mpos = monitor.position();
    let msize = monitor.size();
    let wsize = window.outer_size().unwrap_or_default();

    // If outer_size is still bogus (zero), bail rather than placing wrongly.
    if wsize.width == 0 || wsize.height == 0 {
        return Ok(());
    }

    let mw = msize.width as i32;
    let mh = msize.height as i32;
    let ww = wsize.width as i32;
    let wh = wsize.height as i32;

    // Desired position: horizontally centered, ~⅓ down.
    let mut x = mpos.x + (mw - ww) / 2;
    let mut y = mpos.y + (mh - wh) / 3;

    // Clamp to monitor bounds. If the window is somehow larger than the
    // monitor (extreme zoom, etc.), `max_x < min_x` — pin to top-left.
    let min_x = mpos.x;
    let max_x = mpos.x + (mw - ww).max(0);
    let min_y = mpos.y;
    let max_y = mpos.y + (mh - wh).max(0);
    x = x.clamp(min_x, max_x);
    y = y.clamp(min_y, max_y);

    window.set_position(PhysicalPosition::new(x, y))?;
    Ok(())
}
