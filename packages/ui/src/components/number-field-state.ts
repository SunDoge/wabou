import { decimalPlaces, finiteOr } from "./range";

export interface NumberFieldRange {
  min: number;
  max: number;
  step: number;
  largeStep: number;
}

export function normalizeNumberFieldRange(
  minValue: number | undefined,
  maxValue: number | undefined,
  stepValue: number | undefined,
  largeStepValue: number | undefined,
): NumberFieldRange {
  const min = finiteOr(minValue, Number.NEGATIVE_INFINITY);
  const max = Math.max(min, finiteOr(maxValue, Number.POSITIVE_INFINITY));
  const candidateStep = finiteOr(stepValue, 1);
  const step = candidateStep > 0 ? candidateStep : 1;
  const candidateLargeStep = finiteOr(largeStepValue, step * 10);
  const largeStep = candidateLargeStep > 0 ? candidateLargeStep : step * 10;
  return { min, max, step, largeStep };
}

export function clampNumberFieldValue(
  value: number,
  range: Pick<NumberFieldRange, "min" | "max">,
): number {
  return Math.max(range.min, Math.min(range.max, value));
}

export function addNumberFieldStep(
  value: number,
  amount: number,
  range: Pick<NumberFieldRange, "min" | "max">,
): number {
  const precision = Math.max(decimalPlaces(value), decimalPlaces(amount));
  const next = Number((value + amount).toFixed(precision));
  return clampNumberFieldValue(next, range);
}

export function numberFieldValueFromEmpty(
  direction: -1 | 1,
  range: Pick<NumberFieldRange, "min" | "max">,
): number {
  if (direction > 0 && Number.isFinite(range.min)) return range.min;
  if (direction < 0 && Number.isFinite(range.max)) return range.max;
  return clampNumberFieldValue(0, range);
}
