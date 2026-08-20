import { expect, test } from "bun:test";
import { moveMenuHighlight } from "./menu-state";

const items = [
  { id: "open", label: "Open" },
  { id: "rename", label: "Rename", disabled: true },
  { id: "delete", label: "Delete" },
];

test("menu movement skips disabled items and loops", () => {
  expect(moveMenuHighlight(items, undefined, "first")).toBe("open");
  expect(moveMenuHighlight(items, "open", "next")).toBe("delete");
  expect(moveMenuHighlight(items, "delete", "next")).toBe("open");
  expect(moveMenuHighlight(items, "open", "previous")).toBe("delete");
});
