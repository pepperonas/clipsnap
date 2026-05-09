import { useEffect, useRef, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import {
  AlertTriangle,
  Archive,
  CheckCircle2,
  ChevronDown,
  ChevronUp,
  ClipboardType,
  Download,
  Info,
  Keyboard,
  PlayCircle,
  Upload,
  Wand2,
  Zap,
} from "lucide-react";
import { AboutModal } from "./AboutModal";
import {
  diagnoseExpandAtCursor,
  forceResetAndRequestGrant,
  getAccessibilityStatus,
  getExpanderConfig,
  getPastePlainTextOnly,
  importBackup,
  openAccessibilitySettings,
  quitApp,
  relaunchApp,
  requestAccessibilityGrant,
  saveBackupToFile,
  setExpanderConfig,
  setPastePlainTextOnly,
  setSuppressHide,
  type DiagnoseResult,
  type ExpanderConfig,
} from "../lib/ipc";
import type { BackupImportResult } from "../lib/types";
import { formatBytes } from "../lib/format";
import { HotkeyCapture } from "./HotkeyCapture";

const DEFAULT_HOTKEY = "Alt+Backquote";

interface Props {
  /** Notes-tab refresh — used to reflect imported notes immediately. */
  onBackupImported?: () => Promise<void> | void;
}

export function SettingsPanel({ onBackupImported }: Props = {}) {
  const [cfg, setCfg] = useState<ExpanderConfig | null>(null);
  const [hotkey, setHotkey] = useState<string>(DEFAULT_HOTKEY);
  const [enabled, setEnabled] = useState(false);
  const [accessibility, setAccessibility] = useState<boolean | null>(null);
  // Banner is collapsed by default — only the warning row is visible.
  // The user clicks the chevron / row to expand the full step-by-step
  // walkthrough. The collapsed bar stays prominent (amber border +
  // warning icon) so the problem remains obvious without taking the
  // 8-line block of vertical real estate every settings visit.
  const [accessExpanded, setAccessExpanded] = useState(false);
  // Set to true when polling detects a false→true transition. Drives the
  // "Access detected — restart ClipSnap to activate?" prompt: macOS caches
  // AXIsProcessTrusted per-process, so the running ClipSnap can't actually
  // *use* the just-granted permission until it's relaunched.
  const [justGranted, setJustGranted] = useState(false);
  const [restarting, setRestarting] = useState(false);
  const [busy, setBusy] = useState(false);
  const [diagnose, setDiagnose] = useState<
    | { kind: "running" }
    | { kind: "result"; data: DiagnoseResult }
    | { kind: "err"; message: string }
    | null
  >(null);
  const [status, setStatus] = useState<
    | { kind: "ok"; message: string }
    | { kind: "err"; message: string }
    | null
  >(null);
  const pollRef = useRef<number | null>(null);

  // ── Paste section state ─────────────────────────────────────────────────
  const [plainTextOnly, setPlainTextOnly] = useState<boolean | null>(null);
  const [plainTextSaving, setPlainTextSaving] = useState(false);

  // ── About modal state ───────────────────────────────────────────────────
  // Pulled lazily on first open of the About dialog rather than on
  // SettingsPanel mount — keeps the panel render path independent of
  // the Tauri context (matters for component tests).
  const [aboutOpen, setAboutOpen] = useState(false);
  const [appVersion, setAppVersion] = useState<string | undefined>(undefined);
  useEffect(() => {
    if (!aboutOpen || appVersion) return;
    getVersion().then(setAppVersion).catch(() => undefined);
  }, [aboutOpen, appVersion]);

  useEffect(() => {
    let alive = true;
    getPastePlainTextOnly()
      .then((v) => {
        if (alive) setPlainTextOnly(v);
      })
      .catch(() => {
        if (alive) setPlainTextOnly(true);
      });
    return () => {
      alive = false;
    };
  }, []);

  const togglePlainText = async (next: boolean) => {
    setPlainTextOnly(next); // optimistic
    setPlainTextSaving(true);
    try {
      await setPastePlainTextOnly(next);
    } catch (e) {
      // Revert on failure.
      setPlainTextOnly(!next);
      setStatus({ kind: "err", message: String(e) });
    } finally {
      setPlainTextSaving(false);
    }
  };

  // ── Backup section state ────────────────────────────────────────────────
  const [includeHistory, setIncludeHistory] = useState(true);
  const [includeSnippets, setIncludeSnippets] = useState(true);
  const [includeNotes, setIncludeNotes] = useState(true);
  const [backupBusy, setBackupBusy] = useState<"export" | "import" | null>(null);
  const [backupStatus, setBackupStatus] = useState<
    | { kind: "ok"; message: string }
    | { kind: "import-ok"; result: BackupImportResult }
    | { kind: "err"; message: string }
    | null
  >(null);

  const onExport = async () => {
    if (!includeHistory && !includeSnippets && !includeNotes) {
      setBackupStatus({
        kind: "err",
        message: "Select at least one section to export.",
      });
      return;
    }
    setBackupStatus(null);
    setBackupBusy("export");
    await setSuppressHide(true).catch(() => {});
    try {
      const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
      const path = await saveDialog({
        title: "Save ClipSnap backup",
        defaultPath: `clipsnap-backup-${stamp}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      const bytes = await saveBackupToFile(path, {
        includeHistory,
        includeSnippets,
        includeNotes,
      });
      const filename = path.split("/").pop() ?? path;
      setBackupStatus({
        kind: "ok",
        message: `Exported ${formatBytes(bytes)} to ${filename}`,
      });
    } catch (e) {
      setBackupStatus({ kind: "err", message: String(e) });
    } finally {
      await setSuppressHide(false).catch(() => {});
      setBackupBusy(null);
    }
  };

  const onImport = async () => {
    setBackupStatus(null);
    setBackupBusy("import");
    await setSuppressHide(true).catch(() => {});
    try {
      const path = await openDialog({
        multiple: false,
        directory: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
        title: "Select ClipSnap backup file",
      });
      if (!path) return;
      const result = await importBackup(path);
      setBackupStatus({ kind: "import-ok", result });
      if (onBackupImported) await onBackupImported();
    } catch (e) {
      setBackupStatus({ kind: "err", message: String(e) });
    } finally {
      await setSuppressHide(false).catch(() => {});
      setBackupBusy(null);
    }
  };

  // Initial load.
  useEffect(() => {
    let alive = true;
    getExpanderConfig()
      .then((c) => {
        if (!alive) return;
        setCfg(c);
        setHotkey(c.hotkey);
        setEnabled(c.enabled);
        setAccessibility(c.accessibility_granted);
      })
      .catch((e) => setStatus({ kind: "err", message: String(e) }));
    return () => {
      alive = false;
    };
  }, []);

  // While Accessibility is *not* granted, poll once a second so the badge
  // flips to green within a moment of the user toggling it on in System
  // Settings — instead of needing a panel reload. Crucially: when the
  // status flips false→true, surface a one-time restart prompt because
  // the running process can't actually *use* the new grant until it's
  // re-launched (macOS caches AXIsProcessTrusted per-process).
  useEffect(() => {
    if (accessibility === null || accessibility === true) return;
    const id = window.setInterval(async () => {
      try {
        const ok = await getAccessibilityStatus();
        if (ok) {
          setAccessibility(true);
          setJustGranted(true);
        }
      } catch {
        /* ignore — keep polling */
      }
    }, 1000);
    pollRef.current = id;
    return () => {
      if (pollRef.current !== null) window.clearInterval(pollRef.current);
      pollRef.current = null;
    };
  }, [accessibility]);

  const dirty =
    cfg !== null && (cfg.enabled !== enabled || cfg.hotkey !== hotkey);

  const save = async () => {
    setBusy(true);
    setStatus(null);
    try {
      const applied = await setExpanderConfig(enabled, hotkey || DEFAULT_HOTKEY);
      setCfg(applied);
      setHotkey(applied.hotkey);
      setEnabled(applied.enabled);
      setStatus({
        kind: "ok",
        message: applied.enabled
          ? `Expander armed: ${applied.hotkey}`
          : "Expander disabled.",
      });
    } catch (e) {
      setStatus({ kind: "err", message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  const reset = () => {
    setHotkey(DEFAULT_HOTKEY);
    setStatus(null);
  };

  return (
    <div className="relative flex min-h-0 flex-1 flex-col overflow-auto p-6">
      {/* Accessibility banner — collapsed by default to keep the
          settings page scannable, but the warning row stays sticky
          and amber-bordered so the problem is impossible to miss.
          Click the row / chevron to expand the full walkthrough. */}
      {accessibility === false && (
        <div className="sticky top-[-24px] z-20 mx-auto -mt-2 mb-4 w-full max-w-2xl">
          <div className="rounded border border-amber-500/60 bg-[var(--color-bg)] text-[12px] text-[var(--color-text)] shadow-md ring-1 ring-amber-500/30">
            {/* Always-visible warning row — clickable to toggle.
                Shows the headline + the primary action so the user
                can resolve the problem in one click without ever
                expanding. */}
            <div className="flex items-center gap-2 px-3 py-2">
              <button
                type="button"
                onClick={() => setAccessExpanded((v) => !v)}
                aria-expanded={accessExpanded}
                aria-label={accessExpanded ? "Hide details" : "Show details"}
                className="flex flex-1 items-center gap-2 text-left"
              >
                <AlertTriangle size={14} className="shrink-0 text-amber-500" />
                <span className="flex-1 font-medium">
                  Accessibility access required (macOS)
                </span>
                {accessExpanded ? (
                  <ChevronUp size={14} className="shrink-0 text-[var(--color-muted)]" />
                ) : (
                  <ChevronDown size={14} className="shrink-0 text-[var(--color-muted)]" />
                )}
              </button>
              <button
                onClick={async () => {
                  try {
                    await openAccessibilitySettings();
                  } catch (e) {
                    setStatus({ kind: "err", message: String(e) });
                  }
                }}
                className="rounded bg-[var(--color-accent)] px-2.5 py-1 text-[11px] text-[var(--color-accent-fg)] hover:opacity-90"
              >
                Open System Settings
              </button>
            </div>

            {/* Expanded details — full step-by-step + every recovery
                button. Hidden by default; revealed when the user clicks
                the row above. */}
            {accessExpanded && (
              <div className="border-t border-amber-500/30 px-3 py-2.5">
                <div className="flex items-start gap-2">
                  <div className="w-3.5 shrink-0" aria-hidden />
                  <div className="flex-1">
                    <div className="text-[var(--color-muted)]">
                      ClipSnap can't synthesize Cmd+Shift+← / Cmd+C / Cmd+V
                      without it. This panel auto-detects when you flip the
                      toggle in System Settings.
                    </div>
                    <ol className="mt-2 list-decimal space-y-0.5 pl-4 text-[11px] text-[var(--color-muted)]">
                      <li>Click <b>Open System Settings</b> above.</li>
                      <li>Enable the <b>ClipSnap</b> toggle. (If it isn't in the list yet, click <b>+</b> and pick <code className="rounded bg-[var(--color-surface)] px-1">/Applications/ClipSnap.app</code>.)</li>
                      <li>Switch back to ClipSnap. Within a second, this banner flips to a green <b>Restart now</b> prompt — one click, and you're done.</li>
                    </ol>
                    <details className="mt-2 text-[11px] text-[var(--color-muted)]">
                      <summary className="cursor-pointer">Why does this keep happening on rebuild?</summary>
                      <p className="mt-1">
                        macOS Tahoe binds the Accessibility grant to the app's
                        code-signature hash (<code>cdhash</code>). ClipSnap is
                        ad-hoc-signed (no Apple Developer ID), so any binary
                        change produces a new <code>cdhash</code> and macOS
                        treats it as a new app. The{" "}
                        <code>scripts/install-macos.sh</code> helper detects
                        "binary unchanged" via SHA-256 and skips re-signing in
                        that case, so a no-op rebuild keeps your grant. Real
                        source changes still require a re-grant — the only
                        permanent fix is an Apple Developer ID.
                      </p>
                    </details>
                  </div>
                </div>
                <div className="mt-3 flex flex-wrap gap-2 pl-6">
                  <button
                    onClick={async () => {
                      try {
                        if (!window.confirm("Quit ClipSnap now? Re-launch it via Spotlight / Dock to pick up the new Accessibility grant.")) return;
                        await quitApp();
                      } catch (e) {
                        setStatus({ kind: "err", message: String(e) });
                      }
                    }}
                    className="rounded border border-amber-500/60 bg-amber-500/10 px-2.5 py-1 text-[11px] text-amber-600 hover:bg-amber-500/20 dark:text-amber-400"
                  >
                    Quit ClipSnap
                  </button>
                  <button
                    onClick={async () => {
                      try {
                        if (
                          !window.confirm(
                            "This wipes any stale Accessibility / PostEvent grants for ClipSnap and re-fires the macOS prompt with the current binary's signature. Use this when System Settings says ClipSnap is enabled but ClipSnap still asks for permission on every action. Continue?",
                          )
                        )
                          return;
                        await forceResetAndRequestGrant();
                      } catch (e) {
                        setStatus({ kind: "err", message: String(e) });
                      }
                    }}
                    className="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
                    title="Runs `tccutil reset` for io.celox.clipsnap and re-fires the system permission prompt. Fixes the 'toggle is on but expansion still prompts' state."
                  >
                    Force re-grant (clear stale)
                  </button>
                  <button
                    onClick={async () => {
                      try {
                        await requestAccessibilityGrant();
                      } catch (e) {
                        setStatus({ kind: "err", message: String(e) });
                      }
                    }}
                    className="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
                    title="Triggers macOS' built-in Accessibility prompt without resetting first. Use Force re-grant instead if the prompt fails to appear."
                  >
                    Try system prompt
                  </button>
                  <button
                    onClick={async () => {
                      try {
                        const ok = await getAccessibilityStatus();
                        setAccessibility(ok);
                      } catch (e) {
                        setStatus({ kind: "err", message: String(e) });
                      }
                    }}
                    className="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
                  >
                    Re-check
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      <div className="mx-auto w-full max-w-2xl">
        {/* Text expander section */}
        <Section
          icon={<Wand2 size={16} className="text-[var(--color-accent)]" />}
          title="Text expander"
          subtitle="Type a snippet abbreviation in any text field, then press your hotkey to replace it with the snippet body. Like aText / TextExpander."
        >
          {/* Granted state — shown inline since it's not actionable.
              Special case: if we *just* detected the false→true edge
              while polling, prompt for a restart, because the running
              process can't see the new grant until macOS re-evaluates
              AXIsProcessTrusted on a fresh launch. */}
          {accessibility === true && justGranted && (
            <div className="mb-4 flex flex-col gap-2 rounded border border-emerald-500/40 bg-emerald-500/5 px-3 py-2.5 text-[12px]">
              <div className="flex items-start gap-2">
                <CheckCircle2 size={14} className="mt-0.5 shrink-0 text-emerald-500" />
                <div className="flex-1">
                  <div className="font-medium">Access detected — one more step</div>
                  <div className="mt-0.5 text-[var(--color-muted)]">
                    macOS caches the trust check per-process, so the
                    running ClipSnap can't actually use the just-granted
                    permission until it relaunches. The new instance will
                    take ~1 second to start.
                  </div>
                </div>
              </div>
              <div className="flex flex-wrap gap-2 pl-6">
                <button
                  onClick={async () => {
                    setRestarting(true);
                    try {
                      await relaunchApp();
                      // We won't reach this point — process is exiting.
                    } catch (e) {
                      setRestarting(false);
                      setStatus({ kind: "err", message: String(e) });
                    }
                  }}
                  disabled={restarting}
                  className="rounded bg-emerald-500 px-2.5 py-1 text-[11px] font-medium text-white hover:opacity-90 disabled:opacity-50"
                >
                  {restarting ? "Restarting…" : "Restart now"}
                </button>
                <button
                  onClick={() => setJustGranted(false)}
                  className="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
                  title="Dismiss this prompt — the expander will work next time you launch ClipSnap"
                >
                  Later
                </button>
              </div>
            </div>
          )}
          {accessibility === true && !justGranted && (
            <div className="mb-4 flex items-center gap-2 rounded border border-emerald-500/30 bg-emerald-500/5 px-3 py-2 text-[12px]">
              <CheckCircle2 size={14} className="shrink-0 text-emerald-500" />
              <span className="font-medium">Accessibility access granted</span>
            </div>
          )}

          <Row label="Enable">
            <label className="flex cursor-pointer items-center gap-2 text-[12px]">
              <input
                type="checkbox"
                checked={enabled}
                onChange={(e) => setEnabled(e.target.checked)}
                className="accent-[var(--color-accent)]"
              />
              <span className="text-[var(--color-muted)]">
                {enabled ? "Hotkey is registered globally" : "Hotkey is unregistered"}
              </span>
            </label>
          </Row>

          <Row
            label="Hotkey"
            help="Click the button and press a key combination. Backspace clears, Esc cancels. Names match the W3C KeyboardEvent.code spec (Backquote, KeyE, Digit1, …)."
          >
            <div className="flex items-center gap-2">
              <HotkeyCapture
                value={hotkey}
                onChange={setHotkey}
                disabled={busy}
              />
              <button
                onClick={reset}
                disabled={busy || hotkey === DEFAULT_HOTKEY}
                className="rounded border border-[var(--color-border)] px-2 py-1 text-[11px] text-[var(--color-muted)] hover:border-[var(--color-accent)] hover:text-[var(--color-fg)] disabled:opacity-40"
                title="Reset to Alt+Backquote"
              >
                Reset
              </button>
            </div>
          </Row>

          <div className="mt-4 flex items-center gap-3">
            <button
              onClick={() => void save()}
              disabled={!dirty || busy || !hotkey}
              className="rounded bg-[var(--color-accent)] px-3 py-1 text-[12px] text-[var(--color-accent-fg)] hover:opacity-90 disabled:opacity-50"
            >
              {busy ? "Saving…" : dirty ? "Save & re-register" : "No changes"}
            </button>
            {status && (
              <span
                className={
                  "text-[11px] " +
                  (status.kind === "ok"
                    ? "text-[var(--color-muted)]"
                    : "text-red-400")
                }
              >
                {status.message}
              </span>
            )}
          </div>

          <div className="mt-3 flex flex-col gap-2 rounded border border-dashed border-[var(--color-border)] p-3">
            <div className="flex items-start gap-3">
              <PlayCircle size={14} className="mt-0.5 shrink-0 text-[var(--color-accent)]" />
              <div className="flex-1 text-[11px] text-[var(--color-muted)]">
                <div className="text-[var(--color-fg)]">Diagnose expansion (no paste)</div>
                <p className="mt-0.5">
                  Type a snippet abbreviation in any text field, place the
                  cursor right after it, then click <b>Diagnose</b>. ClipSnap
                  will hide its popup, capture the word before your cursor,
                  look it up — and report what it found, without pasting.
                  This isolates the lookup from the paste step so you can
                  see exactly where expansion is breaking.
                </p>
              </div>
              <button
                onClick={async () => {
                  setDiagnose({ kind: "running" });
                  try {
                    const data = await diagnoseExpandAtCursor();
                    setDiagnose({ kind: "result", data });
                  } catch (e) {
                    setDiagnose({ kind: "err", message: String(e) });
                  }
                }}
                disabled={diagnose?.kind === "running"}
                className="rounded border border-[var(--color-border)] px-3 py-1 text-[11px] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)] disabled:opacity-50"
              >
                {diagnose?.kind === "running" ? "Capturing…" : "Diagnose"}
              </button>
            </div>

            {diagnose?.kind === "result" && (
              <div className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] p-2 text-[11px]">
                <div className="grid grid-cols-[110px_1fr] gap-x-3 gap-y-1">
                  <span className="text-[var(--color-muted)]">Capture path</span>
                  <span>
                    {diagnose.data.path === "ax" ? (
                      <span className="rounded bg-emerald-500/10 px-1.5 py-0.5 font-medium text-emerald-400">
                        macOS AX (clean — no clipboard touch)
                      </span>
                    ) : diagnose.data.path === "uia" ? (
                      <span className="rounded bg-emerald-500/10 px-1.5 py-0.5 font-medium text-emerald-400">
                        Windows UIA (clean — no clipboard touch)
                      </span>
                    ) : (
                      <span className="rounded bg-amber-500/10 px-1.5 py-0.5 font-medium text-amber-400">
                        Clipboard fallback — focused app didn&apos;t expose accessibility info
                      </span>
                    )}
                  </span>
                  <span className="text-[var(--color-muted)]">Captured</span>
                  <code className="rounded bg-[var(--color-bg)] px-1 font-[var(--font-mono)]">
                    {diagnose.data.captured || "(empty)"}
                  </code>
                  <span className="text-[var(--color-muted)]">Snippet match</span>
                  <span>
                    {diagnose.data.matched_abbreviation ? (
                      <code className="rounded bg-[var(--color-bg)] px-1 font-[var(--font-mono)] text-emerald-400">
                        {diagnose.data.matched_abbreviation}
                      </code>
                    ) : (
                      <span className="text-amber-400">
                        no match — add a snippet with this abbreviation
                      </span>
                    )}
                  </span>
                  {diagnose.data.paste_preview && (
                    <>
                      <span className="text-[var(--color-muted)]">Would paste</span>
                      <code className="block truncate rounded bg-[var(--color-bg)] px-1 font-[var(--font-mono)]">
                        {diagnose.data.paste_preview}
                      </code>
                    </>
                  )}
                </div>
                {!diagnose.data.captured && (
                  <p className="mt-2 text-[var(--color-muted)]">
                    Empty capture usually means the popup didn't lose focus
                    fast enough, or there was no text before your cursor.
                    Try again with the cursor placed right after a typed
                    abbreviation.
                  </p>
                )}
                {diagnose.data.captured && !diagnose.data.matched_abbreviation && (
                  <p className="mt-2 text-[var(--color-muted)]">
                    The capture worked, but no snippet has{" "}
                    <code className="rounded bg-[var(--color-bg)] px-1">
                      {diagnose.data.captured}
                    </code>{" "}
                    as its abbreviation. Open the <b>Snippets</b> tab and
                    create one, or pick a different abbreviation.
                  </p>
                )}
              </div>
            )}
            {diagnose?.kind === "err" && (
              <div className="text-[11px] text-red-400">{diagnose.message}</div>
            )}
          </div>

          <details className="mt-5 rounded border border-[var(--color-border)] bg-[var(--color-surface)] p-3 text-[11px] text-[var(--color-muted)]">
            <summary className="cursor-pointer font-medium">How it works</summary>
            <ul className="mt-2 list-disc space-y-1 pl-5">
              <li>
                When you press the hotkey, ClipSnap synthesizes{" "}
                <kbd className="rounded border border-[var(--color-border)] px-1">
                  Cmd/Ctrl+Shift+←
                </kbd>{" "}
                to select the previous word, copies it, and looks it up in
                your snippets.
              </li>
              <li>
                On a hit, the snippet body is written to the clipboard and{" "}
                <kbd className="rounded border border-[var(--color-border)] px-1">
                  Cmd/Ctrl+V
                </kbd>{" "}
                pastes over the still-selected abbreviation.
              </li>
              <li>
                On a miss, the selection stays visible (visual cue) and your
                clipboard is left untouched.
              </li>
              <li>
                Caveats: terminals (iTerm2, kitty, gnome-terminal) sometimes
                interpret the word-select shortcut differently — the
                abbreviation may not be picked up cleanly there. Password
                fields refuse synthetic paste in many apps.
              </li>
            </ul>
          </details>
        </Section>

        {/* Paste behaviour section */}
        <div className="mt-6">
          <Section
            icon={<ClipboardType size={16} className="text-[var(--color-accent)]" />}
            title="Paste"
            subtitle="Control how clipboard entries land in the destination app."
          >
            <Row label="Plain text only">
              <label className="flex cursor-pointer items-center gap-2 text-[12px]">
                <input
                  type="checkbox"
                  checked={plainTextOnly ?? true}
                  disabled={plainTextOnly === null || plainTextSaving}
                  onChange={(e) => void togglePlainText(e.target.checked)}
                  className="accent-[var(--color-accent)]"
                />
                <span className="text-[var(--color-muted)]">
                  {plainTextOnly === null
                    ? "Loading…"
                    : plainTextOnly
                      ? "HTML / RTF entries are stripped to plain text on paste"
                      : "Original formatting is preserved when pasting HTML / RTF"}
                </span>
              </label>
            </Row>

            <div className="mt-1 rounded border border-dashed border-[var(--color-border)] bg-[var(--color-surface)] p-2.5 text-[11px] text-[var(--color-muted)]">
              <span className="text-[var(--color-fg)]">Tip — one-shot override:</span>{" "}
              hold{" "}
              <kbd className="rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1 font-[var(--font-mono)]">
                Shift
              </kbd>{" "}
              while pressing{" "}
              <kbd className="rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1 font-[var(--font-mono)]">
                Enter
              </kbd>{" "}
              in the popup to paste with original formatting{" "}
              <em>this once</em>, regardless of the toggle above.
            </div>
          </Section>
        </div>

        {/* Backup & restore section */}
        <div className="mt-6">
          <Section
            icon={<Archive size={16} className="text-[var(--color-accent)]" />}
            title="Backup & restore"
            subtitle="Export everything (or just parts) to a single JSON file. Import merges back: snippets upsert by abbreviation, history dedupes by SHA-256, notes append."
          >
            <Row label="What to export">
              <div className="flex flex-col gap-1.5 text-[12px]">
                <BackupCheckbox
                  label="Clipboard history"
                  checked={includeHistory}
                  onChange={setIncludeHistory}
                />
                <BackupCheckbox
                  label="Snippets"
                  checked={includeSnippets}
                  onChange={setIncludeSnippets}
                />
                <BackupCheckbox
                  label="Notes"
                  checked={includeNotes}
                  onChange={setIncludeNotes}
                />
              </div>
            </Row>

            <div className="mt-4 flex flex-wrap items-center gap-2">
              <button
                onClick={() => void onExport()}
                disabled={backupBusy !== null}
                className="flex items-center gap-1.5 rounded bg-[var(--color-accent)] px-3 py-1 text-[12px] text-[var(--color-accent-fg)] hover:opacity-90 disabled:opacity-50"
              >
                <Download size={12} />
                {backupBusy === "export" ? "Exporting…" : "Export…"}
              </button>
              <button
                onClick={() => void onImport()}
                disabled={backupBusy !== null}
                className="flex items-center gap-1.5 rounded border border-[var(--color-border)] px-3 py-1 text-[12px] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)] disabled:opacity-50"
              >
                <Upload size={12} />
                {backupBusy === "import" ? "Importing…" : "Import…"}
              </button>
              <span className="ml-auto text-[11px] text-[var(--color-muted)]">
                Import merges into the current database
              </span>
            </div>

            {backupStatus && (
              <div
                className={
                  "mt-3 rounded border px-2.5 py-1.5 text-[11px] " +
                  (backupStatus.kind === "err"
                    ? "border-red-500/40 bg-red-500/5 text-red-400"
                    : "border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-muted)]")
                }
              >
                {backupStatus.kind === "ok" ? (
                  backupStatus.message
                ) : backupStatus.kind === "import-ok" ? (
                  <>
                    Imported{" "}
                    <b>{backupStatus.result.notes_imported}</b> notes,{" "}
                    <b>{backupStatus.result.snippets_imported}</b> snippets,{" "}
                    <b>{backupStatus.result.history_imported}</b> history
                    {backupStatus.result.errors.length > 0 && (
                      <>
                        {" — "}
                        <span className="text-red-400">
                          {backupStatus.result.errors[0]}
                          {backupStatus.result.errors.length > 1 &&
                            ` (+${backupStatus.result.errors.length - 1} more)`}
                        </span>
                      </>
                    )}
                  </>
                ) : (
                  <>Failed: {backupStatus.message}</>
                )}
              </div>
            )}
          </Section>
        </div>

        {/* About section */}
        <div className="mt-6">
          <Section
            icon={<Info size={16} className="text-[var(--color-accent)]" />}
            title="About"
            subtitle="Version, license, project info."
          >
            <button
              onClick={() => setAboutOpen(true)}
              className="flex items-center gap-1.5 rounded bg-[var(--color-accent)] px-3 py-1 text-[12px] text-[var(--color-accent-fg)] hover:opacity-90"
            >
              <Info size={12} />
              Show about dialog
            </button>
          </Section>
        </div>
      </div>

      <AboutModal open={aboutOpen} onClose={() => setAboutOpen(false)} version={appVersion} />
    </div>
  );
}

function BackupCheckbox({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-2">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="accent-[var(--color-accent)]"
      />
      <span>{label}</span>
    </label>
  );
}

function Section({
  icon,
  title,
  subtitle,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  subtitle?: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div className="mb-1 flex items-center gap-2">
        {icon}
        <h2 className="text-[14px] font-semibold">{title}</h2>
      </div>
      {subtitle && (
        <p className="mb-4 text-[12px] text-[var(--color-muted)]">
          {subtitle}
        </p>
      )}
      <div className="rounded-lg border border-[var(--color-border)] p-4">
        {children}
      </div>
    </div>
  );
}

function Row({
  label,
  help,
  children,
}: {
  label: string;
  help?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="mb-4 last:mb-0">
      <div className="mb-1 flex items-center gap-2 text-[12px] font-medium">
        <Zap size={11} className="text-[var(--color-accent)] opacity-70" />
        <span>{label}</span>
      </div>
      <div>{children}</div>
      {help && (
        <p className="mt-1 text-[11px] leading-snug text-[var(--color-muted)]">
          {help}
        </p>
      )}
    </div>
  );
}

// Re-export so `import { Keyboard } from ...` keeps the icon utility nearby
// when we add more settings sections.
export { Keyboard };
