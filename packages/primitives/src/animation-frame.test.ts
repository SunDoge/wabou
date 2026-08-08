import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { createRoot } from "solid-js";
import { createAnimationFrame } from "./animation-frame";

describe("createAnimationFrame", () => {
  const originalRequest = globalThis.requestAnimationFrame;
  const originalCancel = globalThis.cancelAnimationFrame;
  let callbacks: Map<number, FrameRequestCallback>;
  let cancelled: number[];
  let nextId: number;

  beforeEach(() => {
    callbacks = new Map();
    cancelled = [];
    nextId = 1;
    globalThis.requestAnimationFrame = (callback) => {
      const id = nextId++;
      callbacks.set(id, callback);
      return id;
    };
    globalThis.cancelAnimationFrame = (id) => {
      cancelled.push(id);
      callbacks.delete(id);
    };
  });

  afterEach(() => {
    globalThis.requestAnimationFrame = originalRequest;
    globalThis.cancelAnimationFrame = originalCancel;
  });

  test("schedules frames until the callback returns false", () => {
    const timestamps: number[] = [];
    createRoot(() => {
      createAnimationFrame((timestamp) => {
        timestamps.push(timestamp);
        return timestamps.length < 2;
      });
    });

    callbacks.get(1)?.(10);
    callbacks.get(2)?.(26);

    expect(timestamps).toEqual([10, 26]);
    expect(callbacks.size).toBe(2);
    expect(nextId).toBe(3);
  });

  test("cancels the pending frame when its owner is disposed", () => {
    let dispose = () => {};
    createRoot((rootDispose) => {
      dispose = rootDispose;
      createAnimationFrame(() => {});
    });

    dispose();

    expect(cancelled).toEqual([1]);
    expect(callbacks.size).toBe(0);
  });
});
