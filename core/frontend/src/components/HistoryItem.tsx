import { memo, useState } from "react";
import { Bookmark, BookmarkCheck, Calculator, FileCode2, FileText, Files, Image, Palette, Trash2, Type, Zap } from "lucide-react";
import type { ListEntry } from "../lib/types";
import { relativeTime, truncateOneLine } from "../lib/format";

interface Props {
  entry: ListEntry;
  selected: boolean;
  onClick: () => void;
  onDoubleClick: () => void;
  /** Save the underlying clipboard entry as a note. Only invoked for `kind: "clip"`. */
  onSaveAsNote?: () => void;
  /** Delete the underlying clipboard entry from history. Only invoked for `kind: "clip"`. */
  onDelete?: () => void;
  style?: React.CSSProperties;
}

function TypeIcon({ entry }: { entry: ListEntry }) {
  const cls = "shrink-0";
  const size = 14;
  if (entry.kind === "snippet") return <Zap size={size} className={cls} />;
  if (entry.kind === "calc") return <Calculator size={size} className={cls} />;
  if (entry.kind === "color") return <Palette size={size} className={cls} />;
  switch (entry.data.content_type) {
    case "text":  return <Type size={size} className={cls} />;
    case "image": return <Image size={size} className={cls} />;
    case "files": return <Files size={size} className={cls} />;
    case "html":  return <FileCode2 size={size} className={cls} />;
    case "rtf":   return <FileText size={size} className={cls} />;
  }
}

export const HistoryItem = memo(function HistoryItem({
  entry,
  selected,
  onClick,
  onDoubleClick,
  onSaveAsNote,
  onDelete,
  style,
}: Props) {
  const [bookmarkSaved, setBookmarkSaved] = useState(false);
  const isSnippet = entry.kind === "snippet";
  const isCalc = entry.kind === "calc";
  const isColor = entry.kind === "color";

  const label =
    isSnippet
      ? `${entry.data.abbreviation}  ${entry.data.title || entry.data.body.split("\n")[0]}`
      : isCalc || isColor
        ? ""
        : truncateOneLine(entry.data.content_text || "(empty)", 80);

  const right = isSnippet ? (
    <span
      className={
        "shrink-0 rounded px-1 py-0.5 text-[10px] font-medium uppercase tracking-wide " +
        (selected
          ? "bg-white/20 text-white/80"
          : "bg-[var(--color-accent)]/15 text-[var(--color-accent)]")
      }
    >
      snippet
    </span>
  ) : isCalc ? (
    <span
      className={
        "shrink-0 rounded px-1 py-0.5 text-[10px] font-medium uppercase tracking-wide " +
        (selected
          ? "bg-white/20 text-white/80"
          : "bg-[var(--color-accent)]/15 text-[var(--color-accent)]")
      }
    >
      calc
    </span>
  ) : isColor ? (
    <span
      className={
        "shrink-0 rounded px-1 py-0.5 text-[10px] font-medium uppercase tracking-wide " +
        (selected
          ? "bg-white/20 text-white/80"
          : "bg-[var(--color-accent)]/15 text-[var(--color-accent)]")
      }
    >
      color
    </span>
  ) : (
    <span
      className={
        "shrink-0 text-[11px] " +
        (selected ? "text-white/70" : "text-[var(--color-muted)]")
      }
    >
      {relativeTime(entry.data.last_used_at)}
    </span>
  );

  return (
    <div
      style={style}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      className={
        "group flex cursor-pointer items-center gap-2 px-3 py-2 text-[13px] " +
        (selected
          ? "bg-[var(--color-accent)] text-[var(--color-accent-fg)]"
          : "hover:bg-[var(--color-surface)]")
      }
    >
      <span
        className={
          "shrink-0 " +
          (selected ? "text-white/80" : "text-[var(--color-muted)]")
        }
      >
        <TypeIcon entry={entry} />
      </span>
      <span className="flex-1 truncate">
        {isSnippet && entry.kind === "snippet" ? (
          <>
            <span className="font-[var(--font-mono)] font-semibold">
              {entry.data.abbreviation}
            </span>
            {(entry.data.title || entry.data.body.split("\n")[0]) && (
              <span className={selected ? "text-white/70" : "text-[var(--color-muted)]"}>
                {"  "}
                {truncateOneLine(entry.data.title || entry.data.body.split("\n")[0], 50)}
              </span>
            )}
          </>
        ) : isCalc && entry.kind === "calc" ? (
          <span className="font-[var(--font-mono)]">
            <span className={selected ? "text-white/70" : "text-[var(--color-muted)]"}>
              {truncateOneLine(entry.data.expression, 40)} ={" "}
            </span>
            <span className="font-semibold">{entry.data.display}</span>
          </span>
        ) : isColor && entry.kind === "color" ? (
          <span className="flex items-center gap-2">
            <span
              className="inline-block h-4 w-4 shrink-0 rounded border border-[var(--color-border)]"
              style={{ backgroundColor: entry.data.hex }}
              aria-hidden
            />
            <span className="font-[var(--font-mono)] font-semibold">
              {entry.data.hex}
            </span>
            <span
              className={
                "truncate text-[11px] " +
                (selected ? "text-white/70" : "text-[var(--color-muted)]")
              }
            >
              {entry.data.rgbString}
            </span>
          </span>
        ) : (
          label
        )}
      </span>
      {entry.kind === "clip" && onSaveAsNote && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            onSaveAsNote();
            setBookmarkSaved(true);
            setTimeout(() => setBookmarkSaved(false), 1500);
          }}
          title={bookmarkSaved ? "Saved!" : "Save as note"}
          className={
            "shrink-0 rounded p-0.5 " +
            (bookmarkSaved
              ? "opacity-100 text-[var(--color-accent)]"
              : "opacity-0 group-hover:opacity-100 " +
                (selected
                  ? "text-white/80 hover:bg-white/20"
                  : "text-[var(--color-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-accent)]"))
          }
        >
          {bookmarkSaved ? <BookmarkCheck size={12} /> : <Bookmark size={12} />}
        </button>
      )}
      {entry.kind === "clip" && onDelete && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDelete();
          }}
          title="Delete entry from history"
          className={
            "shrink-0 rounded p-0.5 opacity-0 group-hover:opacity-100 " +
            (selected
              ? "text-white/80 hover:bg-white/20"
              : "text-[var(--color-muted)] hover:bg-[var(--color-border)] hover:text-red-400")
          }
        >
          <Trash2 size={12} />
        </button>
      )}
      {right}
    </div>
  );
});
