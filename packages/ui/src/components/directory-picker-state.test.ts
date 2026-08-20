import { expect, test } from "bun:test";
import { directoryPickerOptions } from "./directory-picker-state";

test("a directory picker starts at its controlled path by default", () => {
  expect(directoryPickerOptions(" /downloads ", { title: "Choose" })).toEqual({
    title: "Choose",
    directory: "/downloads",
  });
  expect(
    directoryPickerOptions("/downloads", { directory: "/explicit" }),
  ).toEqual({ directory: "/explicit" });
  expect(directoryPickerOptions("  ", undefined)).toEqual({
    directory: undefined,
  });
});
