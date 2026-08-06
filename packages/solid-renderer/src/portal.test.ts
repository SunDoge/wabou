import { expect, test } from "bun:test";
import { createRoot, type JSX } from "solid-js";
import { createElement, getMountRoot, mount } from "./index";
import { Portal } from "./portal";

test("Portal mounts its container directly under the native window root", () => {
  const disposeMount = mount(() => null);
  const root = getMountRoot();

  createRoot((dispose) => {
    const child = createElement("view");
    Portal({ children: child as unknown as JSX.Element, role: "presentation" });

    expect(root.lastChild?.tag).toBe("view");
    expect(root.lastChild?.firstChild).toBe(child);
    expect(child.parent).toBe(root.lastChild);

    dispose();
    expect(root.lastChild).toBeNull();
  });

  disposeMount();
});
