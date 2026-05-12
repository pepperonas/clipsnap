//! Windows UI Automation (UIA) implementation of `FieldAccess`.
//!
//! Reads the focused element's text and selection via the COM API
//! `IUIAutomation` (UIAutomationCore.dll). For the *replace* half of the
//! trait we deliberately go back to keystroke synthesis (Backspace×N +
//! type-body via `enigo`), because:
//!
//! - `IUIAutomationTextEditPattern2::Replace` is only implemented by a
//!   small set of controls (modern UIAuto-aware Edit / RichEdit). Most
//!   real-world apps don't expose it.
//! - `IUIAutomationValuePattern::SetValue` replaces the *entire* control
//!   value, not just the word range.
//! - Keystroke synthesis on Windows uses `SendInput` (via `enigo`),
//!   which has none of the macOS TSM main-thread restrictions and works
//!   reliably across every app that accepts keyboard input.
//!
//! So we use UIA for the *read* (the unreliable part on macOS without
//! AX) and fall back to `enigo` for the deterministic *write*.
//!
//! No additional permission required: UIA is part of the standard
//! Windows accessibility infrastructure and any user-process can
//! consume it.

use anyhow::{anyhow, Context, Result};
use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use windows::core::Interface;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationTextPattern, TextUnit_Word, UIA_TextPatternId,
};

use super::{trim_word, FieldAccess, ReplaceOutcome};

pub struct UiaFieldAccess;

impl UiaFieldAccess {
    /// One-shot UIA query: returns the word immediately before the cursor,
    /// or `None` when the focused element doesn't expose a TextPattern at
    /// all. Each call sets up its own COM apartment because there's no
    /// guarantee the IPC thread that calls us has one already.
    fn read_word(&self) -> Result<Option<String>> {
        unsafe {
            // CoInitializeEx is reference-counted; calling it multiple
            // times is fine, and we don't bother to CoUninitialize on
            // success — Tauri's IPC threads stay alive for the app's
            // lifetime.
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                    .context("CoCreateInstance(CUIAutomation)")?;

            let focused = match automation.GetFocusedElement() {
                Ok(e) => e,
                Err(_) => return Ok(None),
            };

            // TextPattern: read selection / move text-range cursors.
            let pattern_unknown = match focused.GetCurrentPattern(UIA_TextPatternId) {
                Ok(p) => p,
                Err(_) => return Ok(None),
            };
            let text_pattern: IUIAutomationTextPattern = match pattern_unknown.cast() {
                Ok(p) => p,
                Err(_) => return Ok(None),
            };

            // Get the current selection. For a bare cursor (no real
            // selection) this is a zero-length range positioned at the
            // caret.
            let selection = match text_pattern.GetSelection() {
                Ok(s) => s,
                Err(_) => return Ok(None),
            };
            let count = selection.Length().unwrap_or(0);
            if count == 0 {
                return Ok(None);
            }
            let range = match selection.GetElement(0) {
                Ok(r) => r,
                Err(_) => return Ok(None),
            };

            // Clone, then move the start of the cloned range backwards
            // by exactly one word. The end stays at the cursor. GetText
            // on that range gives us the previous word.
            let word_range = match range.Clone() {
                Ok(r) => r,
                Err(_) => return Ok(None),
            };
            let moved = word_range
                .MoveEndpointByUnit(
                    windows::Win32::UI::Accessibility::TextPatternRangeEndpoint_Start,
                    TextUnit_Word,
                    -1,
                )
                .unwrap_or(0);
            if moved == 0 {
                return Ok(None);
            }

            let bstr = match word_range.GetText(-1) {
                Ok(s) => s,
                Err(_) => return Ok(None),
            };
            let raw = bstr.to_string();
            let trimmed = trim_word(&raw);
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
    }
}

impl FieldAccess for UiaFieldAccess {
    fn read_word_before_cursor(&self) -> Result<Option<String>> {
        self.read_word()
    }

    /// Windows replace: Backspace × char_count(word) + type the body via
    /// `SendInput`. The caller hides the popup beforehand so the keystrokes
    /// land in the previously focused app, not in ClipSnap.
    ///
    /// Returns `Unsupported` only when the focused element exposes no UIA
    /// TextPattern at all (caller does the keystroke-select fallback);
    /// otherwise `Replaced` — `SendInput` on Windows has none of the
    /// macOS reliability caveats, so the Backspace+type path is trusted.
    fn try_replace_word_before_cursor(&self, replacement: &str) -> Result<ReplaceOutcome> {
        let word = match self.read_word()? {
            Some(w) => w,
            None => return Ok(ReplaceOutcome::Unsupported),
        };
        let backspaces = word.chars().count();

        // `open_prompt_to_get_permissions: false` is a no-op on
        // Windows (macOS-only flag) but we set it everywhere for
        // consistency with the paste / expander modules.
        let settings = Settings {
            open_prompt_to_get_permissions: false,
            ..Settings::default()
        };
        let mut e = Enigo::new(&settings)
            .map_err(|err| anyhow!("enigo init failed: {err:?}"))?;
        for _ in 0..backspaces {
            e.key(Key::Backspace, Press)
                .map_err(|err| anyhow!("backspace press: {err:?}"))?;
            e.key(Key::Backspace, Release)
                .map_err(|err| anyhow!("backspace release: {err:?}"))?;
        }
        e.text(replacement)
            .map_err(|err| anyhow!("type body: {err:?}"))?;
        Ok(ReplaceOutcome::Replaced)
    }
}
