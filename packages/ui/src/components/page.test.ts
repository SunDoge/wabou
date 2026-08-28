import { describe, expect, test } from "bun:test";
import {
  pageHeaderClass,
  pageViewportClass,
  pageViewportContentClass,
} from "./page";

describe("PageViewport layout contract", () => {
  test("owns the bounded flex viewport defaults", () => {
    expect(pageViewportClass()).toBe("min-w-0 min-h-0 flex-1");
    expect(pageViewportClass("bg-canvas")).toBe(
      "min-w-0 min-h-0 flex-1 bg-canvas",
    );
  });

  test("lets long page content grow beyond its minimum viewport height", () => {
    expect(pageViewportContentClass()).toBe("w-full");
    expect(pageViewportContentClass("max-w-6xl")).toBe("w-full max-w-6xl");
  });

  test("keeps page headings outside the flexible body", () => {
    expect(pageHeaderClass()).toContain("flex-none");
    expect(pageHeaderClass()).toContain("min-h-12");
    expect(pageHeaderClass(undefined, true)).toContain("flex-col");
  });
});
