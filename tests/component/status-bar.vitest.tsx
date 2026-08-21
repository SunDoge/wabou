import { renderComponent } from "@wabou/test/component";
import { StatusBar, StatusBarItem, StatusBarSeparator } from "@wabou/ui";
import { expect, test } from "vitest";

test("composes stable desktop status regions", () => {
  const screen = renderComponent(() => (
    <StatusBar aria-label="Editor status">
      <StatusBarItem grow>main.rs</StatusBarItem>
      <StatusBarSeparator />
      <StatusBarItem>Ln 12, Col 4</StatusBarItem>
      <StatusBarSeparator />
      <StatusBarItem>UTF-8</StatusBarItem>
    </StatusBar>
  ));

  expect(
    screen.getByRole("status", { name: "Editor status" }).className,
  ).toContain("h-7");
  expect(screen.getAllByRole("separator")).toHaveLength(2);
  expect(screen.roots[0]?.text).toContain("Ln 12, Col 4");
});
