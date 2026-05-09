import { useEffect } from "react";
import { ExternalLink, Heart, Info, X } from "lucide-react";

interface Props {
  open: boolean;
  onClose: () => void;
  /** App version, e.g. "0.7.0". Pulled from `getVersion()` in the parent. */
  version?: string;
}

/**
 * About-this-app modal. Static content — version number, author, license,
 * tech stack, target-audience pitch. Designed to slot into the popup
 * height (~500 px) without scrolling on macOS / Windows.
 *
 * Why a modal and not a dedicated tab: this is reference content the
 * user reads once or twice, not a live workspace. A tab would steal
 * permanent navigation real estate; a modal stays out of the way.
 */
export function AboutModal({ open, onClose, version }: Props) {
  // Close on Escape — same pattern as ColorPickerModal.tsx so muscle
  // memory carries across modals.
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        onClose();
      }
    };
    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    >
      {/* Three-row flex grid (header / scrollable body / footer) so
          a popup window shorter than the modal's natural height
          (~700 px on small displays / the 500-px-tall ClipSnap
          popup) renders sticky chrome with the body scrolling
          inside. `max-h-[calc(100vh-2rem)]` keeps the rounded
          corners visible by leaving 1 rem breathing room top + bottom. */}
      <div className="flex max-h-[calc(100vh-2rem)] w-[420px] max-w-[92vw] flex-col overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] shadow-2xl">
        {/* Header — sticky at top of the modal, never scrolls away. */}
        <div className="flex shrink-0 items-center justify-between border-b border-[var(--color-border)] px-4 py-3">
          <h2 className="flex items-center gap-2 text-[14px] font-semibold">
            <Info size={14} className="text-[var(--color-accent)]" />
            About ClipSnap
          </h2>
          <button
            onClick={onClose}
            className="rounded p-1 text-[var(--color-muted)] hover:bg-[var(--color-surface)] hover:text-[var(--color-fg)]"
            title="Close (Esc)"
          >
            <X size={14} />
          </button>
        </div>

        {/* Scrollable body. min-h-0 + flex-1 + overflow-y-auto is the
            React/Tailwind incantation for "fill remaining space and
            scroll inside instead of pushing the parent". */}
        <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
          {/* Identity block */}
          <div className="mb-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3 text-[12px]">
            <div className="flex items-baseline justify-between">
              <span className="text-[14px] font-semibold">ClipSnap</span>
              {version && (
                <span className="font-[var(--font-mono)] text-[12px] text-[var(--color-muted)]">
                  v{version}
                </span>
              )}
            </div>
            <div className="mt-0.5 text-[11px] leading-snug text-[var(--color-muted)]">
              Clipboard productivity toolkit for power users — searchable
              history, snippets, calculator, color picker, image tools.
            </div>
          </div>

          {/* Meta table */}
          <table className="mb-3 w-full text-[12px]">
            <tbody>
              <Meta label="Developer" value="Martin Pfeffer" />
              <Meta label="License" value="MIT" />
              <Meta label="Year" value="2026" />
              <Meta label="Audience" value="Keyboard-driven power users" />
            </tbody>
          </table>

          {/* Workflow pitch */}
          <div className="mb-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-3 text-[11px] leading-relaxed">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-[var(--color-accent)]">
              Workflow optimization
            </div>
            One hotkey, no mouse. Search clipboard history fuzzy, expand
            snippets system-wide, calculate inline, sample colors, recolor
            and freistellen images — without leaving the keyboard. Local
            SQLite, AES-256 encrypted at rest, no telemetry, no cloud.
          </div>

          {/* Tech-stack mini table */}
          <div>
            <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-wide text-[var(--color-muted)]">
              Tech stack
            </div>
            <table className="w-full text-[11px]">
              <tbody>
                <Tech label="Shell" value="Tauri 2 · Wry / WebView" />
                <Tech label="Backend" value="Rust (stable) · rusqlite · clipboard-rs · enigo" />
                <Tech label="Storage" value="SQLite · AES-256-GCM · OS keychain" />
                <Tech label="Frontend" value="React 19 · TypeScript 5 · Vite 7 · Tailwind v4" />
                <Tech label="Image" value="image 0.25 (PNG only)" />
              </tbody>
            </table>
          </div>
        </div>

        {/* Footer — sticky at the bottom, like the header. */}
        <div className="flex shrink-0 items-center justify-between border-t border-[var(--color-border)] px-4 py-2.5 text-[11px] text-[var(--color-muted)]">
          <a
            href="https://github.com/pepperonas/clipsnap"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1 hover:text-[var(--color-accent)]"
          >
            <ExternalLink size={11} />
            github.com/pepperonas/clipsnap
          </a>
          <span className="flex items-center gap-1">
            made with <Heart size={10} className="text-red-400" /> by Martin Pfeffer
          </span>
        </div>
      </div>
    </div>
  );
}

function Meta({ label, value }: { label: string; value: string }) {
  return (
    <tr>
      <td className="w-[110px] py-0.5 pr-2 text-[var(--color-muted)]">{label}</td>
      <td className="py-0.5">{value}</td>
    </tr>
  );
}

function Tech({ label, value }: { label: string; value: string }) {
  return (
    <tr>
      <td className="w-[80px] py-0.5 pr-2 align-top text-[var(--color-muted)]">
        {label}
      </td>
      <td className="py-0.5 font-[var(--font-mono)]">{value}</td>
    </tr>
  );
}
