import { renderComponent } from "@wabou/test/component";
import { Button, Sheet, SheetTitle } from "@wabou/ui";
import { createComponent, createSignal } from "solid-js";
import { expect, test } from "vitest";

test("slides a solid edge panel fully out before unmounting it", () => {
  const screen = renderComponent(() => (
    <Sheet
      aria-label="Preferences"
      side="right"
      trigger={(trigger) => <Button {...trigger}>Preferences</Button>}
    >
      {(controls) => (
        <>
          <SheetTitle>Preferences</SheetTitle>
          <Button onClick={controls.close}>Done</Button>
        </>
      )}
    </Sheet>
  ));

  screen.getByRole("button", { name: "Preferences" }).click();
  const sheet = screen.getByRole("dialog", { name: "Preferences" });
  expect(sheet.className).toContain("w-[400px]");
  expect(sheet.className).toContain("border-l");
  expect(sheet.transform).toEqual([1, 0, 0, 1, 0, 0]);
  const entering = JSON.parse(
    sheet.attribute("__wabou_native_transition") ?? "null",
  );
  expect(entering).toMatchObject({
    duration: 0.22,
    easing: "easeOut",
    fromTransform: [1, 0, 0, 1, 400, 0],
    toTransform: [1, 0, 0, 1, 0, 0],
    fromOpacity: 1,
    toOpacity: 1,
  });
  sheet.emit("transitionend", { generation: entering.generation });
  expect(sheet.transform).toEqual([1, 0, 0, 1, 0, 0]);
  screen.getByRole("button", { name: "Done" }).click();
  expect(screen.queryByRole("dialog") !== null).toBe(true);
  expect(sheet.attribute("aria-hidden")).toBe("true");
  expect(sheet.parent?.className).not.toContain("backdrop-blur-sm");
  expect(sheet.parent?.style("background-color")).toEqual({
    kind: 5,
    value: 0,
  });
  expect(sheet.parent?.attribute("__wabou_native_transition")).toBeNull();
  const exiting = JSON.parse(
    sheet.attribute("__wabou_native_transition") ?? "null",
  );
  expect(exiting).toMatchObject({
    duration: 0.18,
    easing: "linear",
    fromTransform: [1, 0, 0, 1, 0, 0],
    toTransform: [1, 0, 0, 1, 400, 0],
  });
  sheet.emit("transitionend", { generation: entering.generation });
  expect(screen.queryByRole("dialog")).not.toBeNull();
  sheet.emit("transitionend", { generation: exiting.generation });
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("supports top and bottom placement without a separate overlay primitive", () => {
  const top = renderComponent(() => (
    <Sheet
      aria-label="Top sheet"
      side="top"
      trigger={(trigger) => <Button {...trigger}>Open top</Button>}
    />
  ));
  top.getByRole("button", { name: "Open top" }).click();
  const topSheet = top.getByRole("dialog");
  expect(topSheet.className).toContain("border-b");
  expect(
    JSON.parse(topSheet.attribute("__wabou_native_transition") ?? "null")
      .fromTransform,
  ).toEqual([1, 0, 0, 1, 0, -320]);
  top.dispose();

  const bottom = renderComponent(() => (
    <Sheet
      aria-label="Bottom sheet"
      side="bottom"
      trigger={(trigger) => <Button {...trigger}>Open bottom</Button>}
    />
  ));
  bottom.getByRole("button", { name: "Open bottom" }).click();
  const bottomSheet = bottom.getByRole("dialog");
  expect(bottomSheet.className).toContain("border-t");
  expect(
    JSON.parse(bottomSheet.attribute("__wabou_native_transition") ?? "null")
      .fromTransform,
  ).toEqual([1, 0, 0, 1, 0, 320]);
});

test("waits for a controlled owner before starting the exit transition", () => {
  let commitOpen: ((open: boolean) => void) | undefined;
  let closeRequested = false;
  const screen = renderComponent(() => {
    const [open, setOpen] = createSignal(false);
    commitOpen = setOpen;
    return createComponent(Sheet, {
      "aria-label": "Controlled sheet",
      get open() {
        return open();
      },
      onOpenChange(next) {
        if (next) setOpen(true);
        else closeRequested = true;
      },
      trigger: (trigger) =>
        createComponent(Button, {
          ...trigger,
          children: "Open controlled sheet",
        }),
      children: (controls) => <Button onClick={controls.close}>Close</Button>,
    });
  });

  screen.getByRole("button", { name: "Open controlled sheet" }).click();
  const sheet = screen.getByRole("dialog", { name: "Controlled sheet" });
  screen.getByRole("button", { name: "Close" }).click();
  expect(closeRequested).toBe(true);
  expect(sheet.attribute("aria-hidden")).toBeNull();
  expect(sheet.parent?.className).toContain("backdrop-blur-sm");
  expect(sheet.transform).toEqual([1, 0, 0, 1, 0, 0]);

  commitOpen?.(false);
  screen.flush();
  expect(sheet.attribute("aria-hidden")).toBe("true");
  expect(sheet.parent?.className).not.toContain("backdrop-blur-sm");
  expect(sheet.parent?.style("pointer-events")).toBe("none");
});
