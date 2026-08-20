import { describe, expect, test } from "bun:test";
import {
  containmentDiagnostic,
  matchingBoundsDiagnostic,
  overlapDiagnostic,
} from "./locator-bounds";

const viewport = { x: 0, y: 0, width: 900, height: 600 };

describe("locator bounds containment", () => {
  test("accepts edges and the configured tolerance", () => {
    expect(
      containmentDiagnostic(
        { x: -0.5, y: 0, width: 900.5, height: 600.5 },
        viewport,
        0.5,
        "inside viewport",
      ),
    ).toBeNull();
  });

  test("rejects horizontal and vertical overflow", () => {
    expect(
      containmentDiagnostic(
        { x: 800, y: 20, width: 101, height: 30 },
        viewport,
        0.5,
        "inside viewport",
      ),
    ).toContain("inside viewport");
    expect(
      containmentDiagnostic(
        { x: 20, y: 590, width: 30, height: 11 },
        viewport,
        0.5,
        "inside viewport",
      ),
    ).toContain("inside viewport");
  });
});

describe("locator bounds overlap", () => {
  test("accepts separated and touching rectangles", () => {
    const first = { x: 0, y: 0, width: 100, height: 40 };
    expect(
      overlapDiagnostic(first, { x: 101, y: 0, width: 20, height: 40 }, 0),
    ).toBeNull();
    expect(
      overlapDiagnostic(first, { x: 100, y: 0, width: 20, height: 40 }, 0),
    ).toBeNull();
  });

  test("reports overlap and honors tolerance", () => {
    const first = { x: 0, y: 0, width: 100, height: 40 };
    const second = { x: 99.5, y: 0, width: 20, height: 40 };
    expect(overlapDiagnostic(first, second, 0)).toContain("not to overlap");
    expect(overlapDiagnostic(first, second, 0.5)).toBeNull();
  });
});

describe("locator bounds matching", () => {
  test("compares only the selected fields", () => {
    const first = { x: 10, y: 20, width: 100, height: 40 };
    const second = { x: 50, y: 20.5, width: 100, height: 80 };
    expect(
      matchingBoundsDiagnostic(first, second, ["y", "width"], 0.5),
    ).toBeNull();
    expect(matchingBoundsDiagnostic(first, second, ["x"], 0.5)).toContain(
      "bounds.x",
    );
  });
});
