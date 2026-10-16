import type { LauncherEntry } from "./generated/rust-bindings";

function score(entry: LauncherEntry, query: string): number | undefined {
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  if (terms.length === 0) return 100;
  const label = entry.label.toLocaleLowerCase();
  const words = label.split(/[^\p{L}\p{N}]+/u).filter(Boolean);
  const metadata = [entry.description, ...entry.keywords]
    .join(" ")
    .toLocaleLowerCase();
  let total = 0;
  for (const term of terms) {
    if (label === term) total += 0;
    else if (label.startsWith(term)) total += 2;
    else if (words.some((word) => word.startsWith(term))) total += 5;
    else if (label.includes(term)) total += 9;
    else if (metadata.includes(term)) total += 14;
    else return undefined;
  }
  return total + label.length / 100;
}

export function rankLauncherEntries(
  entries: readonly LauncherEntry[],
  query: string,
  limit = 7,
): LauncherEntry[] {
  return entries
    .map((entry) => ({ entry, score: score(entry, query) }))
    .filter(
      (candidate): candidate is { entry: LauncherEntry; score: number } =>
        candidate.score !== undefined,
    )
    .sort(
      (left, right) =>
        left.score - right.score ||
        left.entry.label.localeCompare(right.entry.label),
    )
    .slice(0, limit)
    .map(({ entry }) => entry);
}
