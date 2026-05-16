import { invoke } from "@tauri-apps/api/core";
import type { BackupImportResult, ClipEntry, Note, Snippet } from "./types";

// ── Clipboard history ────────────────────────────────────────────────────────

export function getHistory(limit = 500, offset = 0): Promise<ClipEntry[]> {
  return invoke("get_history", { limit, offset });
}

export function searchHistory(query: string, limit = 500): Promise<ClipEntry[]> {
  return invoke("search_history", { query, limit });
}

/** Paste a clipboard entry. Honours the `paste.plain_text_only` setting:
 *  HTML / RTF entries are downgraded to their plain-text preview when
 *  the toggle is on. Image / Files entries paste as-is. */
export function pasteEntry(id: number): Promise<void> {
  return invoke("paste_entry", { id });
}

/** Paste a clipboard entry preserving its original content type. Bypasses
 *  the plain-text setting — used by Shift+Enter as a one-shot override. */
export function pasteEntryFormatted(id: number): Promise<void> {
  return invoke("paste_entry_formatted", { id });
}

export function getPastePlainTextOnly(): Promise<boolean> {
  return invoke("get_paste_plain_text_only");
}

export function setPastePlainTextOnly(value: boolean): Promise<void> {
  return invoke("set_paste_plain_text_only", { value });
}

export function deleteEntry(id: number): Promise<void> {
  return invoke("delete_entry", { id });
}

export function clearHistory(): Promise<void> {
  return invoke("clear_history");
}

export function toggleCapture(paused: boolean): Promise<void> {
  return invoke("toggle_capture", { paused });
}

export function getCaptureState(): Promise<boolean> {
  return invoke("get_capture_state");
}

export function hidePopup(): Promise<void> {
  return invoke("hide_popup");
}

/** Write `text` to the OS clipboard and paste it into the previously
 *  active app. Used by the inline calculator. */
export function pasteText(text: string): Promise<void> {
  return invoke("paste_text", { text });
}

/** Tell the backend to (not) auto-hide the popup on blur. Use while a
 *  native modal (file dialog) is open, then reset to `false`. */
export function setSuppressHide(suppress: boolean): Promise<void> {
  return invoke("set_suppress_hide", { suppress });
}

// ── Snippets ─────────────────────────────────────────────────────────────────

export function listSnippets(): Promise<Snippet[]> {
  return invoke("list_snippets");
}

export function findSnippets(query: string): Promise<Snippet[]> {
  return invoke("find_snippets", { query });
}

/** Pass id = null to create, id = number to update. Returns the snippet id. */
export function upsertSnippet(
  id: number | null,
  abbreviation: string,
  title: string,
  body: string,
): Promise<number> {
  return invoke("upsert_snippet", { id, abbreviation, title, body });
}

export function deleteSnippet(id: number): Promise<void> {
  return invoke("delete_snippet", { id });
}

export function pasteSnippet(id: number): Promise<void> {
  return invoke("paste_snippet", { id });
}

export interface ImportResult {
  imported: number;
  skipped: number;
  errors: string[];
}

/** Import snippets from a JSON string. Existing abbreviations get overwritten. */
export function importSnippets(json: string): Promise<ImportResult> {
  return invoke("import_snippets", { json });
}

/** Read a JSON file from the given path and import its snippets. */
export function importSnippetsFromFile(path: string): Promise<ImportResult> {
  return invoke("import_snippets_from_file", { path });
}

/** Re-import the bundled default AI-prompt snippets. Existing snippets
 *  sharing an `abbreviation` get overwritten; user-added snippets with
 *  distinct abbreviations are untouched. Surfaced via the Snippets-tab
 *  "Restore defaults" button. */
export function restoreDefaultPrompts(): Promise<ImportResult> {
  return invoke("restore_default_prompts");
}

// ── Notes ────────────────────────────────────────────────────────────────────

export function listNotes(): Promise<Note[]> {
  return invoke("list_notes");
}

export function listNoteCategories(): Promise<string[]> {
  return invoke("list_note_categories");
}

/** Promote a clipboard entry to a persistent note. Returns the new note id. */
export function saveClipAsNote(
  clipId: number,
  title: string,
  category: string,
): Promise<number> {
  return invoke("save_clip_as_note", { clipId, title, category });
}

/** Create a from-scratch text note. Returns the new note id. */
export function createNote(
  title: string,
  body: string,
  category: string,
): Promise<number> {
  return invoke("create_note", { title, body, category });
}

/** Update a note's title / body / category. Body edits are ignored for
 *  image and files notes (the backend short-circuits). */
export function updateNote(
  id: number,
  title: string,
  body: string,
  category: string,
): Promise<void> {
  return invoke("update_note", { id, title, body, category });
}

export function deleteNote(id: number): Promise<void> {
  return invoke("delete_note", { id });
}

export function clearNotes(): Promise<void> {
  return invoke("clear_notes");
}

export function pasteNote(id: number): Promise<void> {
  return invoke("paste_note", { id });
}

// ── Backup (full app export / import) ────────────────────────────────────────

export interface BackupExportOptions {
  includeHistory?: boolean;
  includeSnippets?: boolean;
  includeNotes?: boolean;
}

/** Returns a pretty-printed JSON string. Each section is included only
 *  when the corresponding flag is true (or undefined — defaults to true
 *  for backwards compatibility). */
export function exportBackup(opts: BackupExportOptions = {}): Promise<string> {
  return invoke("export_backup", {
    includeHistory: opts.includeHistory ?? true,
    includeSnippets: opts.includeSnippets ?? true,
    includeNotes: opts.includeNotes ?? true,
  });
}

/** Build the backup JSON (with the same selective semantics as
 *  `exportBackup`) and write it directly to `path`. Returns the number
 *  of bytes written. */
export function saveBackupToFile(
  path: string,
  opts: BackupExportOptions = {},
): Promise<number> {
  return invoke("save_backup_to_file", {
    path,
    includeHistory: opts.includeHistory ?? true,
    includeSnippets: opts.includeSnippets ?? true,
    includeNotes: opts.includeNotes ?? true,
  });
}

// ── Text expander ────────────────────────────────────────────────────────────

export interface ExpanderConfig {
  enabled: boolean;
  /** Tauri shortcut string, e.g. "Alt+Backquote", "Ctrl+Shift+E". */
  hotkey: string;
  /** True if the OS has granted ClipSnap permission to synthesize keyboard
   *  events. macOS: Accessibility. Other OSes: always true. */
  accessibility_granted: boolean;
}

export function getExpanderConfig(): Promise<ExpanderConfig> {
  return invoke("get_expander_config");
}

/** Persist a new expander config and re-register the hotkey. The backend
 *  validates the hotkey string and errors out *before* writing settings if
 *  it's malformed, so the previous registration stays intact on failure. */
export function setExpanderConfig(
  enabled: boolean,
  hotkey: string,
): Promise<ExpanderConfig> {
  return invoke("set_expander_config", { enabled, hotkey });
}

/** Programmatically trigger an expand-at-cursor cycle. Used by the
 *  "Test now" button in settings. */
export function triggerExpandAtCursor(): Promise<void> {
  return invoke("trigger_expand_at_cursor");
}

export interface DiagnoseResult {
  captured: string;
  matched_abbreviation: string | null;
  paste_preview: string | null;
  /** Which capture mechanism was actually used. */
  path: "ax" | "uia" | "clipboard";
}

/** Capture the word before the cursor (select prev word + copy) and run
 *  the snippet lookup, but *don't* paste. Hides the popup first so the
 *  synthetic keystrokes target the prior frontmost app. Returns the raw
 *  captured text and the matched snippet abbreviation, if any. */
export function diagnoseExpandAtCursor(): Promise<DiagnoseResult> {
  return invoke("diagnose_expand_at_cursor");
}

// ── Direct hotkey → snippet slots ────────────────────────────────────────────

/** A "press this hotkey → paste this snippet's body" binding. Unlike the
 *  abbreviation expander it reads nothing — it just pastes — so it works in
 *  any app, including terminals. `abbreviation`/`title` are `null` if the
 *  bound snippet was deleted. */
export interface DirectSlot {
  /** Tauri shortcut string, e.g. "Alt+Digit2". */
  hotkey: string;
  snippet_id: number;
  abbreviation: string | null;
  title: string | null;
}

export function getDirectSlots(): Promise<DirectSlot[]> {
  return invoke("get_direct_slots");
}

/** Replace the whole direct-slot list. The backend validates snippet ids,
 *  re-registers the global shortcuts (rejecting collisions with the popup /
 *  OCR / abbreviation hotkeys and duplicates), then persists — nothing is
 *  written if registration fails, so the previous slots stay live on error.
 *  Returns the re-resolved list. */
export function setDirectSlots(
  slots: { hotkey: string; snippet_id: number }[],
): Promise<DirectSlot[]> {
  return invoke("set_direct_slots", { slots });
}

/** Cheap probe — returns true if synthetic-input permission is granted
 *  (macOS Accessibility / other OSes always true). Used for polling
 *  while the user is in System Settings granting access. */
export function getAccessibilityStatus(): Promise<boolean> {
  return invoke("get_accessibility_status");
}

/** Triggers the macOS "would like to control this computer" dialog and
 *  adds ClipSnap to the Accessibility list. Returns the still-likely-
 *  false trusted state immediately after the prompt fires. No-op on
 *  Windows / Linux. */
export function requestAccessibilityGrant(): Promise<boolean> {
  return invoke("request_accessibility_grant");
}

/** Opens System Settings → Privacy & Security → Accessibility on macOS
 *  via `open x-apple.systempreferences:…`. No-op on other OSes. */
export function openAccessibilitySettings(): Promise<void> {
  return invoke("open_accessibility_settings");
}

/** Wipe stale TCC Accessibility/PostEvent entries for ClipSnap (via
 *  `tccutil reset`) then fire the system "would like to control" prompt
 *  with the *current* cdhash. Use this when the System Settings toggle
 *  says "on" but ClipSnap still asks for permission on every action —
 *  that means the toggle is for an older binary's cdhash. */
export function forceResetAndRequestGrant(): Promise<boolean> {
  return invoke("force_reset_and_request_grant");
}

/** Quit the app process. Used after granting Accessibility on macOS so
 *  the next launch picks up the fresh AXIsProcessTrusted state. */
export function quitApp(): Promise<void> {
  return invoke("quit_app");
}

/** Spawn a fresh ClipSnap process and exit the current one. Used by the
 *  Settings panel's auto-restart prompt: the new process picks up the
 *  freshly granted Accessibility state which the running process can't
 *  see (macOS caches the trust check per-process). */
export function relaunchApp(): Promise<void> {
  return invoke("relaunch_app");
}

// ── Autostart (login item / LaunchAgent) ─────────────────────────────────────

/** Whether ClipSnap is set to launch automatically on login.
 *  macOS: checks `~/Library/LaunchAgents/ClipSnap.plist`.
 *  Windows: checks the run-key registry entry. */
export function getAutostartEnabled(): Promise<boolean> {
  return invoke("get_autostart_enabled");
}

/** Toggle autostart. Returns the *now-effective* state read back from the
 *  OS (so the UI can reconcile against actual filesystem / registry state
 *  if the underlying call partially failed). The backend also emits the
 *  `autostart-changed` event with the same boolean — listen for it to
 *  keep tray + Settings in sync when one toggles the other. */
export function setAutostartEnabled(enabled: boolean): Promise<boolean> {
  return invoke("set_autostart_enabled", { enabled });
}

/** Read a backup JSON file from `path` and merge it into the live database. */
export function importBackup(path: string): Promise<BackupImportResult> {
  return invoke("import_backup", { path });
}

/** Show the system-wide screen color picker (eyedropper). Returns
 *  immediately. The result arrives later via the Tauri event
 *  `"color-picked"` with payload `string | null` (`null` = cancelled).
 *
 *  - macOS: invokes Apple's NSColorSampler (the same magnifier-loupe
 *    used by Pages / Keynote / Sketch). 10.15+.
 *  - Windows: pops up a fullscreen overlay; click anywhere to sample. */
export function pickScreenColor(): Promise<void> {
  return invoke("pick_screen_color");
}

/** Tint an image clipboard entry to `hex` (with or without leading `#`).
 *  Creates a NEW history entry containing the recolored PNG; the
 *  original stays put. Returns the new entry id. The popup list
 *  auto-refreshes via the `clipboard-changed` event. */
export function recolorImageEntry(id: number, hex: string): Promise<number> {
  return invoke("recolor_image_entry", { id, hex });
}

/** Returns max chromaticity (0..1) from a sample of opaque pixels in an
 *  image entry. ~0 means grayscale silhouette → tint will look clean.
 *  ~0.5+ means a saturated photo → tint will look weird. */
export function imageChromaticity(id: number): Promise<number> {
  return invoke("image_chromaticity", { id });
}

/** Background-remove an image entry via corner-sampled chroma-key.
 *  Saves the transparent PNG to `~/Downloads/clipsnap-cutout-<ts>.png`
 *  and returns the absolute path. Leaves the history entry untouched. */
export function cutOutImageEntry(id: number): Promise<string> {
  return invoke("cut_out_image_entry", { id });
}

/** Same as `cutOutImageEntry` but reads the image from a file path on
 *  disk (any supported format: PNG, JPEG, WebP, GIF, BMP). Output is
 *  always PNG with alpha. Used when the selected entry is a single-file
 *  Files-typed clipboard entry pointing at an image. */
export function cutOutImageFile(path: string): Promise<string> {
  return invoke("cut_out_image_file", { path });
}

/** Save a clipboard image entry to `~/Downloads/clipsnap-image-<ts>.png`
 *  unchanged. Companion to recolor — recolor produces a new history
 *  entry with the tinted image; this lets the user grab that entry as
 *  a real file on disk. Returns the saved absolute path. */
export function saveImageEntryToDownloads(id: number): Promise<string> {
  return invoke("save_image_entry_to_downloads", { id });
}

/** Result of an OCR run. `cancelled` distinguishes user-pressed-Esc
 *  from "ran but no text detected". `chars` is the unicode character
 *  count of the recognized text, included so toasts don't have to
 *  recalculate. */
export interface OcrResult {
  text: string;
  cancelled: boolean;
  chars: number;
}

/** Trigger the OCR pipeline: hide popup → interactive region pick
 *  (macOS `screencapture -i`) → OCR via Vision → write text to system
 *  clipboard → also push as a History entry. macOS only for now;
 *  Windows returns an error string. Blocks while the user is dragging
 *  the marquee.
 *
 *  Possible error sentinels (raw strings, switch on these):
 *    - "screen.permission_denied" — Screen Recording not granted
 *    - other — wrapped error message from the backend  */
export function ocrRegion(): Promise<OcrResult> {
  return invoke("ocr_region");
}

// ── macOS Screen Recording permission ──────────────────────────────────────

/** Whether ClipSnap currently has Screen Recording (TCC ScreenCapture)
 *  granted. Required for OCR to work — `screencapture -i` is attributed
 *  to ClipSnap, so without this the marquee never appears. Always
 *  `true` on non-macOS. */
export function getScreenRecordingStatus(): Promise<boolean> {
  return invoke("get_screen_recording_status");
}

/** Trigger the macOS Screen Recording prompt. Returns the (almost
 *  always false) status immediately after firing. */
export function requestScreenRecordingGrant(): Promise<boolean> {
  return invoke("request_screen_recording_grant");
}

/** Open System Settings → Privacy & Security → Screen Recording. */
export function openScreenRecordingSettings(): Promise<void> {
  return invoke("open_screen_recording_settings");
}

/** Reset the Screen Recording TCC entry for ClipSnap (no sudo) and
 *  re-fire the prompt. Use when System Settings shows ClipSnap as
 *  enabled but the running process still sees the policy as denied. */
export function forceResetScreenRecordingGrant(): Promise<boolean> {
  return invoke("force_reset_screen_recording_grant");
}
