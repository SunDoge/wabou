import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { LauncherSurface } from "../../apps/launcher/ui/launcher";
import { rankLauncherEntries } from "../../apps/launcher/ui/ranking";

const entries = [
  {
    id: "code",
    label: "Visual Studio Code",
    description: "Code editor",
    keywords: ["development", "typescript"],
  },
  {
    id: "browser",
    label: "Firefox",
    description: "Web browser",
    keywords: ["internet", "mozilla"],
  },
  {
    id: "notes",
    label: "Paper",
    description: "Notes",
    keywords: ["write", "text editor"],
  },
];

test("ranks label prefixes before metadata matches", () => {
  expect(rankLauncherEntries(entries, "fire").map((entry) => entry.id)).toEqual(
    ["browser"],
  );
  expect(
    rankLauncherEntries(entries, "editor").map((entry) => entry.id),
  ).toEqual(["notes", "code"]);
});

test("keeps the launcher keyboard-first and dismissible", () => {
  const launch = vi.fn();
  const dismiss = vi.fn();
  const screen = renderComponent(() => (
    <LauncherSurface entries={entries} onLaunch={launch} onDismiss={dismiss} />
  ));
  const input = screen.getByRole("textbox", { name: "Search applications" });
  input.input("fire");
  expect(screen.getByRole("option", { name: "Firefox" }).selected).toBe(true);
  input.press("Enter");
  expect(launch).toHaveBeenCalledWith("browser");
  input.press("Escape");
  expect(dismiss).toHaveBeenCalledOnce();
});

test("reports loading, empty, and failure states without losing the search field", () => {
  const loading = renderComponent(() => (
    <LauncherSurface
      entries={[]}
      loading
      onLaunch={() => {}}
      onDismiss={() => {}}
    />
  ));
  expect(loading.getByRole("status").text).toContain("Indexing");
  expect(
    loading.getByRole("textbox", { name: "Search applications" }),
  ).toBeTruthy();
  loading.dispose();

  const retry = vi.fn();
  const failed = renderComponent(() => (
    <LauncherSurface
      entries={[]}
      error={new Error("index unavailable")}
      onLaunch={() => {}}
      onDismiss={() => {}}
      onRetry={retry}
    />
  ));
  failed.getByRole("button", { name: "Try again" }).click();
  expect(retry).toHaveBeenCalledOnce();
});
