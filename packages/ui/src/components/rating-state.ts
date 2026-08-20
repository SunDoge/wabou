export function normalizeRatingMax(max: number | undefined): number {
  if (max === undefined || !Number.isFinite(max)) return 5;
  return Math.max(1, Math.min(20, Math.floor(max)));
}

export function clampRatingValue(
  value: number | undefined,
  max: number,
): number {
  if (value === undefined || !Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(normalizeRatingMax(max), Math.round(value)));
}

export function ratingLabel(value: number): string {
  return `${value} ${value === 1 ? "star" : "stars"}`;
}
