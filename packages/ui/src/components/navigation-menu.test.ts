import { describe, expect, test } from "bun:test";
import { navigationMenuTriggerClass } from "./navigation-menu";

describe("NavigationMenu", () => {
  test("projects open state without descendant data selectors", () => {
    expect(navigationMenuTriggerClass(false)).toContain("bg-transparent");
    expect(navigationMenuTriggerClass(true)).toContain("bg-selected");
  });
});
