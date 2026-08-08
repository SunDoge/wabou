import { describe, expect, test } from "bun:test";
import { createMemoryHistory } from "./history";

describe("createMemoryHistory", () => {
  test("pushes, replaces, traverses, and truncates forward entries", () => {
    const history = createMemoryHistory();
    const visited: string[] = [];
    history.listen((entry) => visited.push(entry.value));

    history.set({ value: "/one" });
    history.set({ value: "/two", state: { from: 1 } });
    expect(history.canGoBack()).toBe(true);
    history.back();
    expect(history.get().value).toBe("/one");
    expect(history.canGoForward()).toBe(true);
    history.set({ value: "/three" });
    expect(history.canGoForward()).toBe(false);
    history.set({ value: "/final", replace: true });
    expect(history.get()).toEqual({ value: "/final" });
    expect(visited).toEqual(["/one", "/two", "/one", "/three", "/final"]);
  });

  test("clamps initial index and out-of-range traversal", () => {
    const history = createMemoryHistory({
      initialEntries: ["/one", "/two"],
      initialIndex: 99,
    });
    expect(history.get().value).toBe("/two");
    history.go(-99);
    expect(history.get().value).toBe("/one");
    history.go(Number.POSITIVE_INFINITY);
    expect(history.get().value).toBe("/one");
  });
});
