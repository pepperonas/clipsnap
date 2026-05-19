import { IS_MAC } from "../lib/platform";

interface Props {
  index: number;
  total: number;
  /** App version, e.g. "0.2.6". Rendered as `v0.2.6` next to the counter
   *  when provided. Optional so unit tests don't need a Tauri context. */
  version?: string;
}

export function Footer({ index, total, version }: Props) {
  const label = total === 0 ? "0/0" : `${index + 1}/${total}`;
  // OCR + Screenshot are the most-hidden global shortcuts — they fire
  // from anywhere on the system without needing the popup open.
  // Surfaced in the footer so users discover them without having to dig
  // into the tray menu or Settings → Keyboard shortcuts.
  const ocrKey = IS_MAC ? "⌃⇧O" : "Ctrl+⇧+O";
  const screenshotKey = IS_MAC ? "⌃⇧S" : "Ctrl+⇧+S";
  return (
    <div className="flex h-8 items-center justify-between border-t border-[var(--color-border)] px-4 text-[11px] text-[var(--color-muted)]">
      <div className="flex items-center gap-3">
        <Hint k="⏎" label="Paste" />
        <Hint k="↑↓" label="Navigate" />
        <Hint k="Esc" label="Close" />
        <Hint k={ocrKey} label="OCR" />
        <Hint k={screenshotKey} label="Shot" />
      </div>
      <div className="flex items-center gap-3">
        <span title="Made with love by Martin Pfeffer">
          made with <span className="text-red-400">♥</span> by Martin Pfeffer
        </span>
        {version && (
          <span title="ClipSnap version" className="font-[var(--font-mono)]">
            v{version}
          </span>
        )}
        <span>{label}</span>
      </div>
    </div>
  );
}

function Hint({ k, label }: { k: string; label: string }) {
  return (
    <span className="flex items-center gap-1">
      <kbd className="rounded border border-[var(--color-border)] bg-[var(--color-surface)] px-1.5 py-0.5 font-[var(--font-mono)] text-[10px]">
        {k}
      </kbd>
      <span>{label}</span>
    </span>
  );
}
