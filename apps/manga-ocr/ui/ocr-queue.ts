export type OcrPageState = "queued" | "recognizing" | "complete" | "failed";

/** Current page first, then alternate forward/backward within a bounded radius. */
export function prioritizePageIndices(
  length: number,
  currentIndex: number,
  radius = 2,
): number[] {
  if (length <= 0) return [];
  const center = Math.max(0, Math.min(length - 1, currentIndex));
  const indices: number[] = [];
  for (let distance = 0; distance <= radius; distance += 1) {
    const next = center + distance;
    if (next < length) indices.push(next);
    if (distance === 0) continue;
    const previous = center - distance;
    if (previous >= 0) indices.push(previous);
  }
  return indices;
}

export function ocrStateLabel(state: OcrPageState | undefined): string {
  switch (state) {
    case "queued":
      return "Queued";
    case "recognizing":
      return "Recognizing";
    case "complete":
      return "Ready";
    case "failed":
      return "Failed";
    default:
      return "Not scanned";
  }
}
