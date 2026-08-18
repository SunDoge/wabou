import { describe, expect, test } from "bun:test";
import { isServer } from "@wabou/solid-renderer";
import { createRoot, createSignal, flush } from "solid-js";
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
    flush();
    await Promise.resolve();
    expect(calls).toHaveLength(isServer ? 1 : 2);
    other.scrollTo({ top: 20 });
    expect(unrelated).toEqual([{ top: 20 }]);
    dispose();
  });
});
