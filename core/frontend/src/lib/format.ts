export function relativeTime(unixMs: number): string {
  const diff = Date.now() - unixMs;
  if (diff < 60_000) return "just now";
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
  if (diff < 604_800_000) return `${Math.floor(diff / 86_400_000)}d ago`;
  return new Date(unixMs).toLocaleDateString();
}

/** Full local date+time string for absolute timestamps in tooltips
 *  and click-to-reveal chips. Uses the user's locale via Intl —
 *  on macOS that's whatever System Settings → Language & Region says,
 *  matches Finder / mail / calendar formatting muscle memory. */
export function formatAbsolute(unixMs: number): string {
  return new Date(unixMs).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function truncateOneLine(text: string, max = 120): string {
  const flat = text.replace(/\s+/g, " ").trim();
  if (flat.length <= max) return flat;
  return flat.slice(0, max - 1) + "…";
}
