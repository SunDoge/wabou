import { renderComponent } from "@wabou/test/component";
import {
  Button,
  ButtonGroup,
  ButtonGroupSeparator,
  ButtonGroupText,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("turns ordinary buttons into one horizontal control surface", () => {
  const screen = renderComponent(() => (
    <ButtonGroup aria-label="History">
      <Button variant="outline">Back</Button>
      <ButtonGroupSeparator />
      <Button variant="outline">Forward</Button>
      <ButtonGroupText>2 selected</ButtonGroupText>
      <Button variant="destructive">Delete</Button>
    </ButtonGroup>
  ));
  const group = screen.getByRole("group", { name: "History" });
  expect(group.className).toContain("border-strong");
  expect(group.className).toContain("rounded-lg");
  const buttons = screen.getAllByRole("button");
  expect(buttons[0]?.className).toContain("rounded-l-lg");
  expect(buttons[1]?.className).toContain("rounded-none");
  expect(buttons[2]?.className).toContain("rounded-r-lg");
  for (const button of buttons) {
    expect(button.className).toContain("border-transparent");
  }
  expect(screen.getByRole("separator").className).toContain("w-px");
});

test("propagates vertical composition without changing standalone buttons", () => {
  const screen = renderComponent(() => (
    <>
      <ButtonGroup orientation="vertical" aria-label="Account">
        <Button variant="outline">Profile</Button>
        <ButtonGroupSeparator orientation="horizontal" />
        <Button variant="outline">Sign out</Button>
      </ButtonGroup>
      <Button aria-label="Standalone">Save</Button>
    </>
  ));
  const standalone = screen.getByRole("button", { name: "Standalone" });
  expect(standalone.className).toContain("h-8");
  expect(standalone.className).toContain("rounded-md");
  expect(screen.getByRole("group", { name: "Account" }).className).toContain(
    "flex-col",
  );
  expect(screen.getByRole("separator").className).toContain("h-px");
  expect(screen.getByRole("button", { name: "Profile" }).className).toContain(
    "rounded-t-lg",
  );
  expect(screen.getByRole("button", { name: "Sign out" }).className).toContain(
    "rounded-b-lg",
  );
  expect(
    screen.getByRole("button", { name: "Standalone" }).className,
  ).not.toContain("rounded-none");
});

test("keeps control labels non-selectable while allowing an explicit override", () => {
  const screen = renderComponent(() => (
    <>
      <Button aria-label="Default selection">Default</Button>
      <Button aria-label="Selectable label" class="select-text">
        Selectable
      </Button>
    </>
  ));

  expect(
    screen.getByRole("button", { name: "Default selection" }).className,
  ).toContain("select-none");
  const selectable = screen.getByRole("button", { name: "Selectable label" });
  expect(selectable.className).toContain("select-text");
  expect(selectable.className).not.toContain("select-none");
});

test("inherits group geometry while enforcing one disabled palette", () => {
  const screen = renderComponent(() => (
    <ButtonGroup
      aria-label="Bulk actions"
      size="lg"
      variant="secondary"
      disabled
    >
      <Button>Archive</Button>
      <Button size="sm" variant="destructive" disabled={false}>
        Delete
      </Button>
      <ButtonGroupText>2 selected</ButtonGroupText>
    </ButtonGroup>
  ));
  const group = screen.getByRole("group", { name: "Bulk actions" });
  const archive = screen.getByRole("button", { name: "Archive" });
  const remove = screen.getByRole("button", { name: "Delete" });

  expect(group.orientation).toBe("horizontal");
  expect(group.disabled).toBe(true);
  expect(group.attribute("size")).toBeNull();
  expect(group.attribute("variant")).toBeNull();
  expect(archive.className).toContain("h-10");
  expect(archive.className).toContain("bg-surface-muted");
  expect(archive.className).toContain("opacity-60");
  expect(archive.disabled).toBe(true);
  expect(remove.className).toContain("h-7");
  expect(remove.className).toContain("bg-surface-muted");
  expect(remove.className).toContain("opacity-60");
  expect(remove.disabled).toBe(true);
  expect(group.children[2]?.className).toContain("h-10");
});
