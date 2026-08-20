import { expect, test } from "bun:test";
import { filterCommandItems, reconcileCommandHighlight } from "./command-state";

const items = [
  { id: "settings", label: "Open settings", keywords: ["preferences"] },
  { id: "locked", label: "Locked action", disabled: true },
  { id: "theme", label: "Change theme", keywords: ["appearance", "dark"] },
];

test("command filtering matches labels and keywords", () => {
  expect(filterCommandItems(items, "open set").map((item) => item.id)).toEqual([
    "settings",
  ]);
  expect(filterCommandItems(items, "dark").map((item) => item.id)).toEqual([
    "theme",
  ]);
});

test("command highlight survives filtering and skips disabled items", () => {
  expect(reconcileCommandHighlight(items, "theme")).toBe("theme");
  expect(reconcileCommandHighlight(items.slice(1), "settings")).toBe("theme");
});
