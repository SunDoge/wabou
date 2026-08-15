import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { createOverlayLayer, type DismissEvent } from "./overlay-layer";

const event = (): DismissEvent & { key: string } => ({
  key: "Escape",
  preventDefault() {},
  stopPropagation() {},
});

test("only the most recently opened overlay can dismiss", () => {
  createRoot((dispose) => {
    const [outerOpen, setOuterOpen] = createSignal(true);
    const [innerOpen, setInnerOpen] = createSignal(true);
    const dismissed: string[] = [];
    const outer = createOverlayLayer({
      open: outerOpen,
      onDismiss: () => dismissed.push("outer"),
    });
    const inner = createOverlayLayer({
      open: innerOpen,
      onDismiss: () => dismissed.push("inner"),
    });
    flush();

    outer.onEscape(event());
    inner.onEscape(event());
    expect(dismissed).toEqual(["inner"]);

    setInnerOpen(false);
    flush();
    outer.onOutside(event());
    expect(dismissed).toEqual(["inner", "outer"]);

    setOuterOpen(false);
    dispose();
  });
});

test("an overlay restores focus when closed or unmounted while open", () => {
  let restored = 0;
  let setOpen: ((open: boolean) => void) | undefined;
  const dispose = createRoot((dispose) => {
    const [open, updateOpen] = createSignal(true);
    setOpen = updateOpen;
    createOverlayLayer({
      open,
      onDismiss() {},
      returnFocus: () => ({ focus: () => restored++ }),
    });
    flush();
    return dispose;
  });

  setOpen?.(false);
  flush();
  expect(restored).toBe(1);
  setOpen?.(true);
  flush();
  dispose();
  expect(restored).toBe(2);
});

