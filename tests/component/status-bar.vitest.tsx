import { renderComponent } from "@wabou/test/component";
import {
  StatusBar,
  StatusBarGroup,
  StatusBarIndicator,
  StatusBarItem,
  StatusBarSeparator,
  Text,
  View,
} from "@wabou/ui";
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

  const status = screen.getByRole("status", { name: "Editor status" });
  expect(status.className).toContain("h-7");
  expect(screen.getAllByRole("separator")).toHaveLength(2);
  expect(screen.roots[0]?.text).toContain("Ln 12, Col 4");
  expect(status.children[0]?.attribute("grow")).toBeNull();
});

test("groups rich status content without sacrificing shrink semantics", () => {
  const screen = renderComponent(() => (
    <StatusBar aria-label="Workspace status">
      <StatusBarGroup grow>
        <View aria-hidden="true" class="w-3 h-3" />
        <Text class="truncate">A very long workspace name</Text>
      </StatusBarGroup>
      <StatusBarGroup>
        <StatusBarIndicator tone="success" />
        <Text>Ready</Text>
      </StatusBarGroup>
      <StatusBarGroup shrink>
        <Text>feat/native-runtime</Text>
      </StatusBarGroup>
    </StatusBar>
  ));

  const status = screen.getByRole("status", { name: "Workspace status" });
  expect(status.children[0]?.className).toContain("min-w-0");
  expect(status.children[0]?.className).toContain("flex-1");
  expect(status.children[0]?.attribute("grow")).toBeNull();
  expect(status.children[1]?.className).toContain("flex-none");
  expect(status.children[2]?.className).toContain("shrink");
  expect(status.children[2]?.attribute("shrink")).toBeNull();
  expect(status.text).toContain("Ready");
});
