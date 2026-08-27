import { expect, test } from "bun:test";
import { flush } from "solid-js";
import { dispatchHostMessage } from "./host-messages";
import {
  subscribeKeyboardModifiers,
  useKeyboardModifiers,
} from "./keyboard-modifiers";

test("native modifier changes update reactive state and subscribers", () => {
  const observed: number[] = [];
  const unsubscribe = subscribeKeyboardModifiers((value) =>
    observed.push(value.bits),
  );
  dispatchHostMessage("wabou:keyboard-modifiers", 1 | 2 | 16);
  flush();
  unsubscribe();

  expect(useKeyboardModifiers()()).toEqual({
    bits: 3,
    shift: true,
    control: true,
    alt: false,
    meta: false,
    primary: true,
  });
  expect(observed).toEqual([3]);
});
