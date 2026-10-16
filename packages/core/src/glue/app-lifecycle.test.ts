import { expect, test } from "bun:test";
import { subscribeAppLifecycle } from "./app-lifecycle";
import { dispatchHostMessage } from "./host-messages";

test("decodes operating-system lifecycle notifications", () => {
  const states: string[] = [];
  const unsubscribe = subscribeAppLifecycle(({ state }) => states.push(state));
  for (const state of ["resumed", "suspended", "memory-warning"]) {
    dispatchHostMessage("wabou:app-lifecycle", JSON.stringify({ state }));
  }
  unsubscribe();
  expect(states).toEqual(["resumed", "suspended", "memory-warning"]);
});
