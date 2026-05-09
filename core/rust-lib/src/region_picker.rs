//! Interactive screen-region picker for the OCR feature.
//!
//! macOS uses Apple's own `screencapture(1)` with `-i -t png` — Cmd+Shift+4
//! is *literally* this binary, so the UX is the polished one users
//! already know: drag a marquee, Esc cancels, hold Space to drag the
//! whole rect, etc. Way more reliable than reinventing an overlay
//! window in `objc2`. The captured PNG is written to a temp file we
//! then read back into memory and delete.
//!
//! Windows is stubbed for now — implementation will use either the
//! `ms-screenclip:` URI handler or a direct GDI overlay later.

// Context is used by the macOS implementation only; Linux / Windows
// stubs don't need it. Per-platform import keeps clippy happy on all
// targets without sprinkling allow attributes.
#[cfg(target_os = "macos")]
use anyhow::Context;
use anyhow::Result;

/// User pressed Esc / clicked away — distinct error so the IPC layer
/// can return success-with-no-text instead of bubbling up a real error.
#[derive(Debug)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "region capture cancelled")
    }
}

impl std::error::Error for Cancelled {}

/// Show the interactive region picker, return the captured PNG bytes.
/// Blocks until the user finishes drawing the rect or cancels (Esc).
pub fn capture() -> Result<Vec<u8>> {
    capture_impl()
}

#[cfg(target_os = "macos")]
fn capture_impl() -> Result<Vec<u8>> {
    use chrono::Utc;
    use std::process::Command;

    let tmp = std::env::temp_dir().join(format!(
        "clipsnap-ocr-{}.png",
        Utc::now().timestamp_millis()
    ));

    // -i = interactive selection (drag rectangle)
    // -t png = output format (default is also PNG, but be explicit)
    // -x = silent (no shutter sound)
    // -o = no shadow/window chrome capture (irrelevant for region but harmless)
    // We do NOT pass `-c` (clipboard) because we want the file to read
    // back; -c would leave us guessing at the clipboard format.
    let status = Command::new("/usr/sbin/screencapture")
        .args(["-i", "-x", "-t", "png"])
        .arg(&tmp)
        .status()
        .context("spawn /usr/sbin/screencapture")?;

    if !status.success() {
        // Clean up if anything was written despite non-zero exit.
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("screencapture exited with status {:?}", status.code());
    }

    // screencapture exits 0 even on cancel — the only signal is "did
    // the file get written?". A zero-byte file is also considered a
    // cancel (some macOS versions create the file then write nothing).
    if !tmp.exists() {
        return Err(Cancelled.into());
    }
    let bytes = std::fs::read(&tmp).context("read captured png")?;
    let _ = std::fs::remove_file(&tmp);
    if bytes.is_empty() {
        return Err(Cancelled.into());
    }
    Ok(bytes)
}

#[cfg(target_os = "windows")]
fn capture_impl() -> Result<Vec<u8>> {
    // Windows region capture is intentionally postponed — implementing
    // it cleanly needs either the `ms-screenclip:` URI handler (which
    // shells out to the OS Snipping Tool and copies to clipboard, then
    // we read it back), or a custom GDI fullscreen overlay similar to
    // the one in `screen_picker::windows_impl`. Both are non-trivial
    // and OCR shipped first on macOS where Vision is the right backend.
    anyhow::bail!("region capture is not yet implemented on Windows")
}

// Catch-all for Linux / other Unixes so the workspace builds in CI.
// Region capture would need an X11 / Wayland-specific overlay; not
// shipped yet but explicitly handled so cargo doesn't error out at
// link time on Linux runners.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn capture_impl() -> Result<Vec<u8>> {
    anyhow::bail!("region capture is not implemented on this platform")
}
