import {
  assertFocusOwnerCount,
  assertSingleSurfaceOwner,
  renderComponent,
} from "@wabou/test/component";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
  InputGroupTextArea,
  Text,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("records the complete authored InputGroup contract without a native host", () => {
  const screen = renderComponent(() => (
    <InputGroup aria-label="Project URL">
      <InputGroupAddon align="inline-start" aria-label="Focus hostname">
        <InputGroupText>https://</InputGroupText>
      </InputGroupAddon>
      <InputGroupInput aria-label="Hostname" placeholder="example.com" />
      <InputGroupButton>Copy</InputGroupButton>
    </InputGroup>
  ));
  const group = screen.getByRole("group", { name: "Project URL" });

  expect(assertSingleSurfaceOwner(group).name).toBe("Project URL");
  expect(assertFocusOwnerCount(group, 2)).toHaveLength(2);
  expect(group.className).toContain("rounded-lg");
  expect(
    screen.getByRole("textbox", { name: "Hostname" }).className,
  ).not.toContain("rounded");
  const input = screen.getByRole("textbox", { name: "Hostname" });
  expect(input.attribute("placeholder")).toBe("example.com");
  expect(input.className).toContain("h-full");
  expect(input.attribute("data-wabou-owns")).toBe("native-editor");
  expect(screen.getByRole("group", { name: "Focus hostname" }).text).toBe(
    "https://",
  );
  expect(screen.getByRole("button", { name: "Copy" })).toBeDefined();
});

test("uses the shared disabled control surface", () => {
  const screen = renderComponent(() => (
    <InputGroup aria-label="Unavailable URL" disabled>
      <InputGroupInput aria-label="Unavailable hostname" disabled />
    </InputGroup>
  ));
  const group = screen.getByRole("group", { name: "Unavailable URL" });

  expect(group.className).toContain("bg-surface-muted");
  expect(group.className).toContain("border-subtle");
  expect(group.className).toContain("text-muted");
  expect(group.className).toContain("cursor-not-allowed");
  expect(group.className).toContain("opacity-60");
});

test("focuses the registered native editor through an addon", () => {
  const screen = renderComponent(() => (
    <InputGroup aria-label="Project URL">
      <InputGroupAddon align="inline-start" aria-label="Focus hostname">
        <Text>https://</Text>
      </InputGroupAddon>
      <InputGroupInput aria-label="Hostname" />
    </InputGroup>
  ));

  screen.getByRole("group", { name: "Focus hostname" }).click();

  expect(screen.getByRole("textbox", { name: "Hostname" }).focused).toBe(true);
  expect(
    screen.getByRole("group", { name: "Project URL" }).className,
  ).toContain("border-focus");
});

test("quiet groups reveal their boundary only while focused", () => {
  const screen = renderComponent(() => (
    <InputGroup aria-label="Navigation search" variant="quiet">
      <InputGroupInput aria-label="Search projects" />
    </InputGroup>
  ));
  const group = screen.getByRole("group", { name: "Navigation search" });
  const input = screen.getByRole("textbox", { name: "Search projects" });

  expect(group.className).toContain("bg-transparent");
  expect(group.className).toContain("border-transparent");
  expect(group.className).toContain("shadow-none");
  input.focus();
  expect(group.className).toContain("border-focus");
  expect(group.className).not.toContain("border-transparent");
});

test("keeps multiline input chrome on the compound surface", () => {
  const screen = renderComponent(() => (
    <InputGroup aria-label="Comment" orientation="vertical">
      <InputGroupTextArea aria-label="Comment body" />
    </InputGroup>
  ));
  const group = screen.getByRole("group", { name: "Comment" });
  const editor = screen.getByRole("textbox", { name: "Comment body" });

  expect(assertSingleSurfaceOwner(group)).toEqual(group);
  expect(editor.attribute("data-wabou-owns")).toBe("native-editor");
  expect(editor.className).not.toContain("rounded");
  expect(editor.className).not.toContain("border");
  expect(editor.className).not.toContain("shadow");
  expect(editor.className).not.toContain("bg-");
});
