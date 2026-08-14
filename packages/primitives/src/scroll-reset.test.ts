import { describe, expect, test } from "bun:test";
import { createRoot, createSignal } from "solid-js";
import { isServer } from "@solidjs/web";
import { createScrollReset } from "./scroll-reset";

describe("createScrollReset", () => {
  test("resets only the explicitly supplied viewport when the key changes", async () => {
    const calls: Array<{ left?: number; top?: number }> = [];
    const unrelated: Array<{ left?: number; top?: number }> = [];
    const target = {
      scrollTo: (options: { left?: number; top?: number }) =>
        calls.push(options),
    };
    const other = {
      scrollTo: (options: { left?: number; top?: number }) =>
        unrelated.push(options),
    };
    let dispose = () => {};
    let setPath = (_path: string) => {};
    createRoot((rootDispose) => {
      dispose = rootDispose;
      const [path, updatePath] = createSignal("/components/button");
      setPath = updatePath;
      const reset = createScrollReset({ target: () => target, key: path });

      expect(calls).toEqual([]);
      reset();
      expect(calls).toEqual([{ left: 0, top: 0 }]);
      expect(unrelated).toEqual([]);
    });
    await Promise.resolve();
    setPath("/components/card");
    await Promise.resolve();
    expect(calls).toHaveLength(isServer ? 1 : 2);
    other.scrollTo({ top: 20 });
    expect(unrelated).toEqual([{ top: 20 }]);
    dispose();
  });
});
