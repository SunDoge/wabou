import { describe, expect, test } from "bun:test";
import { createRoot } from "solid-js";
import { match } from "ts-pattern";
import {
  createCollection,
  createMachine,
  createRovingFocus,
  createTypeahead,
  toggleSelection,
  unchanged,
  updateDisclosure,
  type UpdateResult,
} from ".";

describe("machine", () => {
  type State = { value: "idle" | "open" };
  type Event = { type: "OPEN" } | { type: "CLOSE" };
  type Command = { type: "FOCUS" };
  const update = (state: State, event: Event): UpdateResult<State, Command> =>
    match(event)
      .with({ type: "OPEN" }, () => ({
        state: { value: "open" as const },
        commands: [{ type: "FOCUS" as const }],
      }))
      .with({ type: "CLOSE" }, () =>
        state.value === "idle"
          ? unchanged<State, Command>(state)
          : { state: { value: "idle" as const }, commands: [] },
      )
      .exhaustive();

  test("combines a pure update with Solid state and explicit commands", () => {
    createRoot((dispose) => {
      const commands: Command[] = [];
      const machine = createMachine({
        initialState: { value: "idle" },
        update,
        execute: (command) => commands.push(command),
      });
      expect(machine.send({ type: "OPEN" })).toBe(true);
      expect(machine.state()).toEqual({ value: "open" });
      expect(commands).toEqual([{ type: "FOCUS" }]);
      dispose();
    });
  });
});

describe("collection behaviors", () => {
  const items = [
    { id: "apple", textValue: "Apple" },
    { id: "banana", textValue: "Banana", disabled: true },
    { id: "blueberry", textValue: "Blueberry" },
  ];

  test("navigates enabled items and loops deliberately", () => {
    const collection = createCollection(() => items);
    expect(collection.next("apple")?.id).toBe("blueberry");
    expect(collection.next("blueberry")).toBeUndefined();
    expect(collection.next("blueberry", true)?.id).toBe("apple");
  });

  test("supports repeated-key typeahead", () => {
    const typeahead = createTypeahead({ timeout: 10_000 });
    expect(typeahead.search(items, "b")?.id).toBe("blueberry");
    expect(typeahead.search(items, "b", "blueberry")?.id).toBe("blueberry");
    typeahead.reset();
  });
});

describe("selection and disclosure", () => {
  test("supports single and multiple selection", () => {
    expect(toggleSelection("one", "one", "single", true)).toBeUndefined();
    expect(toggleSelection(["one"], "two", "multiple")).toEqual(["one", "two"]);
  });

  test("keeps disabled disclosure transitions explicit", () => {
    const state = { open: false, disabled: true };
    expect(updateDisclosure(state, { type: "OPEN" }).state).toBe(state);
    expect(
      updateDisclosure(state, { type: "DISABLED", disabled: false }).state,
    ).toEqual({ open: false, disabled: false });
  });
});

test("roving focus follows orientation and skips disabled targets", () => {
  const focused: string[] = [];
  const selected: string[] = [];
  const roving = createRovingFocus({
    orientation: () => "vertical",
    onMove: (id) => selected.push(id),
  });
  roving.register({ id: "one", target: { focus: () => focused.push("one") } });
  roving.register({
    id: "two",
    disabled: () => true,
    target: { focus: () => focused.push("two") },
  });
  roving.register({
    id: "three",
    target: { focus: () => focused.push("three") },
  });
  expect(roving.move("one", "ArrowDown")).toBe(true);
  expect(selected).toEqual(["three"]);
  expect(focused).toEqual(["three"]);
});
