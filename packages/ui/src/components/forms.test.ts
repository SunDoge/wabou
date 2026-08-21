import { describe, expect, test } from "bun:test";
import { fieldClass, fieldErrorLabel, uniqueFieldErrors } from "./forms";

describe("Field", () => {
  test("owns explicit native orientation instead of CSS container selectors", () => {
    expect(fieldClass("vertical")).toContain("flex-col");
    expect(fieldClass("horizontal")).toContain("flex-row");
    expect(fieldClass("horizontal")).toContain("min-w-0");
  });

  test("projects invalid state into semantic theme color", () => {
    expect(fieldClass("vertical", true)).toContain("text-danger-primary");
    expect(fieldClass("vertical", false)).not.toContain("text-danger-primary");
  });

  test("deduplicates schema errors while preserving order", () => {
    expect(
      uniqueFieldErrors([
        { message: "Required" },
        undefined,
        { message: "Required" },
        { message: "Invalid format" },
      ]),
    ).toEqual(["Required", "Invalid format"]);
  });

  test("gives structured errors a stable semantic name", () => {
    expect(fieldErrorLabel(undefined, undefined, ["Required", "Invalid"])).toBe(
      "Required Invalid",
    );
    expect(fieldErrorLabel("Explicit", "Ignored", ["Required"])).toBe(
      "Explicit",
    );
  });
});
