import { expect, test } from "bun:test";
import { selectControlsId } from "./select-semantics";

test("select only exposes aria-controls while its listbox exists", () => {
  expect(selectControlsId("example-listbox", false)).toBeUndefined();
  expect(selectControlsId("example-listbox", true)).toBe("example-listbox");
});
