import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { createControllableState } from "./state";

test("controllable state updates local state and reports changes", () =>
  createRoot((dispose) => {
    const changes: boolean[] = [];
    const state = createControllableState({
      value: () => undefined,
      defaultValue: false,
      onChange: (value) => changes.push(value),
    });
    expect(state.value()).toBe(false);
    expect(state.set(true)).toBe(true);
    flush();
    expect(state.value()).toBe(true);
    expect(state.set(true)).toBe(false);
    expect(changes).toEqual([true]);
    dispose();
  }));

test("controlled state requests changes without mutating its source", () =>
  createRoot((dispose) => {
    const [external, setExternal] = createSignal("account");
    const changes: string[] = [];
    const state = createControllableState({
      value: external,
      defaultValue: "fallback",
      onChange: (value) => changes.push(value),
    });
    expect(state.set("security")).toBe(true);
    expect(state.value()).toBe("account");
    expect(changes).toEqual(["security"]);
    setExternal("security");
    flush();
    expect(state.value()).toBe("security");
    dispose();
  }));

test("disabled state rejects transitions", () =>
  createRoot((dispose) => {
    const state = createControllableState({
      value: () => undefined,
      defaultValue: false,
      disabled: () => true,
    });
    expect(state.set(true)).toBe(false);
    expect(state.value()).toBe(false);
    dispose();
  }));

test("uncontrolled state can store a function value", () =>
  createRoot((dispose) => {
    const first = () => "first";
    const second = () => "second";
    const state = createControllableState({
      value: () => undefined,
      defaultValue: first,
    });
    expect(state.value()).toBe(first);
    expect(state.set(second)).toBe(true);
    flush();
    expect(state.value()).toBe(second);
    dispose();
  }));
