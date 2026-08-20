import { expect, test } from "bun:test";
// This test intentionally exercises the publishable artifacts. Explicit paths
// keep the workspace-wide `wabou-source` test condition from changing what is
// under test.
// The built UI artifact leaves @wabou/core external, so under the repository's
// source condition it shares this exact writer instance.
import { writer } from "../../../core/src/renderer.ts";
import { Button } from "../../dist/primitives.mjs";
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
