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
    </ButtonGroup>
  ));
  const group = screen.getByRole("group", { name: "History" });
  expect(group.className).toContain("overflow-hidden");
  expect(group.className).toContain("border-strong");
  expect(group.className).toContain("rounded-lg");
  for (const button of screen.getAllByRole("button")) {
    expect(button.className).toContain("rounded-none");
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
  expect(standalone.className).toContain("rounded-lg");
  expect(screen.getByRole("group", { name: "Account" }).className).toContain(
    "flex-col",
  );
  expect(screen.getByRole("separator").className).toContain("h-px");
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
