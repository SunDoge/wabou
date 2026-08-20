import { renderComponent } from "@wabou/test/component";
import { Menubar, MenubarMenu } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const fileItems = [
  { id: "new", label: "New file" },
  { id: "open", label: "Open" },
] as const;
const editItems = [
  { id: "copy", label: "Copy" },
  { id: "paste", label: "Paste", disabled: true },
] as const;

test("switches an open menu by hover, click, and horizontal keys", () => {
  const screen = renderComponent(() => (
    <Menubar aria-label="Application menu">
      <MenubarMenu value="file" label="File" items={fileItems} />
      <MenubarMenu value="edit" label="Edit" items={editItems} />
    </Menubar>
  ));
  const menubar = screen.getByRole("menubar", { name: "Application menu" });
  const file = menubar.getByRole("menuitem", { name: "File" });
  const edit = menubar.getByRole("menuitem", { name: "Edit" });

  expect(file.focusOrder).toBe(0);
  file.click();
  edit.hover();
  expect(screen.queryByRole("menu", { name: "File menu" })).toBeNull();
  const editMenu = screen.getByRole("menu", { name: "Edit menu" });
  edit.click();
  expect(screen.getByRole("menu", { name: "Edit menu" })).not.toBeNull();
  editMenu.press("ArrowLeft");

  expect(screen.queryByRole("menu", { name: "Edit menu" })).toBeNull();
  expect(screen.getByRole("menu", { name: "File menu" })).not.toBeNull();
  expect(file.focused).toBe(true);
});

test("uses toolbar roving focus while closed and skips disabled menus", () => {
  const screen = renderComponent(() => (
    <Menubar aria-label="Document menu">
      <MenubarMenu value="file" label="File" items={fileItems} />
      <MenubarMenu value="edit" label="Edit" items={editItems} disabled />
      <MenubarMenu value="help" label="Help" items={fileItems} />
    </Menubar>
  ));
  const menubar = screen.getByRole("menubar", { name: "Document menu" });
  const file = menubar.getByRole("menuitem", { name: "File" });
  const help = menubar.getByRole("menuitem", { name: "Help" });

  file.focus();
  file.press("ArrowRight");
  expect(help.focused).toBe(true);
  expect(help.focusOrder).toBe(0);
  expect(screen.queryByRole("menu")).toBeNull();
});

test("supports an application-owned null open value", () => {
  const Controlled = () => {
    const [open, setOpen] = createSignal<string | null>(null);
    return (
      <Menubar
        aria-label="Controlled menu"
        value={open()}
        onValueChange={setOpen}
      >
        <MenubarMenu value="file" label="File" items={fileItems} />
      </Menubar>
    );
  };
  const screen = renderComponent(Controlled);

  expect(screen.queryByRole("menu")).toBeNull();
  screen.getByRole("menuitem", { name: "File" }).click();
  expect(screen.getByRole("menu", { name: "File menu" })).not.toBeNull();
  screen.getByRole("menu").press("Escape");
  expect(screen.queryByRole("menu")).toBeNull();
});
