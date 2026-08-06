import { describe, expect, test } from "bun:test";
import type { ModuleNode } from "vite";
import {
  allHostProperties,
  rejectUnsupportedProperty,
  wabouHotUpdateModules,
  wabouSourceDirectories,
} from "./index";

describe("style support contract", () => {
  test("host properties are explicit and unsupported names are rejected", () => {
    expect(allHostProperties().length).toBeGreaterThan(40);
    expect(rejectUnsupportedProperty("display")).toBeUndefined();
    expect(rejectUnsupportedProperty("cursor")).toMatch(/unsupported/);
    expect(rejectUnsupportedProperty("transition")).toMatch(/unsupported/);
    expect(rejectUnsupportedProperty("filter")).toMatch(/support matrix/);
  });
});

test("source HMR preserves the component update and adds the Style IR update", () => {
  const component = { id: "/Component.tsx" } as ModuleNode;
  const stylesheet = { id: "\0virtual:wabou-stylesheet" } as ModuleNode;

  expect(wabouHotUpdateModules([component], stylesheet)).toEqual([
    component,
    stylesheet,
  ]);
});

test("Vite scans Rust, UI, and shared-package source conventions", () => {
  expect(wabouSourceDirectories("/app")).toEqual([
    "/app/src",
    "/app/ui",
    "/app/packages",
  ]);
});
