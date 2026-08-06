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

    const planeRoot = root.lastChild;
    const container = planeRoot?.firstChild;
    expect(planeRoot?.tag).toBe("view");
    expect(container?.firstChild).toBe(child);
    expect(child.parent).toBe(container ?? null);

    dispose();
    expect(root.lastChild).toBeNull();
  });

  disposeMount();
});

test("Portal instances share plane roots and modal remains a separate plane", () => {
  const disposeMount = mount(() => null);
  const root = getMountRoot();

  createRoot((dispose) => {
    Portal({ children: createElement("view") as unknown as JSX.Element });
    Portal({ children: createElement("view") as unknown as JSX.Element });
    expect(root.firstChild).toBe(root.lastChild);
    expect(root.firstChild?.firstChild).not.toBe(root.firstChild?.lastChild);

    Portal({
      plane: "modal",
      children: createElement("view") as unknown as JSX.Element,
    });
    expect(root.firstChild).not.toBe(root.lastChild);

    dispose();
    expect(root.firstChild).toBeNull();
  });

  disposeMount();
});
