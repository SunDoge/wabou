import type { LocatorBounds } from "./index";

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
