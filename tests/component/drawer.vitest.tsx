import { renderComponent } from "@wabou/test/component";
import { Button, Drawer, DrawerClose, DrawerTitle } from "@wabou/ui";
import { expect, test } from "vitest";

test("removes the scrim immediately while retaining the drawer for JS exit", async () => {
  const screen = renderComponent(
    () => (
      <Drawer
        aria-label="Create task"
        direction="bottom"
        trigger={(trigger) => <Button {...trigger}>Open drawer</Button>}
      >
        <DrawerTitle>Create task</DrawerTitle>
        <DrawerClose>Cancel</DrawerClose>
      </Drawer>
    ),
    { clock: "fake" },
  );

  screen.getByRole("button", { name: "Open drawer" }).click();
  const drawer = screen.getByRole("dialog", { name: "Create task" });
  expect(drawer.transform).toEqual([1, 0, 0, 1, 0, 48]);
  expect(drawer.attribute("__wabou_native_transition")).toBeNull();
  await screen.advanceTime(220);
  expect(drawer.transform).toEqual([1, 0, 0, 1, 0, 0]);

  screen.getByRole("button", { name: "Cancel" }).click();
  expect(drawer.attribute("aria-hidden")).toBe("true");
  expect(drawer.interactionBlocked).toBe(true);
  expect(drawer.parent?.className).not.toContain("backdrop-blur-sm");
  expect(drawer.parent?.style("background-color")).toEqual({
    kind: 5,
    value: 0,
  });
  expect(drawer.parent?.attribute("__wabou_native_transition")).toBeNull();

  await screen.advanceTime(100);
  expect(screen.queryByRole("dialog", { name: "Create task" })).not.toBeNull();
  expect((drawer.transform?.[5] ?? 0) > 0).toBe(true);
  await screen.advanceTime(120);
  expect(screen.queryByRole("dialog", { name: "Create task" })).toBeNull();
});
