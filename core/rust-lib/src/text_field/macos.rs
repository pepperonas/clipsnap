//! macOS Accessibility (AXUIElement) implementation of `FieldAccess`.
//!
//! References:
//! - <https://developer.apple.com/documentation/applicationservices/axuielement_h>
//! - The "Accessibility Programming Guidelines for OS X"
//!
//! Permission requirement: the same `AXIsProcessTrusted()` permission
//! we already require for keystroke synthesis. Granting "Accessibility"
//! to ClipSnap in System Settings unlocks both paths.

use anyhow::{anyhow, Result};
use std::ffi::{c_void, CString};

use super::{trim_word, word_start_before_cursor, FieldAccess, ReplaceOutcome};

// ── Core Foundation FFI ─────────────────────────────────────────────────────

type CFTypeRef = *const c_void;
type CFAllocatorRef = *const c_void;
type CFStringRef = *const c_void;
type CFRange = (CFIndex, CFIndex);
type CFIndex = isize;
type Boolean = u8;

const KCF_STRING_ENCODING_UTF8: u32 = 0x08000100;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFAllocatorDefault: CFAllocatorRef;

    fn CFStringCreateWithCString(
        allocator: CFAllocatorRef,
        c_str: *const i8,
        encoding: u32,
    ) -> CFStringRef;

    fn CFStringGetLength(s: CFStringRef) -> CFIndex;

    fn CFStringGetCString(
        s: CFStringRef,
        buffer: *mut i8,
        buffer_size: CFIndex,
        encoding: u32,
    ) -> Boolean;

    fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;

    fn CFRelease(cf: CFTypeRef);
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
}

// ── ApplicationServices / AXUIElement FFI ───────────────────────────────────

type AXUIElementRef = *const c_void;
type AXValueRef = *const c_void;
type AXError = i32;

const KAX_ERROR_SUCCESS: AXError = 0;

// AXValue type IDs (from AXValue.h).
const KAX_VALUE_CFRANGE_TYPE: u32 = 4;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;

    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        out_value: *mut CFTypeRef,
    ) -> AXError;

    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;

    fn AXValueCreate(value_type: u32, value_ptr: *const c_void) -> AXValueRef;
    fn AXValueGetValue(value: AXValueRef, value_type: u32, value_ptr: *mut c_void) -> Boolean;
    fn AXValueGetType(value: AXValueRef) -> u32;
}

// ── AX attribute name constants (constructed at runtime as CFStrings) ──────

fn cf_string(s: &str) -> CFStringRef {
    let c = CString::new(s).expect("AX attribute name must be ASCII");
    unsafe { CFStringCreateWithCString(kCFAllocatorDefault, c.as_ptr(), KCF_STRING_ENCODING_UTF8) }
}

/// Convert a `CFStringRef` to an owned Rust `String`. Returns `None` if
/// the CF object isn't actually a CFString or if conversion fails.
unsafe fn cf_string_to_rust(cf: CFTypeRef) -> Option<String> {
    if cf.is_null() {
        return None;
    }
    if CFGetTypeID(cf) != CFStringGetTypeID() {
        return None;
    }
    let len = CFStringGetLength(cf);
    let max = CFStringGetMaximumSizeForEncoding(len, KCF_STRING_ENCODING_UTF8) + 1;
    let mut buf = vec![0i8; max as usize];
    let ok = CFStringGetCString(cf, buf.as_mut_ptr(), max, KCF_STRING_ENCODING_UTF8);
    if ok == 0 {
        return None;
    }
    let bytes: Vec<u8> = buf
        .into_iter()
        .take_while(|&b| b != 0)
        .map(|b| b as u8)
        .collect();
    String::from_utf8(bytes).ok()
}

// ── Public impl ─────────────────────────────────────────────────────────────

pub struct AxFieldAccess;

impl AxFieldAccess {
    /// One-shot: read focused element, return (`value`, `cursor_chars`)
    /// where `cursor_chars` is the cursor's start position in UTF-16 code
    /// units (which AX reports), or `None` if the element doesn't expose
    /// the necessary attributes.
    fn read_focused(&self) -> Result<Option<(AXUIElementRef, String, usize)>> {
        unsafe {
            let system = AXUIElementCreateSystemWide();
            if system.is_null() {
                return Err(anyhow!("AXUIElementCreateSystemWide returned null"));
            }

            // 1) Find the focused UI element.
            let attr_focused = cf_string("AXFocusedUIElement");
            let mut focused_value: CFTypeRef = std::ptr::null();
            let err = AXUIElementCopyAttributeValue(system, attr_focused, &mut focused_value);
            CFRelease(attr_focused);
            CFRelease(system);
            if err != KAX_ERROR_SUCCESS || focused_value.is_null() {
                return Ok(None);
            }
            let focused: AXUIElementRef = focused_value;

            // 2) Read kAXValueAttribute (the field's text content).
            let attr_value = cf_string("AXValue");
            let mut value_cf: CFTypeRef = std::ptr::null();
            let err = AXUIElementCopyAttributeValue(focused, attr_value, &mut value_cf);
            CFRelease(attr_value);
            if err != KAX_ERROR_SUCCESS || value_cf.is_null() {
                CFRelease(focused);
                return Ok(None);
            }
            let value_str = cf_string_to_rust(value_cf);
            CFRelease(value_cf);
            let Some(value_str) = value_str else {
                CFRelease(focused);
                return Ok(None);
            };

            // 3) Read kAXSelectedTextRangeAttribute (cursor position +
            //    selection length). It comes back as an AXValue wrapping
            //    a CFRange — location is the cursor position in UTF-16
            //    units when length == 0.
            let attr_range = cf_string("AXSelectedTextRange");
            let mut range_cf: CFTypeRef = std::ptr::null();
            let err = AXUIElementCopyAttributeValue(focused, attr_range, &mut range_cf);
            CFRelease(attr_range);
            if err != KAX_ERROR_SUCCESS || range_cf.is_null() {
                CFRelease(focused);
                return Ok(None);
            }
            // Verify it's actually a CFRange-bearing AXValue.
            if AXValueGetType(range_cf) != KAX_VALUE_CFRANGE_TYPE {
                CFRelease(range_cf);
                CFRelease(focused);
                return Ok(None);
            }
            let mut range: CFRange = (0, 0);
            let got = AXValueGetValue(
                range_cf,
                KAX_VALUE_CFRANGE_TYPE,
                &mut range as *mut _ as *mut c_void,
            );
            CFRelease(range_cf);
            if got == 0 {
                CFRelease(focused);
                return Ok(None);
            }
            let cursor_utf16 = range.0.max(0) as usize;

            // AX reports cursor in UTF-16 code units. Our `value_str` is
            // UTF-8. Convert: walk char_indices, find the char at cursor
            // by counting UTF-16 units. For BMP chars one UTF-16 unit per
            // char; for non-BMP (emoji, …) two. For ASCII text these are
            // the same.
            let cursor_chars = utf16_units_to_char_index(&value_str, cursor_utf16);

            Ok(Some((focused, value_str, cursor_chars)))
        }
    }
}

impl FieldAccess for AxFieldAccess {
    fn read_word_before_cursor(&self) -> Result<Option<String>> {
        let Some((focused, value, cursor_chars)) = self.read_focused()? else {
            return Ok(None);
        };
        unsafe { CFRelease(focused) };
        let start = word_start_before_cursor(&value, cursor_chars);
        // Compute the byte slice [start..cursor-byte] of `value`.
        let cursor_byte: usize = value
            .char_indices()
            .nth(cursor_chars)
            .map(|(i, _)| i)
            .unwrap_or(value.len());
        if start >= cursor_byte {
            return Ok(None);
        }
        let word = trim_word(&value[start..cursor_byte]).to_string();
        if word.is_empty() {
            return Ok(None);
        }
        Ok(Some(word))
    }

    fn try_replace_word_before_cursor(&self, replacement: &str) -> Result<ReplaceOutcome> {
        let Some((focused, value, cursor_chars)) = self.read_focused()? else {
            return Ok(ReplaceOutcome::Unsupported);
        };
        let start_byte = word_start_before_cursor(&value, cursor_chars);
        // The cursor in *char* index → byte index.
        let cursor_byte: usize = value
            .char_indices()
            .nth(cursor_chars)
            .map(|(i, _)| i)
            .unwrap_or(value.len());
        if start_byte >= cursor_byte {
            // Nothing before the cursor — nothing to replace.
            unsafe { CFRelease(focused) };
            return Ok(ReplaceOutcome::Unsupported);
        }

        // Compute the word range as UTF-16 code units (which is what AX
        // expects when we set the selected range).
        let prefix = &value[..start_byte];
        let word = &value[start_byte..cursor_byte];
        let start_utf16: isize = utf16_count(prefix) as isize;
        let length_utf16: isize = utf16_count(word) as isize;

        unsafe {
            // 1) Set kAXSelectedTextRangeAttribute to (start, length) of
            //    the word — this *selects* the abbreviation. If even this
            //    fails the element exposes no settable text attributes;
            //    let the caller do the full keystroke fallback.
            let range = (start_utf16, length_utf16);
            let range_value =
                AXValueCreate(KAX_VALUE_CFRANGE_TYPE, &range as *const _ as *const c_void);
            if range_value.is_null() {
                CFRelease(focused);
                return Ok(ReplaceOutcome::Unsupported);
            }
            let attr_range = cf_string("AXSelectedTextRange");
            let err = AXUIElementSetAttributeValue(focused, attr_range, range_value);
            CFRelease(attr_range);
            CFRelease(range_value);
            if err != KAX_ERROR_SUCCESS {
                CFRelease(focused);
                return Ok(ReplaceOutcome::Unsupported);
            }

            // 2) Set kAXSelectedTextAttribute to the replacement string.
            //    On a well-behaved Cocoa text view this replaces the
            //    selected range in place. On Electron / Chromium /
            //    Mac-Catalyst text views it commonly returns
            //    kAXErrorSuccess but does nothing — so we don't trust the
            //    return code; we verify by re-reading AXValue below.
            let attr_seltext = cf_string("AXSelectedText");
            let replacement_cf = {
                let c = CString::new(replacement.replace('\0', "")).unwrap();
                CFStringCreateWithCString(kCFAllocatorDefault, c.as_ptr(), KCF_STRING_ENCODING_UTF8)
            };
            if replacement_cf.is_null() {
                CFRelease(attr_seltext);
                CFRelease(focused);
                // The range was set in step 1 → the abbreviation is
                // selected → caller can paste over it.
                return Ok(ReplaceOutcome::SelectionActive);
            }
            let _ = AXUIElementSetAttributeValue(focused, attr_seltext, replacement_cf);
            CFRelease(attr_seltext);
            CFRelease(replacement_cf);

            // 3) Verify. Give the app a beat, then re-read AXValue. If the
            //    field's text actually changed, the in-place replace took.
            //    If not, the word is still selected from step 1 — tell the
            //    caller to paste over the selection.
            std::thread::sleep(std::time::Duration::from_millis(15));
            let attr_value = cf_string("AXValue");
            let mut new_value_cf: CFTypeRef = std::ptr::null();
            let verr = AXUIElementCopyAttributeValue(focused, attr_value, &mut new_value_cf);
            CFRelease(attr_value);
            let new_value = if verr == KAX_ERROR_SUCCESS && !new_value_cf.is_null() {
                let s = cf_string_to_rust(new_value_cf);
                CFRelease(new_value_cf);
                s
            } else {
                None
            };
            CFRelease(focused);

            match new_value {
                Some(nv) if nv != value => Ok(ReplaceOutcome::Replaced),
                _ => Ok(ReplaceOutcome::SelectionActive),
            }
        }
    }
}

// ── UTF-16 helpers ──────────────────────────────────────────────────────────

fn utf16_count(s: &str) -> usize {
    s.chars().map(|c| c.len_utf16()).sum()
}

/// Convert a UTF-16 code-unit offset into a Rust char index.
fn utf16_units_to_char_index(s: &str, target_units: usize) -> usize {
    let mut units = 0;
    for (char_idx, c) in s.chars().enumerate() {
        if units >= target_units {
            return char_idx;
        }
        units += c.len_utf16();
    }
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_units_ascii_match_chars() {
        assert_eq!(utf16_units_to_char_index("hello", 3), 3);
        assert_eq!(utf16_count("hello"), 5);
    }

    #[test]
    fn utf16_units_count_supplementary_as_two() {
        // 🚀 is U+1F680 — outside BMP, takes 2 UTF-16 units.
        let s = "🚀abc";
        // Cursor in UTF-16 units = 2 → after the rocket → char index 1.
        assert_eq!(utf16_units_to_char_index(s, 2), 1);
        // Cursor in UTF-16 units = 4 → after "🚀ab" → char index 3.
        assert_eq!(utf16_units_to_char_index(s, 4), 3);
        assert_eq!(utf16_count("🚀abc"), 5); // 2 + 3
    }

    #[test]
    fn utf16_units_handle_german_umlauts_as_single_unit() {
        // Größe — ö is BMP, single UTF-16 unit.
        assert_eq!(utf16_count("Größe"), 5);
        assert_eq!(utf16_units_to_char_index("Größe", 5), 5);
    }
}
