import { describe, expect, test } from "bun:test";
import { drawerDragOffset, drawerShouldDismiss } from "./drawer";

describe("Drawer", () => {
  test("accepts only outward drag movement", () => {
    expect(drawerDragOffset("bottom", 40)).toBe(40);
    expect(drawerDragOffset("bottom", -40)).toBe(0);
    expect(drawerDragOffset("left", -40)).toBe(-40);
    expect(drawerDragOffset("left", 40)).toBe(-0);
  });

  test("dismisses after crossing a proportional threshold", () => {
    expect(drawerShouldDismiss(79, 400, 0.2)).toBe(false);
    expect(drawerShouldDismiss(80, 400, 0.2)).toBe(true);
    expect(drawerShouldDismiss(79, 0, 0.2)).toBe(false);
    expect(drawerShouldDismiss(80, 0, 0.2)).toBe(true);
  });
});
