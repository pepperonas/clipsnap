//! `clipsnap-core` — shared, OS-independent app logic for ClipSnap.

mod backup;
mod clipboard_watcher;
mod commands;
mod crypto;
mod db;
mod expander;
mod hotkey;
mod models;
mod notes;
mod cutout;
mod cutout_ml;
mod ocr;
mod paste;
mod recolor;
mod region_picker;
mod screen_picker;
mod screen_recording;
mod seed;
mod settings;
mod snippets;
mod text_field;
mod ui_state;

pub use ui_state::UiState;

use std::sync::atomic::Ordering;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, Wry, WindowEvent,
};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::clipboard_watcher::WatcherState;

pub fn run(context: tauri::Context<Wry>) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = db::default_db_path()?;
            tracing::info!("db at {}", db_path.display());

            // Initialise at-rest encryption *before* opening the DB so
            // every subsequent insert/select runs through the cipher.
            // The data dir is the same parent as the DB file.
            if let Some(data_dir) = db_path.parent() {
                if let Err(e) = crypto::init(data_dir) {
                    tracing::warn!("crypto init failed: {e:#} — DB will be plaintext");
                }
            }

            let db_handle = db::open(&db_path)?;

            snippets::init_table(&db_handle)?;
            notes::init_table(&db_handle)?;
            settings::init_table(&db_handle)?;

            // One-shot migration: rewrite any pre-encryption rows in
            // place so the next read paths through the cipher cleanly.
            // Idempotent — already-encrypted rows are skipped.
            {
                let conn = db_handle.lock();
                let mut total = 0usize;
                for (table, cols) in &[
                    ("entries", &["content_text", "content_data"][..]),
                    ("snippets", &["body"][..]),
                    ("notes", &["content_text", "content_data"][..]),
                ] {
                    match crypto::migrate_table(&conn, table, cols) {
                        Ok(n) => total += n,
                        Err(e) => tracing::warn!("crypto migrate {table}: {e:#}"),
                    }
                }
                if total > 0 {
                    tracing::info!("encrypted {total} legacy plaintext field(s) at startup");
                }
            }

            // First-run: seed the curated default AI-prompt snippets.
            // Idempotent — runs once per database lifetime, then the
            // settings flag prevents re-import. User-deleted snippets
            // stay deleted.
            if let Err(e) = seed::maybe_seed_defaults(&db_handle) {
                tracing::warn!("default snippet seed failed: {e:#}");
            }

            let watcher_state = WatcherState::new();
            let paused = watcher_state.paused.clone();
            let self_written = watcher_state.self_written.clone();

            let ui_state = UiState::default();
            let suppress_hide = ui_state.suppress_hide.clone();

            let expander_state = hotkey::ExpanderShortcutState::default();

            app.manage(db_handle.clone());
            app.manage(watcher_state);
            app.manage(ui_state);
            app.manage(expander_state);

            hotkey::register(&app.handle())?;

            // Restore the expander hotkey from settings if it was enabled
            // last time the app ran. Default is disabled — opt-in. One-time
            // migration bumps the pre-0.12 `Alt+Backquote` default (broken
            // on German ISO Macs) to the layout-stable `Alt+Digit1`.
            {
                let enabled = settings::get_bool(&db_handle, expander::KEY_ENABLED, false)
                    .unwrap_or(false);
                let hotkey_str = expander::migrate_legacy_default(&db_handle);
                let state = app
                    .state::<hotkey::ExpanderShortcutState>();
                if let Err(e) = hotkey::register_expander(
                    &app.handle(),
                    &state,
                    &hotkey_str,
                    enabled,
                ) {
                    tracing::warn!("expander hotkey register failed at startup: {e:#}");
                }
            }

            clipboard_watcher::spawn(app.handle().clone(), db_handle, paused, self_written);

            build_tray(&app.handle())?;

            // Hide from macOS Dock — ClipSnap is a tray-only background app.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let autostart = app.autolaunch();
            let _ = autostart;

            if let Some(window) = app.get_webview_window(hotkey::POPUP_LABEL) {
                let app_handle = app.handle().clone();
                window.on_window_event(move |ev| {
                    if let WindowEvent::Focused(false) = ev {
                        // Don't auto-hide if a modal (e.g., file dialog) is
                        // owning focus — the popup needs to stay visible
                        // until the modal closes.
                        if !suppress_hide.load(Ordering::Relaxed) {
                            hotkey::hide_popup(&app_handle);
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_history,
            commands::search_history,
            commands::paste_entry,
            commands::paste_entry_formatted,
            commands::get_paste_plain_text_only,
            commands::set_paste_plain_text_only,
            commands::delete_entry,
            commands::clear_history,
            commands::toggle_capture,
            commands::get_capture_state,
            commands::hide_popup,
            commands::paste_text,
            commands::list_snippets,
            commands::find_snippets,
            commands::upsert_snippet,
            commands::delete_snippet,
            commands::paste_snippet,
            commands::paste_note_formatted,
            commands::import_snippets,
            commands::import_snippets_from_file,
            commands::restore_default_prompts,
            commands::set_suppress_hide,
            commands::list_notes,
            commands::list_note_categories,
            commands::save_clip_as_note,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::clear_notes,
            commands::paste_note,
            commands::export_backup,
            commands::save_backup_to_file,
            commands::import_backup,
            commands::get_expander_config,
            commands::set_expander_config,
            commands::trigger_expand_at_cursor,
            commands::diagnose_expand_at_cursor,
            commands::get_accessibility_status,
            commands::request_accessibility_grant,
            commands::open_accessibility_settings,
            commands::force_reset_and_request_grant,
            commands::quit_app,
            commands::relaunch_app,
            commands::pick_screen_color,
            commands::recolor_image_entry,
            commands::image_chromaticity,
            commands::cut_out_image_entry,
            commands::cut_out_image_file,
            commands::save_image_entry_to_downloads,
            commands::ocr_region,
            commands::get_screen_recording_status,
            commands::request_screen_recording_grant,
            commands::open_screen_recording_settings,
            commands::force_reset_screen_recording_grant,
        ])
        .run(context)
        .expect("error while running ClipSnap");
}

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let open_item = MenuItemBuilder::with_id("open", "Open (Ctrl+Shift+V)").build(app)?;
    let snippets_item = MenuItemBuilder::with_id("snippets", "Manage Snippets").build(app)?;
    let notes_item = MenuItemBuilder::with_id("notes", "Manage Notes").build(app)?;
    let ocr_label = if cfg!(target_os = "macos") {
        "OCR Region (⌘⇧O)"
    } else {
        "OCR Region (Ctrl+Shift+O)"
    };
    let ocr_item = MenuItemBuilder::with_id("ocr", ocr_label).build(app)?;
    let pause_item = MenuItemBuilder::with_id("pause", "Pause Capture").build(app)?;
    let clear_item = MenuItemBuilder::with_id("clear", "Clear History…").build(app)?;
    let autostart_label = if cfg!(target_os = "windows") { "Start with Windows" } else { "Start at Login" };
    let autostart_item =
        MenuItemBuilder::with_id("autostart", autostart_label).build(app)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let sep_ocr = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit ClipSnap").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[
            &open_item,
            &snippets_item,
            &notes_item,
            &sep_ocr,
            &ocr_item,
            &sep,
            &pause_item,
            &autostart_item,
            &clear_item,
            &sep2,
            &quit_item,
        ])
        .build()?;

    let _tray = TrayIconBuilder::with_id("main")
        .tooltip("ClipSnap")
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "open" => {
                if let Err(e) = hotkey::toggle_popup(app) {
                    tracing::warn!("open from tray: {e:#}");
                }
            }
            "snippets" => {
                if let Err(e) = hotkey::show_popup(app) {
                    tracing::warn!("show popup for snippets: {e:#}");
                }
                let _ = app.emit("open-snippets-tab", ());
            }
            "notes" => {
                if let Err(e) = hotkey::show_popup(app) {
                    tracing::warn!("show popup for notes: {e:#}");
                }
                let _ = app.emit("open-notes-tab", ());
            }
            "ocr" => {
                // Same dispatch model as the global shortcut path:
                // hand off to a worker thread so the menu callback
                // returns immediately and the screencapture overlay
                // doesn't fight the menu close animation.
                let app2 = app.clone();
                std::thread::spawn(move || {
                    match commands::run_ocr_pipeline(&app2) {
                        Ok(r) if !r.cancelled && r.chars > 0 => {
                            tracing::info!("OCR (tray): {} chars", r.chars);
                        }
                        Ok(_) => tracing::debug!("OCR (tray): cancelled or empty"),
                        Err(e) => tracing::warn!("OCR (tray) failed: {e}"),
                    }
                });
            }
            "pause" => {
                if let Some(state) = app.try_state::<WatcherState>() {
                    let now = state.paused.load(Ordering::Relaxed);
                    state.paused.store(!now, Ordering::Relaxed);
                    let _ = app.emit("capture-state-changed", !now);
                }
            }
            "clear" => {
                // Native confirm — the tray menu has no UI surface of its own,
                // so we can't reuse the popup's `window.confirm` flow without
                // first showing the popup. A modal dialog keeps the user where
                // they are; the OK button is destructive on macOS / labeled
                // "Yes" on Windows.
                let app2 = app.clone();
                app.dialog()
                    .message("Delete all clipboard history? This cannot be undone.")
                    .title("ClipSnap")
                    .kind(MessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::OkCancelCustom(
                        "Delete".to_string(),
                        "Cancel".to_string(),
                    ))
                    .show(move |confirmed| {
                        if !confirmed {
                            return;
                        }
                        if let Some(db) = app2.try_state::<db::DbHandle>() {
                            if let Err(e) = db::clear(&db) {
                                tracing::warn!("clear: {e:#}");
                            }
                            let _ = app2.emit("clipboard-changed", ());
                        }
                    });
            }
            "autostart" => {
                let am = app.autolaunch();
                let enabled = am.is_enabled().unwrap_or(false);
                let res = if enabled { am.disable() } else { am.enable() };
                if let Err(e) = res {
                    tracing::warn!("autostart toggle: {e:#}");
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    let _ = MacosLauncher::LaunchAgent;
    Ok(())
}
