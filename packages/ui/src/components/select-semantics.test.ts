import { expect, test } from "bun:test";
import type { ButtonState } from "../primitives";
import { pickerTriggerClass, selectControlsId } from "./select-semantics";

const idle: ButtonState = {
  hovered: false,
  pressed: false,
  focused: false,
  focusVisible: false,
  selected: false,
  disabled: false,
};

test("select only exposes aria-controls while its listbox exists", () => {
  expect(selectControlsId("example-listbox", false)).toBeUndefined();
  expect(selectControlsId("example-listbox", true)).toBe("example-listbox");
});

test("ghost picker triggers expose quiet idle and explicit interaction states", () => {
  expect(pickerTriggerClass("ghost", idle)).toContain("bg-transparent");
  expect(pickerTriggerClass("ghost", { ...idle, hovered: true })).toContain(
    "bg-control-hover",
  );
  expect(pickerTriggerClass("ghost", { ...idle, pressed: true })).toContain(
    "bg-control-pressed",
  );
  expect(pickerTriggerClass("ghost", { ...idle, focused: true })).toContain(
    "border-focus",
  );
});

test("default picker triggers retain input surface and focus treatment", () => {
  expect(pickerTriggerClass("default", idle)).toContain("bg-input");
  expect(pickerTriggerClass("default", idle)).toContain("border-subtle");
  expect(pickerTriggerClass("default", { ...idle, focused: true })).toContain(
    "border-focus",
  );
});
