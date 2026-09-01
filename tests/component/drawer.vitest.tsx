import { renderComponent } from "@wabou/test/component";
import { Button, Drawer, DrawerClose, DrawerTitle } from "@wabou/ui";
import { expect, test } from "vitest";

test("removes the scrim immediately while retaining the drawer for native exit", () => {
  const screen = renderComponent(() => (
    <Drawer
      aria-label="Create task"
      direction="bottom"
      trigger={(trigger) => <Button {...trigger}>Open drawer</Button>}
    >
      <DrawerTitle>Create task</DrawerTitle>
      <DrawerClose>Cancel</DrawerClose>
    </Drawer>
  ));

  screen.getByRole("button", { name: "Open drawer" }).click();
  const drawer = screen.getByRole("dialog", { name: "Create task" });
  const entering = JSON.parse(
    drawer.attribute("__wabou_native_transition") ?? "null",
  );
  expect(entering).toMatchObject({
    duration: 0.22,
    fromTransform: [1, 0, 0, 1, 0, 48],
    toTransform: [1, 0, 0, 1, 0, 0],
    fromOpacity: 1,
    toOpacity: 1,
  });
  drawer.emit("transitionend", { generation: entering.generation });

  screen.getByRole("button", { name: "Cancel" }).click();
  expect(drawer.attribute("aria-hidden")).toBe("true");
  expect(drawer.interactionBlocked).toBe(true);
  expect(drawer.parent?.className).not.toContain("backdrop-blur-sm");
  expect(drawer.parent?.style("background-color")).toEqual({
    kind: 5,
    value: 0,
  });
  expect(drawer.parent?.attribute("__wabou_native_transition")).toBeNull();

  const exiting = JSON.parse(
    drawer.attribute("__wabou_native_transition") ?? "null",
  );
  expect(exiting).toMatchObject({
    fromTransform: [1, 0, 0, 1, 0, 0],
    toTransform: [1, 0, 0, 1, 0, 48],
  });
  drawer.emit("transitionend", { generation: entering.generation });
  expect(screen.queryByRole("dialog", { name: "Create task" })).not.toBeNull();
  drawer.emit("transitionend", { generation: exiting.generation });
  expect(screen.queryByRole("dialog", { name: "Create task" })).toBeNull();
});
