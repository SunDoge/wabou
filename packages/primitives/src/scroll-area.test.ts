import { describe, expect, test } from "bun:test";
import { scrollAreaViewportClass } from "./scroll-area";

describe("ScrollArea sizing", () => {
  test("does not implicitly grow inside an intrinsic-height ancestor", () => {
    expect(scrollAreaViewportClass()).toBe("min-h-0 overflow-y-auto");
    expect(scrollAreaViewportClass()).not.toContain("flex-1");
  });

  test("allows a bounded flex parent to opt into filling available height", () => {
    expect(scrollAreaViewportClass("flex-1")).toBe(
      "min-h-0 overflow-y-auto flex-1",
    );
  });
});
