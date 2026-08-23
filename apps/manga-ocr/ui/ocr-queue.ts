export type OcrPageState = "queued" | "recognizing" | "complete" | "failed";

/** Current page first, then alternate forward/backward across the whole chapter. */
export function prioritizeAllPageIndices(
  length: number,
  currentIndex: number,
): number[] {
  if (length <= 0) return [];
  const center = Math.max(0, Math.min(length - 1, currentIndex));
  const indices: number[] = [];
  for (let distance = 0; indices.length < length; distance += 1) {
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
