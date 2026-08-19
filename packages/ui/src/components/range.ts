export function finiteOr(value: number | undefined, fallback: number): number {
  return value !== undefined && Number.isFinite(value) ? value : fallback;
}

export function normalizePercentage(value: number | undefined): number {
  return Math.max(0, Math.min(100, finiteOr(value, 0)));
}

export interface NormalizedRange {
  min: number;
  max: number;
  step: number;
}

export function normalizeRange(
  minValue: number | undefined,
  maxValue: number | undefined,
  stepValue: number | undefined,
): NormalizedRange {
  const min = finiteOr(minValue, 0);
  const max = Math.max(min, finiteOr(maxValue, 100));
  const candidateStep = finiteOr(stepValue, 1);
  const step = candidateStep > 0 ? candidateStep : 1;
  return { min, max, step };
}

export function decimalPlaces(value: number): number {
  const [coefficient, exponentText] = String(value).toLowerCase().split("e");
  const fractionLength = coefficient.split(".")[1]?.length ?? 0;
  const exponent = exponentText === undefined ? 0 : Number(exponentText);
  return Math.max(0, Math.min(100, fractionLength - exponent));
}
