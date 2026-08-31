import { describe, expect, test } from "bun:test";
import {
  workbenchClass,
  workbenchContentClass,
  workbenchContentColumnClass,
  workbenchFooterClass,
  workbenchHeaderClass,
  workbenchInspectorClass,
  workbenchInspectorContentClass,
  workbenchInspectorHeaderClass,
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
    expect(workbenchHeaderClass()).toContain("w-full");
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

  test("provides one readable desktop column that still shrinks", () => {
    expect(workbenchContentColumnClass()).toContain("w-full");
    expect(workbenchContentColumnClass()).toContain("max-w-4xl");
    expect(workbenchContentColumnClass()).toContain("min-w-0");
    expect(workbenchContentColumnClass("max-w-5xl")).toContain("max-w-5xl");
    expect(workbenchContentColumnClass("max-w-5xl")).not.toContain("max-w-4xl");
  });

  test("bounds inspectors and keeps their chrome out of body flexing", () => {
    expect(workbenchInspectorClass()).toContain("min-w-0");
    expect(workbenchInspectorClass()).toContain("min-h-0");
    expect(workbenchInspectorClass()).toContain("overflow-hidden");
    expect(workbenchInspectorHeaderClass()).toContain("h-14");
    expect(workbenchInspectorHeaderClass()).toContain("flex-none");
    expect(workbenchInspectorContentClass()).toContain("flex-1");
    expect(workbenchInspectorContentClass()).toContain("min-h-0");
  });
});
