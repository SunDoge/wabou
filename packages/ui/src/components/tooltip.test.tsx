import { expect, test } from "bun:test";
import {
  INTERACTION_POLICY,
  mount,
  writer,
} from "@wabou/core/renderer";
import { createRoot, flush } from "solid-js";
// Exercise the Solid-transformed public artifact.
import { Tooltip, View } from "../../dist/index.mjs";

test("Tooltip keeps its non-interactive overlay out of native hit testing", () => {
  const policies: number[] = [];
  const host = globalThis as Record<string, unknown>;
  const resizeObserve = host.__wabou_resize_observe;
  const resizeUnobserve = host.__wabou_resize_unobserve;
  const setInteractionPolicy = writer.setInteractionPolicy.bind(writer);
  host.__wabou_resize_observe = () => {};
  host.__wabou_resize_unobserve = () => {};
  const disposeMount = mount(() => null);
  writer.setInteractionPolicy = (_id, flags) => policies.push(flags);

  try {
    createRoot((dispose) => {
      Tooltip({
        defaultOpen: true,
        motion: false,
        trigger: (bindings) => View({ ref: bindings.ref }),
        children: "Non-interactive hint",
      });
      flush();
      dispose();
    });
  } finally {
    writer.setInteractionPolicy = setInteractionPolicy;
    disposeMount();
    if (resizeObserve === undefined) delete host.__wabou_resize_observe;
    else host.__wabou_resize_observe = resizeObserve;
    if (resizeUnobserve === undefined) delete host.__wabou_resize_unobserve;
    else host.__wabou_resize_unobserve = resizeUnobserve;
  }

  expect(
    policies.some(
      (flags) => (flags & INTERACTION_POLICY.BlockSubtree) !== 0,
    ),
  ).toBe(true);
});
