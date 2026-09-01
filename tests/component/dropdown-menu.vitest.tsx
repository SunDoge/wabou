import { renderComponent } from "@wabou/test/component";
import { Button, DropdownMenu } from "@wabou/ui";
import { expect, test } from "vitest";

test("opens, skips disabled actions, and selects with the keyboard", () => {
  const actions: string[] = [];
  const screen = renderComponent(() => (
    <DropdownMenu
      aria-label="Project actions"
      items={[
        { id: "open", label: "Open" },
        { id: "rename", label: "Rename", disabled: true },
        { id: "delete", label: "Delete", destructive: true },
      ]}
      onAction={(id) => actions.push(id)}
      trigger={(trigger) => (
        <Button aria-label="Actions" variant="outline" {...trigger}>
          Actions
        </Button>
      )}
    />
  ));
  const trigger = screen.getByRole("button", { name: "Actions" });

  trigger.click();
  const menu = screen.getByRole("menu", { name: "Project actions" });
  expect(menu.closestByRole("presentation")?.className).toContain("rounded-lg");
  expect(screen.getByRole("menuitem", { name: "Open" }).className).toContain(
    "bg-control-hover",
  );
  expect(screen.getByRole("menuitem", { name: "Open" }).className).toContain(
    "rounded-md",
  );

  menu.press("ArrowDown");
  expect(screen.getByRole("menuitem", { name: "Delete" }).className).toContain(
    "bg-control-hover",
  );
  menu.press("Enter");

  expect(actions).toEqual(["delete"]);
  expect(screen.queryByRole("menu")).toBeNull();
});

test("opens at the last action with ArrowUp and closes with Escape", () => {
  const screen = renderComponent(() => (
    <DropdownMenu
      aria-label="Account actions"
      items={[
        { id: "profile", label: "Profile" },
        { id: "sign-out", label: "Sign out" },
      ]}
      trigger={(trigger) => (
        <Button aria-label="Account" {...trigger}>
          Account
        </Button>
      )}
    />
  ));
  const trigger = screen.getByRole("button", { name: "Account" });

  trigger.press("ArrowUp");
  expect(
    screen.getByRole("menuitem", { name: "Sign out" }).className,
  ).toContain("bg-control-hover");
  screen.getByRole("menu").press("Escape");
  expect(screen.queryByRole("menu")).toBeNull();
});

test("forwards popup motion configuration", () => {
  const screen = renderComponent(
    () => (
      <DropdownMenu
        aria-label="Animated actions"
        items={[{ id: "open", label: "Open" }]}
        motion={{ duration: 10, fromScale: 0.9 }}
        trigger={(trigger) => (
          <Button aria-label="Animated" {...trigger}>
            Animated
          </Button>
        )}
      />
    ),
    { clock: "fake" },
  );

  screen.getByRole("button", { name: "Animated" }).click();
  const panel = screen.getByRole("menu").closestByRole("presentation");
  expect(panel?.transform).toEqual([0.9, 0, 0, 0.9, 0, 0]);
});
