use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, State};

use crate::backup::{self, BackupImportResult};
use crate::clipboard_watcher::WatcherState;
use crate::db::{self, DbHandle};
use crate::expander;
use crate::hotkey::{self, ExpanderShortcutState};
use crate::models::ClipEntry;
use crate::notes::{self, Note};
use crate::paste;
use crate::seed;
use crate::settings;
use crate::snippets::{self, ImportResult, Snippet};
use crate::ui_state::UiState;

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ── Clipboard history ────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_history(
    db: State<'_, DbHandle>,
    limit: usize,
    offset: usize,
) -> Result<Vec<ClipEntry>, String> {
    db::list(&db, limit, offset).map_err(map_err)
}

#[tauri::command]
pub fn search_history(
    db: State<'_, DbHandle>,
    query: String,
    limit: usize,
) -> Result<Vec<ClipEntry>, String> {
    let all = db::list(&db, 1000, 0).map_err(map_err)?;
    let q = query.to_lowercase();
    if q.is_empty() {
        return Ok(all.into_iter().take(limit).collect());
    }
    let filtered: Vec<_> = all
        .into_iter()
        .filter(|e| e.content_text.to_lowercase().contains(&q))
        .take(limit)
        .collect();
    Ok(filtered)
}

/// Settings key controlling whether HTML / RTF clipboard entries get
/// downgraded to plain text on paste. Defaults to `true` — most users
/// want to drop the source app's styling when pasting elsewhere.
const KEY_PLAIN_TEXT_ONLY: &str = "paste.plain_text_only";

/// Sentinel error string the frontend recognises and presents as the
/// "Accessibility access required" toast. Kept stable so the JS side
/// can switch on it without parsing localized text.
const ERR_NO_ACCESSIBILITY: &str = "ax.permission_denied";

/// Bail-out helper: returns `Err(ERR_NO_ACCESSIBILITY)` when
/// `accessibility_granted()` is false, so paste IPCs short-circuit
/// before reaching enigo. Without this guard, paste actions on an
/// untrusted process would just silently no-op (because we now pass
/// `open_prompt_to_get_permissions = false` to enigo) — the user
/// wouldn't know why nothing happened. With this guard, the frontend
/// gets a structured error and can show a helpful toast.
fn require_accessibility() -> Result<(), String> {
    if expander::accessibility_granted() {
        Ok(())
    } else {
        Err(ERR_NO_ACCESSIBILITY.to_string())
    }
}

/// Default behaviour: respects the `paste.plain_text_only` setting. For
/// HTML / RTF entries with the setting on, pastes the plain-text
/// preview (`content_text`) instead of the formatted payload.
/// Image / Files entries are unaffected — they're always pasted as-is.
#[tauri::command]
pub fn paste_entry(
    app: AppHandle,
    db: State<'_, DbHandle>,
    watcher: State<'_, WatcherState>,
    id: i64,
) -> Result<(), String> {
    require_accessibility()?;
    let entry = db::get(&db, id)
        .map_err(map_err)?
        .ok_or_else(|| "entry not found".to_string())?;

    let plain_only = settings::get_bool(&db, KEY_PLAIN_TEXT_ONLY, true).unwrap_or(true);

    hotkey::hide_popup(&app);
    if plain_only
        && matches!(
            entry.content_type,
            crate::models::ContentType::Html | crate::models::ContentType::Rtf
        )
    {
        // Mark + write the plain-text downgrade so the watcher skips
        // capturing this back as a duplicate Text clip.
        watcher.mark_self_write(crate::models::ContentType::Text, &entry.content_text);
        paste::paste_text(&entry.content_text).map_err(map_err)?;
    } else {
        watcher.mark_self_write(entry.content_type, &entry.content_data);
        paste::paste_entry(&entry).map_err(map_err)?;
    }
    db::touch(&db, id).map_err(map_err)?;
    Ok(())
}

/// Read the current value of `paste.plain_text_only` (default `true`).
#[tauri::command]
pub fn get_paste_plain_text_only(db: State<'_, DbHandle>) -> Result<bool, String> {
    settings::get_bool(&db, KEY_PLAIN_TEXT_ONLY, true).map_err(map_err)
}

/// Persist a new value for `paste.plain_text_only`.
#[tauri::command]
pub fn set_paste_plain_text_only(
    db: State<'_, DbHandle>,
    value: bool,
) -> Result<(), String> {
    settings::set(
        &db,
        KEY_PLAIN_TEXT_ONLY,
        if value { "true" } else { "false" },
    )
    .map_err(map_err)
}

/// Force-format paste — bypasses the `paste.plain_text_only` setting and
/// always uses the entry's original content type. Wired to Shift+Enter
/// in the popup as a one-shot override for users who normally paste as
/// plain text but want to keep formatting *this* time.
#[tauri::command]
pub fn paste_entry_formatted(
    app: AppHandle,
    db: State<'_, DbHandle>,
    watcher: State<'_, WatcherState>,
    id: i64,
) -> Result<(), String> {
    require_accessibility()?;
    let entry = db::get(&db, id)
        .map_err(map_err)?
        .ok_or_else(|| "entry not found".to_string())?;

    hotkey::hide_popup(&app);
    watcher.mark_self_write(entry.content_type, &entry.content_data);
    paste::paste_entry(&entry).map_err(map_err)?;
    db::touch(&db, id).map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn delete_entry(db: State<'_, DbHandle>, id: i64) -> Result<(), String> {
    db::delete(&db, id).map_err(map_err)
}

#[tauri::command]
pub fn clear_history(db: State<'_, DbHandle>) -> Result<(), String> {
    db::clear(&db).map_err(map_err)
}

#[tauri::command]
pub fn toggle_capture(state: State<'_, WatcherState>, paused: bool) -> Result<(), String> {
    state.paused.store(paused, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn get_capture_state(state: State<'_, WatcherState>) -> bool {
    state.paused.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn hide_popup(app: AppHandle) -> Result<(), String> {
    hotkey::hide_popup(&app);
    Ok(())
}

/// Hide the popup, write `text` to the clipboard, and synthesize the paste
/// shortcut. Used by the inline calculator (and any other "compute and
/// paste" flow). The freshly-written clipboard entry would normally be
/// picked up by the watcher and recorded in history; we mark the write
/// so the watcher skips that one event — calc/color results aren't worth
/// adding to history (they're cheap to recompute).
#[tauri::command]
pub fn paste_text(
    app: AppHandle,
    watcher: State<'_, WatcherState>,
    text: String,
) -> Result<(), String> {
    require_accessibility()?;
    hotkey::hide_popup(&app);
    watcher.mark_self_write(crate::models::ContentType::Text, &text);
    paste::paste_text(&text).map_err(map_err)
}

/// Toggle the popup's hide-on-blur behaviour. The frontend sets this to
/// `true` before opening a modal child window (file dialog) so the popup
/// stays visible while the modal owns focus, then resets to `false` once
/// the modal is dismissed.
#[tauri::command]
pub fn set_suppress_hide(state: State<'_, UiState>, suppress: bool) -> Result<(), String> {
    state.suppress_hide.store(suppress, Ordering::Relaxed);
    Ok(())
}

// ── Snippets ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_snippets(db: State<'_, DbHandle>) -> Result<Vec<Snippet>, String> {
    snippets::list_all(&db).map_err(map_err)
}

#[tauri::command]
pub fn find_snippets(
    db: State<'_, DbHandle>,
    query: String,
) -> Result<Vec<Snippet>, String> {
    snippets::find_by_query(&db, &query).map_err(map_err)
}

/// Create (id = null) or update (id = some) a snippet.
#[tauri::command]
pub fn upsert_snippet(
    db: State<'_, DbHandle>,
    id: Option<i64>,
    abbreviation: String,
    title: String,
    body: String,
) -> Result<i64, String> {
    match id {
        None => snippets::create(&db, &abbreviation, &title, &body).map_err(map_err),
        Some(existing_id) => {
            snippets::update(&db, existing_id, &abbreviation, &title, &body)
                .map_err(map_err)?;
            Ok(existing_id)
        }
    }
}

#[tauri::command]
pub fn delete_snippet(db: State<'_, DbHandle>, id: i64) -> Result<(), String> {
    snippets::delete(&db, id).map_err(map_err)
}

/// Paste a snippet: hide the popup, write body to clipboard, simulate Ctrl+V.
#[tauri::command]
pub fn paste_snippet(
    app: AppHandle,
    db: State<'_, DbHandle>,
    watcher: State<'_, WatcherState>,
    id: i64,
) -> Result<(), String> {
    require_accessibility()?;
    let snippet = snippets::list_all(&db)
        .map_err(map_err)?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| "snippet not found".to_string())?;

    hotkey::hide_popup(&app);
    watcher.mark_self_write(crate::models::ContentType::Text, &snippet.body);
    paste::paste_text(&snippet.body).map_err(map_err)?;
    Ok(())
}

/// Import snippets from a JSON document. Existing rows with the same
/// abbreviation are overwritten. Per-row errors are returned in the result
/// instead of aborting the whole import.
#[tauri::command]
pub fn import_snippets(
    db: State<'_, DbHandle>,
    json: String,
) -> Result<ImportResult, String> {
    snippets::import_from_json(&db, &json).map_err(map_err)
}

/// Read a JSON file from disk and import its snippets. Path is supplied by
/// the frontend after the user picked a file via the native dialog plugin.
#[tauri::command]
pub fn import_snippets_from_file(
    db: State<'_, DbHandle>,
    path: String,
) -> Result<ImportResult, String> {
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("read {path}: {e}"))?;
    snippets::import_from_json(&db, &json).map_err(map_err)
}

/// Re-import the bundled default AI-prompt snippets. Existing rows
/// sharing an `abbreviation` get overwritten; user snippets with
/// distinct abbreviations are untouched. Surfaced via the Snippets-tab
/// "Restore defaults" button.
#[tauri::command]
pub fn restore_default_prompts(db: State<'_, DbHandle>) -> Result<ImportResult, String> {
    seed::restore_defaults(&db).map_err(map_err)
}

// ── Notes ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_notes(db: State<'_, DbHandle>) -> Result<Vec<Note>, String> {
    notes::list_all(&db).map_err(map_err)
}

#[tauri::command]
pub fn list_note_categories(db: State<'_, DbHandle>) -> Result<Vec<String>, String> {
    notes::list_categories(&db).map_err(map_err)
}

/// Promote a clipboard entry to a persistent note. Returns the note's id.
/// Errors if the clip no longer exists (e.g. just got pruned).
#[tauri::command]
pub fn save_clip_as_note(
    db: State<'_, DbHandle>,
    clip_id: i64,
    title: String,
    category: String,
) -> Result<i64, String> {
    notes::save_from_clip(&db, clip_id, &title, &category)
        .map_err(map_err)?
        .ok_or_else(|| "clipboard entry not found".to_string())
}

#[tauri::command]
pub fn create_note(
    db: State<'_, DbHandle>,
    title: String,
    body: String,
    category: String,
) -> Result<i64, String> {
    notes::create_text(&db, &title, &body, &category).map_err(map_err)
}

#[tauri::command]
pub fn update_note(
    db: State<'_, DbHandle>,
    id: i64,
    title: String,
    body: String,
    category: String,
) -> Result<(), String> {
    notes::update(&db, id, &title, &body, &category).map_err(map_err)
}

#[tauri::command]
pub fn delete_note(db: State<'_, DbHandle>, id: i64) -> Result<(), String> {
    notes::delete(&db, id).map_err(map_err)
}

#[tauri::command]
pub fn clear_notes(db: State<'_, DbHandle>) -> Result<(), String> {
    notes::clear_all(&db).map_err(map_err)
}

/// Paste a note. Honours the `paste.plain_text_only` setting in the same
/// way `paste_entry` does: HTML / RTF notes get downgraded to their
/// plain-text preview when the toggle is on. Image / Files notes paste
/// as-is regardless.
#[tauri::command]
pub fn paste_note(
    app: AppHandle,
    db: State<'_, DbHandle>,
    watcher: State<'_, WatcherState>,
    id: i64,
) -> Result<(), String> {
    require_accessibility()?;
    let note = notes::get(&db, id)
        .map_err(map_err)?
        .ok_or_else(|| "note not found".to_string())?;

    let plain_only = settings::get_bool(&db, KEY_PLAIN_TEXT_ONLY, true).unwrap_or(true);

    hotkey::hide_popup(&app);
    if plain_only
        && matches!(
            note.content_type,
            crate::models::ContentType::Html | crate::models::ContentType::Rtf
        )
    {
        watcher.mark_self_write(crate::models::ContentType::Text, &note.content_text);
        paste::paste_text(&note.content_text).map_err(map_err)
    } else {
        watcher.mark_self_write(note.content_type, &note.content_data);
        paste::paste_payload(note.content_type, &note.content_data, &note.content_text)
            .map_err(map_err)
    }
}

/// Force-format paste for notes — bypasses the plain-text setting and
/// uses the note's original content type. Mirrors `paste_entry_formatted`
/// for symmetry; expose to the frontend if a Shift+click override on
/// the Notes-tab Paste button is wanted in a future iteration.
#[tauri::command]
pub fn paste_note_formatted(
    app: AppHandle,
    db: State<'_, DbHandle>,
    watcher: State<'_, WatcherState>,
    id: i64,
) -> Result<(), String> {
    require_accessibility()?;
    let note = notes::get(&db, id)
        .map_err(map_err)?
        .ok_or_else(|| "note not found".to_string())?;

    hotkey::hide_popup(&app);
    watcher.mark_self_write(note.content_type, &note.content_data);
    paste::paste_payload(note.content_type, &note.content_data, &note.content_text)
        .map_err(map_err)
}

// ── Backup (full app export / import) ────────────────────────────────────────

/// Build a backup JSON document. Each section (history / snippets /
/// notes) is included only if the corresponding flag is `true` — lets
/// the user opt out of, say, exporting their clipboard history when
/// sharing snippets with a colleague. Defaults to *all true* if invoked
/// without the flags (legacy callers).
#[tauri::command]
pub fn export_backup(
    db: State<'_, DbHandle>,
    include_history: Option<bool>,
    include_snippets: Option<bool>,
    include_notes: Option<bool>,
) -> Result<String, String> {
    let opts = backup::ExportOptions {
        include_history: include_history.unwrap_or(true),
        include_snippets: include_snippets.unwrap_or(true),
        include_notes: include_notes.unwrap_or(true),
    };
    backup::export_json(&db, opts).map_err(map_err)
}

/// Convenience: build the backup JSON and write it directly to `path`.
/// Returns the number of bytes written. Same selective semantics as
/// `export_backup`.
#[tauri::command]
pub fn save_backup_to_file(
    db: State<'_, DbHandle>,
    path: String,
    include_history: Option<bool>,
    include_snippets: Option<bool>,
    include_notes: Option<bool>,
) -> Result<usize, String> {
    let opts = backup::ExportOptions {
        include_history: include_history.unwrap_or(true),
        include_snippets: include_snippets.unwrap_or(true),
        include_notes: include_notes.unwrap_or(true),
    };
    let json = backup::export_json(&db, opts).map_err(map_err)?;
    std::fs::write(&path, &json).map_err(|e| format!("write {path}: {e}"))?;
    Ok(json.len())
}

/// Read a backup JSON file from `path` and merge it into the live database
/// (snippets upsert by abbreviation, history dedupes by hash, notes are
/// appended).
#[tauri::command]
pub fn import_backup(
    db: State<'_, DbHandle>,
    path: String,
) -> Result<BackupImportResult, String> {
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("read {path}: {e}"))?;
    backup::import_json(&db, &json).map_err(map_err)
}

// ── Text expander ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ExpanderConfig {
    pub enabled: bool,
    pub hotkey: String,
    /// Whether the OS-level synthetic-input permission is granted.
    /// macOS: Accessibility. Other OSes: always `true`.
    pub accessibility_granted: bool,
}

/// Read the expander config from the settings table, applying defaults
/// for any missing key. Used by the frontend on Settings panel mount.
#[tauri::command]
pub fn get_expander_config(db: State<'_, DbHandle>) -> Result<ExpanderConfig, String> {
    let enabled = settings::get_bool(&db, expander::KEY_ENABLED, false).map_err(map_err)?;
    let hotkey = settings::get_or(&db, expander::KEY_HOTKEY, expander::DEFAULT_HOTKEY)
        .map_err(map_err)?;
    Ok(ExpanderConfig {
        enabled,
        hotkey,
        accessibility_granted: expander::accessibility_granted(),
    })
}

/// Probe whether ClipSnap currently has Accessibility access. Cheap; safe
/// to call repeatedly (e.g. polling from the Settings panel after the
/// user grants in System Settings).
#[tauri::command]
pub fn get_accessibility_status() -> bool {
    expander::accessibility_granted()
}

/// Trigger the macOS "would like to control this computer" dialog and
/// add ClipSnap to the Accessibility list. Returns the (still-likely-false)
/// trusted status immediately after the prompt fires.
#[tauri::command]
pub fn request_accessibility_grant() -> bool {
    expander::request_accessibility_grant()
}

/// Open the System Settings → Privacy & Security → Accessibility pane
/// (macOS only). On other OSes this is a no-op.
#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    expander::open_accessibility_settings().map_err(map_err)
}

/// Wipe stale TCC entries for ClipSnap and fire the system Accessibility
/// prompt with the current cdhash. Used when the user has the toggle
/// "on" in System Settings but the running process still sees itself as
/// untrusted (the typical "stale grant from a previous build" state).
#[tauri::command]
pub fn force_reset_and_request_grant() -> Result<bool, String> {
    expander::force_reset_and_request_grant().map_err(map_err)
}

/// Quit the running app process. Intended for the Settings panel's
/// "Quit ClipSnap" button after the user grants Accessibility — macOS
/// caches `AXIsProcessTrusted()` per-process, so a freshly granted app
/// stays "untrusted" until restarted.
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// Relaunch ClipSnap by spawning a fresh instance of the installed `.app`
/// and exiting the current process. Used by the Settings panel's auto-
/// restart prompt after the user grants Accessibility — `open` returns
/// immediately, the new ClipSnap process inherits the just-granted TCC
/// state, and the old process exits cleanly.
///
/// macOS-only meaningful behaviour. On other platforms it just exits.
#[tauri::command]
pub fn relaunch_app(app: AppHandle) {
    #[cfg(target_os = "macos")]
    {
        // Detach `open` so the spawned process is fully owned by launchd —
        // not by us — and survives the `app.exit(0)` that follows.
        let _ = std::process::Command::new("open")
            .arg("-n") // -n: open a new instance even if one is already running
            .arg("/Applications/ClipSnap.app")
            .spawn();
        // Tiny delay so `open` has a chance to actually fork before we exit.
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
    app.exit(0);
}

/// Persist a new expander config and re-register the global hotkey.
/// Returns the (now-effective) config so the frontend can confirm what
/// actually got applied — if the hotkey string was malformed, the function
/// errors *before* writing settings, leaving the previous registration in
/// place.
#[tauri::command]
pub fn set_expander_config(
    app: AppHandle,
    db: State<'_, DbHandle>,
    state: State<'_, ExpanderShortcutState>,
    enabled: bool,
    hotkey: String,
) -> Result<ExpanderConfig, String> {
    // Re-register first — if the hotkey is invalid, this fails and we
    // don't touch the persisted settings.
    hotkey::register_expander(&app, &state, &hotkey, enabled).map_err(map_err)?;

    settings::set(&db, expander::KEY_HOTKEY, &hotkey).map_err(map_err)?;
    settings::set(
        &db,
        expander::KEY_ENABLED,
        if enabled { "true" } else { "false" },
    )
    .map_err(map_err)?;

    Ok(ExpanderConfig {
        enabled,
        hotkey,
        accessibility_granted: expander::accessibility_granted(),
    })
}

/// Trigger an expand-at-cursor cycle programmatically (no hotkey press).
/// Hides the popup first so the synthetic Cmd+Shift+← / Cmd+C / Cmd+V
/// land in the previously focused app instead of ClipSnap itself.
///
/// Dispatches the enigo work to the **main thread** because enigo's macOS
/// `Key::Unicode(...)` mapping uses TSM (Text Services Manager) which
/// asserts main-thread, and dies with EXC_BREAKPOINT otherwise.
#[tauri::command]
pub fn trigger_expand_at_cursor(app: AppHandle) -> Result<(), String> {
    hotkey::hide_popup(&app);
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        // Give macOS a moment to hand key focus back to the prior app
        // before we start synthesizing keystrokes.
        std::thread::sleep(std::time::Duration::from_millis(250));
        if let Some(db) = app2.try_state::<DbHandle>() {
            if let Err(e) = expander::expand_at_cursor(&db) {
                tracing::warn!("expand_at_cursor failed: {e:#}");
            }
        }
    })
    .map_err(|e| format!("dispatch to main thread: {e}"))?;
    Ok(())
}

/// Diagnose the capture half of expansion: select previous word, copy,
/// look up — but **don't paste**. Returns what was captured and whether
/// any snippet matches. Used by the Settings panel's "Test now" button.
///
/// Same main-thread requirement as `trigger_expand_at_cursor`. Uses a
/// blocking `mpsc` to ferry the result back from the main-thread closure
/// to the IPC handler thread.
#[tauri::command]
pub fn diagnose_expand_at_cursor(
    app: AppHandle,
) -> Result<expander::DiagnoseResult, String> {
    hotkey::hide_popup(&app);
    let app2 = app.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        std::thread::sleep(std::time::Duration::from_millis(250));
        let result = match app2.try_state::<DbHandle>() {
            Some(db) => expander::diagnose_at_cursor(&db).map_err(|e| e.to_string()),
            None => Err("db state not initialized".to_string()),
        };
        let _ = tx.send(result);
    })
    .map_err(|e| format!("dispatch to main thread: {e}"))?;
    rx.recv()
        .map_err(|e| format!("main thread didn't reply: {e}"))?
}
