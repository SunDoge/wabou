import type { LocatorBounds, LocatorBoundsField } from "./index";

export function containmentDiagnostic(
  inner: LocatorBounds,
  outer: LocatorBounds,
  tolerance: number,
  description: string,
): string | null {
  const outside =
    inner.x < outer.x - tolerance ||
    inner.y < outer.y - tolerance ||
    inner.x + inner.width > outer.x + outer.width + tolerance ||
    inner.y + inner.height > outer.y + outer.height + tolerance;
  return outside
    ? `expected locator bounds ${JSON.stringify(inner)} to be ${description} ${JSON.stringify(outer)} (tolerance ${tolerance}px)`
    : null;
}

export function overlapDiagnostic(
  first: LocatorBounds,
  second: LocatorBounds,
  tolerance: number,
): string | null {
  const separated =
    first.x + first.width <= second.x + tolerance ||
    second.x + second.width <= first.x + tolerance ||
    first.y + first.height <= second.y + tolerance ||
    second.y + second.height <= first.y + tolerance;
  return separated
    ? null
    : `expected locator bounds ${JSON.stringify(first)} not to overlap ${JSON.stringify(second)} (tolerance ${tolerance}px)`;
}

export function matchingBoundsDiagnostic(
  first: LocatorBounds,
  second: LocatorBounds,
  fields: readonly LocatorBoundsField[],
  tolerance: number,
): string | null {
  for (const field of fields) {
    if (Math.abs(first[field] - second[field]) > tolerance) {
      return `expected locator bounds.${field} ${first[field]} to match ${second[field]} within ${tolerance}px`;
    }
  }
  return null;
}
