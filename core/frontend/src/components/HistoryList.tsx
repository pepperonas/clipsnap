import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Palette, Trash2 } from "lucide-react";
import { ColorPickerModal } from "./ColorPickerModal";
import { HistoryItem } from "./HistoryItem";
import type { ListEntry } from "../lib/types";

interface Props {
  entries: ListEntry[];
  selectedIndex: number;
  onSelect: (i: number) => void;
  onActivate: (i: number) => void;
  /** Save-as-note handler: invoked on the bookmark icon for a clipboard entry. */
  onSaveAsNote?: (i: number) => void;
  /** Delete handler: invoked on the trash icon for a clipboard entry. */
  onDeleteClip?: (i: number) => void;
  /** Clear-all-history handler: invoked from the toolbar button at the top. */
  onClearAll?: () => void;
}

const ROW_HEIGHT = 36;

export function HistoryList({
  entries,
  selectedIndex,
  onSelect,
  onActivate,
  onSaveAsNote,
  onDeleteClip,
  onClearAll,
}: Props) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);

  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
  });

  useEffect(() => {
    if (selectedIndex >= 0 && selectedIndex < entries.length) {
      virtualizer.scrollToIndex(selectedIndex, { align: "auto" });
    }
  }, [selectedIndex, virtualizer, entries.length]);

  // When the popup is dismissed (focus loss → hide_popup, or any other
  // path that closes the popup window), tear the modal down so re-opening
  // the popup shows the default History view, not whatever modal happened
  // to be up at hide-time. Note: the screen eyedropper hides the popup
  // via `w.hide()` *directly* — not through `hide_popup` — so it does
  // NOT emit `popup-hidden`, and the modal correctly stays open across
  // a sample.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen("popup-hidden", () => setPickerOpen(false)).then((u) => {
      unlisten = u;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // Number of clipboard entries (the toolbar's "Clear all" only refers to
  // those — snippets and calc rows are virtual and aren't deleted by it).
  const clipCount = useMemo(
    () => entries.reduce((n, e) => (e.kind === "clip" ? n + 1 : n), 0),
    [entries],
  );

  return (
    <div className="flex h-full flex-col">
      {/* Toolbar always rendered — the color picker is always available;
          the Clear All / count controls are conditional within. */}
      <div className="flex shrink-0 items-center justify-between border-b border-[var(--color-border)] px-3 py-1 text-[11px] text-[var(--color-muted)]">
          <span>
            {clipCount} clip{clipCount === 1 ? "" : "s"}
          </span>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setPickerOpen(true)}
              className="flex items-center gap-1 rounded px-2 py-0.5 hover:bg-[var(--color-surface)] hover:text-[var(--color-accent)]"
              title="Open the color picker; the chosen value can be copied in HEX, RGB, or HSL"
            >
              <Palette size={11} />
              Color picker
            </button>
            {onClearAll && clipCount > 0 && (
              confirmClear ? (
                <div className="flex items-center gap-1">
                  <span className="text-[11px] text-red-400">Delete {clipCount} clip{clipCount === 1 ? "" : "s"}?</span>
                  <button
                    onClick={() => { onClearAll(); setConfirmClear(false); }}
                    className="rounded px-2 py-0.5 text-[11px] text-red-400 hover:bg-red-400/10"
                  >
                    Yes
                  </button>
                  <button
                    onClick={() => setConfirmClear(false)}
                    className="rounded px-2 py-0.5 text-[11px] text-[var(--color-muted)] hover:bg-[var(--color-surface)]"
                  >
                    Cancel
                  </button>
                </div>
              ) : (
                <button
                  onClick={() => setConfirmClear(true)}
                  className="flex items-center gap-1 rounded px-2 py-0.5 hover:bg-[var(--color-surface)] hover:text-red-400"
                  title="Delete all clipboard history"
                >
                  <Trash2 size={11} />
                  Clear all
                </button>
              )
            )}
          </div>
        </div>

      {entries.length === 0 ? (
        <div className="flex flex-1 items-center justify-center text-[13px] text-[var(--color-muted)]">
          No matches
        </div>
      ) : (
        <div ref={parentRef} className="flex-1 overflow-auto">
          <div
            style={{
              height: virtualizer.getTotalSize(),
              width: "100%",
              position: "relative",
            }}
          >
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const entry = entries[virtualRow.index];
              const key =
                entry.kind === "snippet"
                  ? `s-${entry.data.id}`
                  : entry.kind === "calc"
                    ? `calc-${entry.data.expression}`
                    : entry.kind === "color"
                      ? `color-${entry.data.hex}`
                      : `c-${entry.data.id}`;
              return (
                <HistoryItem
                  key={key}
                  entry={entry}
                  selected={virtualRow.index === selectedIndex}
                  onClick={() => onSelect(virtualRow.index)}
                  onDoubleClick={() => onActivate(virtualRow.index)}
                  onSaveAsNote={
                    onSaveAsNote ? () => onSaveAsNote(virtualRow.index) : undefined
                  }
                  onDelete={
                    onDeleteClip ? () => onDeleteClip(virtualRow.index) : undefined
                  }
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    height: virtualRow.size,
                    transform: `translateY(${virtualRow.start}px)`,
                  }}
                />
              );
            })}
          </div>
        </div>
      )}

      <ColorPickerModal open={pickerOpen} onClose={() => setPickerOpen(false)} />
    </div>
  );
}
