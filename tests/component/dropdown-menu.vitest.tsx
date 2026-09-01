import { renderComponent } from "@wabou/test/component";
import { Button, DropdownMenu } from "@wabou/ui";
import { expect, test } from "vitest";

const fixtureIcon =
  '<svg viewBox="0 0 24 24"><path d="M3 6h7l2 2h9v10H3z"/></svg>';

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

test("keeps the authored endpoint while native popup positioning is pending", () => {
  const screen = renderComponent(() => (
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
  ));

  screen.getByRole("button", { name: "Animated" }).click();
  const panel = screen.getByRole("menu").closestByRole("presentation");
  expect(panel?.transform).toEqual([1, 0, 0, 1, 0, 0]);
  expect(panel?.attribute("__wabou_native_transition")).toBeNull();
});

test("owns aligned leading status and shortcut slots", () => {
  const screen = renderComponent(() => (
    <DropdownMenu
      aria-label="View actions"
      defaultOpen
      items={[
        {
          id: "open",
          label: "Open folder",
          icon: fixtureIcon,
          shortcut: "Ctrl O",
        },
        { id: "hidden", label: "Show hidden files", checked: true },
        { id: "plain", label: "Plain action" },
      ]}
      trigger={(trigger) => <Button {...trigger}>View</Button>}
    />
  ));

  const checked = screen.getByRole("menuitem", {
    name: "Show hidden files",
  });
  expect(checked.checked).toBe(true);
  expect(screen.getByRole("menuitem", { name: "Open folder" }).text).toContain(
    "Ctrl O",
  );
  const plain = screen.getByRole("menuitem", { name: "Plain action" });
  expect(plain.children[0]?.className).toContain("w-4 h-4");
});
