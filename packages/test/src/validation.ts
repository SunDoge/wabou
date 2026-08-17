const MAX_SAFE_JAVASCRIPT_INTEGER = 9_007_199_254_740_991;

/** Keep authored pointer input JSON-safe and reproducible across native hosts. */
export function validateInputDeltas(
  kind: "drag" | "wheel",
  deltaX: number,
  deltaY: number,
): void {
  if (!Number.isFinite(deltaX) || !Number.isFinite(deltaY)) {
    throw new RangeError(`${kind} deltas must be finite numbers`);
  }
}

/** A physical key pair without a key identity cannot be routed predictably. */
export function validateKey(key: string): void {
  if (typeof key !== "string" || key.length === 0) {
    throw new RangeError("key must be a non-empty string");
  }
}

export function validateWindowId(windowId: number): void {
  if (
    !Number.isSafeInteger(windowId) ||
    windowId <= 0 ||
    windowId > MAX_SAFE_JAVASCRIPT_INTEGER
  ) {
    throw new RangeError(
      `window id must be an integer between 1 and ${MAX_SAFE_JAVASCRIPT_INTEGER}`,
    );
  }
}

export function validateSurfaceGeneration(value: number): void {
  if (
    !Number.isSafeInteger(value) ||
    value < 0 ||
    value > MAX_SAFE_JAVASCRIPT_INTEGER
  ) {
    throw new RangeError(
      `surface generation must be an integer between 0 and ${MAX_SAFE_JAVASCRIPT_INTEGER}`,
    );
  }
}

export function validateWindowPresence(value: string): void {
  if (
    value !== "visible" &&
    value !== "hidden" &&
    value !== "surface-released" &&
    value !== "closed"
  ) {
    throw new RangeError(`unknown window presence ${JSON.stringify(value)}`);
  }
}

export function validateTolerance(kind: string, tolerance: number): void {
  if (!Number.isFinite(tolerance) || tolerance < 0) {
    throw new RangeError(
      `${kind} tolerance must be a finite non-negative number`,
    );
  }
}

export function validateLocatorCount(value: number): void {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new RangeError("locator count must be a non-negative safe integer");
  }
}
