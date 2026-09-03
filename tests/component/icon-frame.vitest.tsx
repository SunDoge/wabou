import { renderComponent } from "@wabou/test/component";
import { Icon, IconFrame } from "@wabou/ui";
import { expect, test } from "vitest";

const icon = '<svg viewBox="0 0 24 24"><path d="M4 4h16v16H4z"/></svg>';

test("IconFrame owns square sizing and centering", () => {
  const screen = renderComponent(() => (
    <IconFrame
      source={icon}
      size="lg"
      variant="selected"
      label="Preview image"
    />
  ));

  const renderedIcon = screen.getByRole("img", { name: "Preview image" });
  const frame = renderedIcon.parent;
  expect(frame?.className).toContain("flex");
  expect(frame?.className).toContain("items-center");
  expect(frame?.className).toContain("justify-center");
  expect(frame?.className).toContain("w-12");
  expect(frame?.className).toContain("h-12");
  expect(frame?.className).toContain("bg-selected");
  expect(renderedIcon.style("width")).toEqual({ kind: 1, value: 23 });
  expect(renderedIcon.style("height")).toEqual({ kind: 1, value: 23 });
  expect(renderedIcon.style("pointer-events")).toBe("none");
});

test("Icon lets an explicitly interactive graphic opt into hit testing", () => {
  const screen = renderComponent(() => (
    <Icon
      source={icon}
      label="Interactive image"
      style={{ "pointer-events": "auto" }}
    />
  ));

  expect(
    screen.getByRole("img", { name: "Interactive image" }).style(
      "pointer-events",
    ),
  ).toBe("auto");
});
