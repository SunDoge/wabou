import { expect, test } from "bun:test";
import {
  createElement,
  dispatchEvent,
  EVENT_CODE,
  getMountRoot,
  type Handle,
  mount,
} from "@wabou/solid-renderer";
import { createRoot, createSignal } from "solid-js";
import { Modal, type ModalControls, type ModalTriggerProps } from "./modal";
import { View } from "./view";

const event = () => ({
  preventDefault() {},
  stopPropagation() {},
});

test("Modal mounts on the native modal plane and closes from Escape", () => {
  const disposeMount = mount(() => null);
  const root = getMountRoot();

  createRoot((dispose) => {
    let triggerProps: ModalTriggerProps | undefined;
    let modalControls: ModalControls | undefined;
    let triggerHandle: Handle | undefined;
    let restoredFocus = 0;
    const changes: Array<[boolean, string]> = [];

    Modal({
      "aria-label": "Delete item",
      onOpenChange: (open, reason) => changes.push([open, reason]),
      trigger: (bindings) => {
        triggerProps = bindings;
        const node = createElement("view");
        triggerHandle = node;
        node.focus = () => restoredFocus++;
        bindings.ref(node);
        return node as never;
      },
      children: (controls) => {
        modalControls = controls;
        return View({});
      },
    });

    expect(triggerProps?.["aria-expanded"]).toBe(false);
    triggerProps?.onClick(event());
    expect(triggerProps?.["aria-expanded"]).toBe(true);

    const modalPlane = root.lastChild;
    const backdrop = modalPlane?.firstChild;
    const content = backdrop?.firstChild;
    expect(modalPlane).not.toBeNull();
    expect(backdrop).not.toBeNull();
    expect(content).not.toBeNull();

    dispatchEvent(content?.id ?? 0, EVENT_CODE.click, "");
    expect(root.lastChild).toBe(modalPlane);

    const prevented = dispatchEvent(
      content?.id ?? 0,
      EVENT_CODE.keydown,
      JSON.stringify({ key: "Escape" }),
    );
    expect(prevented).toBe(true);
    expect(root.lastChild).toBeNull();
    expect(triggerProps?.["aria-expanded"]).toBe(false);
    expect(changes).toEqual([
      [true, "trigger"],
      [false, "escape"],
    ]);
    expect(restoredFocus).toBe(1);
    expect(triggerHandle).toBeDefined();

    triggerProps?.onClick(event());
    modalControls?.close();
    expect(changes.at(-1)).toEqual([false, "programmatic"]);
    expect(restoredFocus).toBe(2);

    dispose();
  });

  disposeMount();
});

test("Modal can keep backdrop and Escape dismissal disabled", () => {
  const disposeMount = mount(() => null);
  const root = getMountRoot();

  createRoot((dispose) => {
    Modal({
      "aria-label": "Persistent dialog",
      defaultOpen: true,
      closeOnBackdrop: false,
      closeOnEscape: false,
      children: View({}),
    });
    const modalPlane = root.lastChild;
    const backdrop = modalPlane?.firstChild;

    expect(
      dispatchEvent(
        backdrop?.id ?? 0,
        EVENT_CODE.keydown,
        JSON.stringify({ key: "Escape" }),
      ),
    ).toBe(false);
    dispatchEvent(backdrop?.id ?? 0, EVENT_CODE.click, "");
    expect(root.lastChild).toBe(modalPlane);

    dispose();
  });

  expect(root.lastChild).toBeNull();
  disposeMount();
});

test("controlled Modal restores focus only after open actually becomes false", () => {
  const disposeMount = mount(() => null);
  const root = getMountRoot();

  createRoot((dispose) => {
    const [open, setOpen] = createSignal(true);
    let restoredFocus = 0;
    let requestedClose = false;
    Modal({
      "aria-label": "Controlled dialog",
      get open() {
        return open();
      },
      onOpenChange: (next) => {
        requestedClose = !next;
      },
      trigger: (bindings) => {
        const node = createElement("view");
        node.focus = () => restoredFocus++;
        bindings.ref(node);
        return node as never;
      },
      children: View({}),
    });
    const modalPlane = root.lastChild;
    const backdrop = modalPlane?.firstChild;

    dispatchEvent(backdrop?.id ?? 0, EVENT_CODE.click, "");
    expect(requestedClose).toBe(true);
    expect(root.lastChild).toBe(modalPlane);
    expect(restoredFocus).toBe(0);

    setOpen(false);
    expect(root.lastChild).toBeNull();
    expect(restoredFocus).toBe(1);
    dispose();
  });

  disposeMount();
});
