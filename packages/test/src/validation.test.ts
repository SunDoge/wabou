import { describe, expect, test } from "bun:test";
import {
  validateInputDeltas,
  validateKey,
  validateLocatorCount,
  validateSurfaceGeneration,
  validateTolerance,
  validateWindowId,
  validateWindowPresence,
} from "./validation";

describe("recordable native actions", () => {
  test("accepts finite signed pointer deltas", () => {
    expect(() => validateInputDeltas("drag", -12.5, 0)).not.toThrow();
    expect(() => validateInputDeltas("wheel", 0, 24)).not.toThrow();
  });

  test("rejects non-finite values before they enter a trace", () => {
    expect(() => validateInputDeltas("drag", Number.NaN, 0)).toThrow(
      "drag deltas must be finite numbers",
    );
    expect(() =>
      validateInputDeltas("wheel", 0, Number.POSITIVE_INFINITY),
    ).toThrow("wheel deltas must be finite numbers");
  });

  test("requires an explicit physical key identity", () => {
    expect(() => validateKey("Enter")).not.toThrow();
    expect(() => validateKey("")).toThrow("key must be a non-empty string");
  });

  test("keeps window identities and generations exactly representable", () => {
    expect(() => validateWindowId(1)).not.toThrow();
    expect(() => validateWindowId(0)).toThrow("window id must be an integer");
    expect(() => validateWindowId(Number.MAX_SAFE_INTEGER + 1)).toThrow(
      "window id must be an integer",
    );
    expect(() => validateSurfaceGeneration(0)).not.toThrow();
    expect(() => validateSurfaceGeneration(-1)).toThrow(
      "surface generation must be an integer",
    );
    expect(() => validateSurfaceGeneration(0.5)).toThrow(
      "surface generation must be an integer",
    );
    expect(() => validateWindowPresence("visible")).not.toThrow();
    expect(() => validateWindowPresence("minimized")).toThrow(
      'unknown window presence "minimized"',
    );
  });

  test("requires finite non-negative assertion tolerances", () => {
    expect(() => validateTolerance("range", 0)).not.toThrow();
    expect(() => validateTolerance("range", 1e-9)).not.toThrow();
    expect(() => validateTolerance("range", -1)).toThrow(
      "range tolerance must be a finite non-negative number",
    );
    expect(() => validateTolerance("range", Number.NaN)).toThrow(
      "range tolerance must be a finite non-negative number",
    );
  });

  test("requires exactly representable locator counts", () => {
    expect(() => validateLocatorCount(0)).not.toThrow();
    expect(() => validateLocatorCount(3)).not.toThrow();
    expect(() => validateLocatorCount(-1)).toThrow(
      "locator count must be a non-negative safe integer",
    );
    expect(() => validateLocatorCount(1.5)).toThrow(
      "locator count must be a non-negative safe integer",
    );
  });
});
