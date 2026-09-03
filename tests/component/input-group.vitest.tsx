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
  expect(screen.snapshot()).toMatchInlineSnapshot(`
    [
      {
        "attributes": {
          "aria-label": "Project URL",
          "data-wabou-owns": "surface focus-ring",
          "role": "group",
        },
        "children": [
          {
            "attributes": {
              "aria-label": "Focus hostname",
              "role": "group",
            },
            "children": [
              {
                "attributes": {
                  "role": "label",
                },
                "children": [
                  {
                    "name": "https://",
                    "tag": "#text",
                    "text": "https://",
                  },
                ],
                "className": "flex-none text-sm text-muted",
                "name": "https://",
                "role": "label",
                "tag": "text",
                "text": "https://",
              },
            ],
            "className": "h-full flex-none px-3 flex items-center justify-center gap-2 text-sm text-muted",
            "name": "Focus hostname",
            "role": "group",
            "tag": "view",
            "text": "https://",
          },
          {
            "attributes": {
              "aria-disabled": "false",
              "aria-label": "Hostname",
              "data-wabou-owns": "native-editor",
              "placeholder": "example.com",
              "role": "textbox",
            },
            "className": "w-full flex items-center py-2 text-primary px-2.5 gap-2 text-sm h-full flex-1 min-w-0",
            "focusOrder": 0,
            "name": "Hostname",
            "role": "textbox",
            "tag": "input",
          },
          {
            "attributes": {
              "aria-busy": "false",
              "aria-disabled": "false",
              "role": "button",
            },
            "children": [
              {
                "name": "Copy",
                "tag": "#text",
                "text": "Copy",
              },
            ],
            "className": "select-none inline-flex flex-none overflow-hidden whitespace-nowrap items-center justify-center border font-medium bg-transparent text-secondary border-transparent h-7 px-2 gap-1 text-xs rounded-md mx-1",
            "focusOrder": 0,
            "name": "Copy",
            "role": "button",
            "styles": {
              "align-items": "center",
              "border-width": "1",
              "cursor": "pointer",
              "display": "flex",
              "flex-shrink": "0",
              "opacity": "1",
              "white-space": "nowrap",
            },
            "tag": "button",
            "text": "Copy",
          },
        ],
        "className": "relative w-full min-w-0 flex rounded-lg border shadow-xs h-8 flex-row items-center border-strong bg-input",
        "name": "Project URL",
        "role": "group",
        "tag": "view",
        "text": "https://Copy",
      },
    ]
  `);
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
