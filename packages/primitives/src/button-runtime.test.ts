import { expect, test } from "bun:test";
import { Button } from "@wabou/primitives";
import { writer } from "@wabou/core/renderer";
import { createComponent, createRoot } from "solid-js";

test("published Button forwards native focus and accessibility state", () => {
  const attributes: Array<[string, string]> = [];
  const focusOrders: number[] = [];
  const setAttribute = writer.setAttribute.bind(writer);
  const setInteractionPolicy = writer.setInteractionPolicy.bind(writer);
  writer.setAttribute = (_id, name, value) => {
    if (name === "aria-current" || name === "aria-disabled") {
      attributes.push([name, value]);
    }
  };
  writer.setInteractionPolicy = (_id, flags, focusOrder) => {
    if ((flags & 0x01) !== 0) focusOrders.push(focusOrder);
  };
  try {
    createRoot((dispose) => {
      createComponent(Button, {
        focusOrder: -1,
        disabled: true,
        "aria-current": "date",
      });
      dispose();
    });
  } finally {
    writer.setAttribute = setAttribute;
    writer.setInteractionPolicy = setInteractionPolicy;
  }
  expect(attributes).toEqual([
    ["aria-disabled", "true"],
    ["aria-current", "date"],
  ]);
  expect(focusOrders).toEqual([-1]);
});
