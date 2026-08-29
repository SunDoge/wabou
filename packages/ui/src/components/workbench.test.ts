import { describe, expect, test } from "bun:test";
import {
  workbenchClass,
  workbenchContentClass,
  workbenchFooterClass,
  workbenchHeaderClass,
  workbenchMainClass,
  workbenchSidebarClass,
} from "./workbench";

describe("workbench geometry contract", () => {
  test("bounds the application and every resizable content column", () => {
    expect(workbenchClass()).toContain("overflow-hidden");
    expect(workbenchMainClass()).toContain("min-w-0");
    expect(workbenchMainClass()).toContain("min-h-0");
    expect(workbenchContentClass()).toContain("min-w-0");
    expect(workbenchContentClass()).toContain("min-h-0");
  });

  test("uses one chrome height for sidebar and main headers", () => {
    expect(workbenchHeaderClass()).toContain("h-12");
    expect(workbenchHeaderClass()).toContain("flex-none");
  });

  test("keeps navigation and footer chrome out of content flexing", () => {
    expect(workbenchSidebarClass()).toContain("flex-none");
    expect(workbenchFooterClass()).toContain("flex-none");
  });

  test("allows explicit application overrides without duplicate utilities", () => {
    expect(workbenchSidebarClass("w-72")).toContain("w-72");
    expect(workbenchSidebarClass("w-72")).not.toContain("w-64");
    expect(workbenchHeaderClass("bg-canvas")).toContain("bg-canvas");
    expect(workbenchHeaderClass("bg-canvas")).not.toContain("bg-surface");
  });
});
