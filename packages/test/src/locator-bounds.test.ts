import { describe, expect, test } from "bun:test";
import { containmentDiagnostic } from "./locator-bounds";

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
