import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { GithubDesktopReference } from "../../apps/gallery/ui/pages/github-desktop";

test("coordinates changed-file selection with the visible diff", () => {
  const screen = renderComponent(() => <GithubDesktopReference />);

  expect(
    screen.getByRole("region", {
      name: "Selected file diff: github-desktop.tsx",
    }),
  ).not.toBeNull();

  screen.getByRole("button", { name: "Open layout-fixture-pages.tsx" }).click();

  expect(
    screen.getByRole("region", {
      name: "Selected file diff: layout-fixture-pages.tsx",
    }),
  ).not.toBeNull();
});

test("requires a summary and at least one included file before commit", () => {
  const screen = renderComponent(() => <GithubDesktopReference />);

  expect(
    screen.getByRole("button", {
      name: "Commit to feat/electron-reference",
    }).disabled,
  ).toBe(false);

  screen.getByRole("checkbox", { name: "Include all changed files" }).click();
  expect(
    screen.getByRole("button", {
      name: "Commit to feat/electron-reference",
    }).disabled,
  ).toBe(true);

  screen.getByRole("checkbox", { name: "Include all changed files" }).click();
  screen.getByRole("textbox", { name: "Commit summary" }).input("");
  expect(
    screen.getByRole("button", {
      name: "Commit to feat/electron-reference",
    }).disabled,
  ).toBe(true);
});

test("exposes Changes and History as one selected tab at a time", () => {
  const screen = renderComponent(() => <GithubDesktopReference />);
  const changes = screen.getByRole("tab", { name: "Changes (3)" });
  const history = screen.getByRole("tab", { name: "History" });

  expect(changes.selected).toBe(true);
  expect(history.selected).toBe(false);
  history.click();
  expect(changes.selected).toBe(false);
  expect(history.selected).toBe(true);
  expect(
    screen.getByRole("tablist", { name: "Repository activity" }),
  ).not.toBeNull();
});
