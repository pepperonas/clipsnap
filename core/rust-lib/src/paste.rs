use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext, RustImageData};
use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::thread;
use std::time::Duration;

use crate::models::{ClipEntry, ContentType};

/// Build enigo `Settings` with `open_prompt_to_get_permissions = false`.
///
/// **Critical macOS behaviour.** `Settings::default()` ships with
/// `open_prompt_to_get_permissions = true`, which makes `Enigo::new()`
/// internally call `AXIsProcessTrustedWithOptions` *with the prompt
/// option enabled*. Every `Enigo::new()` call on an untrusted process
/// then triggers the standard "would like to control" dialog as a side
/// effect — exactly the noise we hit after every cdhash-changing
/// rebuild. With this flag flipped to `false`, enigo silently returns
/// `NewConError::NoPermission` and we surface the permission state via
/// our own UI instead.
fn enigo_settings() -> Settings {
    Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    }
}

/// Write `entry` to the OS clipboard, then simulate Cmd+V (macOS) / Ctrl+V
/// (Windows / Linux) to paste into the window that had focus before the
/// popup opened. Caller should hide the popup (and on macOS the whole app)
/// *before* calling this so focus returns to the previous app.
pub fn paste_entry(entry: &ClipEntry) -> Result<()> {
    paste_payload(entry.content_type, &entry.content_data, &entry.content_text)
}

/// Write a typed payload to the clipboard and synthesize the paste shortcut.
///
/// `data` carries the type-specific raw payload (raw text for Text/Html/Rtf,
/// base64 PNG for Image, JSON array of paths for Files). `text` is a plain
/// fallback used by the Files branch (which clipboard-rs cannot set as a
/// real file list on all platforms).
pub fn paste_payload(content_type: ContentType, data: &str, text: &str) -> Result<()> {
    write_to_clipboard(content_type, data, text)?;
    thread::sleep(focus_settle_delay());
    send_paste_shortcut()?;
    Ok(())
}

fn write_to_clipboard(content_type: ContentType, data: &str, text: &str) -> Result<()> {
    let ctx = ClipboardContext::new()
        .map_err(|e| anyhow!("clipboard ctx init failed: {e:?}"))?;

    match content_type {
        ContentType::Text => {
            ctx.set_text(data.to_string())
                .map_err(|e| anyhow!("set_text failed: {e:?}"))?;
        }
        ContentType::Html => {
            ctx.set_html(data.to_string())
                .map_err(|e| anyhow!("set_html failed: {e:?}"))?;
        }
        ContentType::Rtf => {
            ctx.set_rich_text(data.to_string())
                .map_err(|e| anyhow!("set_rich_text failed: {e:?}"))?;
        }
        ContentType::Image => {
            let bytes = B64
                .decode(data.as_bytes())
                .context("decode image base64")?;
            let img = RustImageData::from_bytes(&bytes)
                .map_err(|e| anyhow!("decode png failed: {e:?}"))?;
            ctx.set_image(img)
                .map_err(|e| anyhow!("set_image failed: {e:?}"))?;
        }
        ContentType::Files => {
            // clipboard-rs does not currently support setting file lists on
            // all platforms; fall back to joining paths as text.
            ctx.set_text(text.to_string())
                .map_err(|e| anyhow!("set_text (files fallback) failed: {e:?}"))?;
        }
    }
    Ok(())
}

/// Write plain text to the OS clipboard, then simulate the paste shortcut.
pub fn paste_text(text: &str) -> Result<()> {
    let ctx = ClipboardContext::new()
        .map_err(|e| anyhow!("clipboard ctx init failed: {e:?}"))?;
    ctx.set_text(text.to_string())
        .map_err(|e| anyhow!("set_text failed: {e:?}"))?;
    thread::sleep(focus_settle_delay());
    send_paste_shortcut()?;
    Ok(())
}

/// How long to wait between hiding the popup and synthesizing the paste
/// keystroke, so the OS has a chance to restore focus to the previously
/// active app. macOS `NSApp.hide()` takes a frame or two to take effect;
/// Windows is much faster.
fn focus_settle_delay() -> Duration {
    #[cfg(target_os = "macos")]
    {
        Duration::from_millis(120)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Duration::from_millis(50)
    }
}

fn send_paste_shortcut() -> Result<()> {
    let mut enigo = Enigo::new(&enigo_settings())
        .map_err(|e| anyhow!("enigo init failed: {e:?}"))?;

    // macOS paste = Cmd+V; Windows/Linux paste = Ctrl+V.
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    enigo
        .key(modifier, Press)
        .map_err(|e| anyhow!("modifier press: {e:?}"))?;
    enigo
        .key(Key::Unicode('v'), Press)
        .map_err(|e| anyhow!("v press: {e:?}"))?;
    enigo
        .key(Key::Unicode('v'), Release)
        .map_err(|e| anyhow!("v release: {e:?}"))?;
    enigo
        .key(modifier, Release)
        .map_err(|e| anyhow!("modifier release: {e:?}"))?;
    Ok(())
}
