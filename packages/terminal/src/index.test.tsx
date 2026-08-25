import { expect, test } from "bun:test";
import { mount, writer } from "@wabou/core/renderer";
import { createComponent, flush } from "solid-js";
// Exercise the Solid-transformed publishable artifact. Bun's direct TSX
// loader does not apply Wabou's universal renderer transform to source JSX.
import { Terminal } from "../dist/index.mjs";

test("Terminal explicitly participates in native focus routing", () => {
  const focusOrders: number[] = [];
  const setInteractionPolicy = writer.setInteractionPolicy.bind(writer);
  writer.setInteractionPolicy = (_id, flags, focusOrder) => {
    if ((flags & 0x01) !== 0) focusOrders.push(focusOrder);
  };
  let disposeMount: (() => void) | undefined;
  try {
    disposeMount = mount(() => [
      createComponent(Terminal, {}),
      createComponent(Terminal, { focusOrder: -1 }),
    ]);
    flush();
  } finally {
    disposeMount?.();
    writer.setInteractionPolicy = setInteractionPolicy;
  }

  expect(focusOrders).toEqual([0, -1]);
});
