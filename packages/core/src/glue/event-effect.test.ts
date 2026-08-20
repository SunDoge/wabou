import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { createEventEffect } from "./event-effect";

interface Event {
  sequence: number;
  value: string;
}

test("consumes every batched event once in sequence order", () => {
  const received: string[] = [];
  let setEvents!: (
    events:
      | readonly Event[]
      | ((current: readonly Event[]) => readonly Event[]),
  ) => void;
  let dispose!: () => void;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    const [events, writeEvents] = createSignal<readonly Event[]>([
      { sequence: 2, value: "history" },
    ]);
    setEvents = writeEvents;
    createEventEffect({
      source: events,
      sequence: (event) => event.sequence,
      onEvent: (event) => received.push(event.value),
    });
    flush();
  });

  setEvents([
    { sequence: 5, value: "third" },
    { sequence: 4, value: "second" },
    { sequence: 4, value: "duplicate second" },
    { sequence: 3, value: "first" },
    { sequence: 2, value: "history" },
  ]);
  flush();
  expect(received).toEqual(["first", "second", "third"]);
  dispose();

  setEvents((current) => [...current]);
  flush();
  expect(received).toEqual(["first", "second", "third"]);
});

test("can consume retained history from oldest to newest", () => {
  const received: string[] = [];
  let dispose!: () => void;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    const [events] = createSignal<readonly Event[]>([
      { sequence: 3, value: "third" },
      { sequence: 1, value: "first" },
      { sequence: 2, value: "second" },
    ]);
    createEventEffect({
      source: events,
      sequence: (event) => event.sequence,
      consumeInitial: true,
      onEvent: (event) => received.push(event.value),
    });
    flush();
  });
  expect(received).toEqual(["first", "second", "third"]);
  dispose();
});

test("advances the cursor before application callbacks mutate the feed", () => {
  const received: string[] = [];
  let setEvents!: (
    events:
      | readonly Event[]
      | ((current: readonly Event[]) => readonly Event[]),
  ) => void;
  let dispose!: () => void;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    const [events, writeEvents] = createSignal<readonly Event[]>([]);
    setEvents = writeEvents;
    createEventEffect({
      source: events,
      sequence: (event) => event.sequence,
      onEvent: (event) => {
        received.push(event.value);
        if (event.sequence === 1)
          setEvents((current) => [
            { sequence: 2, value: "second" },
            ...current,
          ]);
      },
    });
    flush();
  });
  setEvents([{ sequence: 1, value: "first" }]);
  flush();
  expect(received).toEqual(["first", "second"]);
  dispose();
});

test("continues after feed truncation and skips retained events after remount", () => {
  const received: number[] = [];
  let setEvents!: (
    events:
      | readonly Event[]
      | ((current: readonly Event[]) => readonly Event[]),
  ) => void;
  let remount!: () => void;
  let dispose = () => {};
  let disposeFeed!: () => void;
  createRoot((rootDispose) => {
    disposeFeed = rootDispose;
    const [events, writeEvents] = createSignal<readonly Event[]>([]);
    setEvents = writeEvents;
    const mount = () => {
      dispose();
      createRoot((effectDispose) => {
        dispose = effectDispose;
        createEventEffect({
          source: events,
          sequence: (event) => event.sequence,
          onEvent: (event) => received.push(event.sequence),
        });
        flush();
      });
    };
    remount = mount;
    mount();
  });

  setEvents([{ sequence: 1, value: "first" }]);
  flush();
  setEvents([{ sequence: 2, value: "second" }]);
  flush();
  remount();
  setEvents([{ sequence: 3, value: "third" }]);
  flush();
  expect(received).toEqual([1, 2, 3]);
  dispose();
  disposeFeed();
});

test("isolates synchronous handler failures and continues the batch", () => {
  const received: number[] = [];
  const errors: Array<[unknown, number]> = [];
  createRoot((dispose) => {
    const [events] = createSignal<readonly Event[]>([
      { sequence: 1, value: "broken" },
      { sequence: 2, value: "healthy" },
    ]);
    createEventEffect({
      source: events,
      sequence: (event) => event.sequence,
      consumeInitial: true,
      onEvent: (event) => {
        if (event.sequence === 1) throw new Error("failed");
        received.push(event.sequence);
      },
      onError: (error, event) => errors.push([error, event.sequence]),
    });
    flush();
    dispose();
  });

  expect(received).toEqual([2]);
  expect(errors).toHaveLength(1);
  expect(String(errors[0]?.[0])).toContain("failed");
  expect(errors[0]?.[1]).toBe(1);
});

test("reports asynchronous handler rejections with their event", async () => {
  const errors: Array<[unknown, number]> = [];
  createRoot((dispose) => {
    const [events] = createSignal<readonly Event[]>([
      { sequence: 7, value: "async" },
    ]);
    createEventEffect({
      source: events,
      sequence: (event) => event.sequence,
      consumeInitial: true,
      onEvent: async () => {
        throw new Error("async failed");
      },
      onError: (error, event) => errors.push([error, event.sequence]),
    });
    flush();
    dispose();
  });
  await Promise.resolve();
  await Promise.resolve();

  expect(errors).toHaveLength(1);
  expect(String(errors[0]?.[0])).toContain("async failed");
  expect(errors[0]?.[1]).toBe(7);
});
