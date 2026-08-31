import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { WorkspacePanel } from "../../apps/pi-agent/ui/workspace-panel";

test("workspace panel filters, previews, and attaches a file", async () => {
  const addContext = vi.fn();
  const loadFiles = vi.fn(async () => ["README.md", "src/index.ts"]);
  const readFile = vi.fn(async (_cwd: string, path: string) => ({
    path,
    text: path.endsWith(".md") ? "# Guide\n\nHello" : "export const value = 1;",
  }));
  const screen = renderComponent(() => (
    <WorkspacePanel
      cwd="/work/project"
      loadFiles={loadFiles}
      readFile={readFile}
      addContext={addContext}
      close={() => {}}
    />
  ));

  await screen.waitFor(() =>
    expect(screen.getByRole("option", { name: "src/index.ts" })).toBeDefined(),
  );
  expect(loadFiles).toHaveBeenCalledTimes(1);
  screen
    .getByRole("textbox", { name: "Search workspace files" })
    .input("index");
  expect(screen.queryByRole("option", { name: "README.md" })).toBeNull();

  screen.getByRole("listbox", { name: "Workspace files" }).press("Enter");
  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Add to context" }),
    ).toBeDefined(),
  );
  expect(readFile).toHaveBeenCalledWith("/work/project", "src/index.ts");
  screen.getByRole("button", { name: "Add to context" }).click();
  expect(addContext).toHaveBeenCalledWith("src/index.ts");
});

test("workspace preview exposes mutually exclusive empty, loading, and error states", async () => {
  let rejectPreview: ((reason: Error) => void) | undefined;
  const preview = new Promise<never>((_resolve, reject) => {
    rejectPreview = reject;
  });
  const screen = renderComponent(() => (
    <WorkspacePanel
      cwd="/work/project"
      loadFiles={async () => ["src/index.ts"]}
      readFile={() => preview}
      addContext={() => {}}
      close={() => {}}
    />
  ));

  await screen.waitFor(() =>
    expect(screen.getByRole("option", { name: "src/index.ts" })).toBeDefined(),
  );
  expect(JSON.stringify(screen.snapshot())).toContain(
    "Select a text file to preview it.",
  );

  screen.getByRole("option", { name: "src/index.ts" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("status", { name: "Loading file preview…" }),
    ).toBeDefined(),
  );
  expect(JSON.stringify(screen.snapshot())).not.toContain(
    "Select a text file to preview it.",
  );

  rejectPreview?.(new Error("permission denied"));
  await screen.waitFor(() =>
    expect(
      screen.getByRole("alert", { name: "Could not load the file preview" })
        .text,
    ).toContain("permission denied"),
  );
  expect(
    screen.queryByRole("status", { name: "Loading file preview…" }),
  ).toBeNull();
});

test("workspace file loading failure is retryable and does not masquerade as a preview error", async () => {
  let attempt = 0;
  const screen = renderComponent(() => (
    <WorkspacePanel
      cwd="/work/project"
      loadFiles={async () => {
        attempt += 1;
        if (attempt === 1) throw new Error("repository unavailable");
        return ["src/index.ts"];
      }}
      readFile={async (_cwd, path) => ({ path, text: "content" })}
      addContext={() => {}}
      close={() => {}}
    />
  ));

  await screen.waitFor(() =>
    expect(
      screen.getByRole("alert", { name: "Could not load workspace files" })
        .text,
    ).toContain("repository unavailable"),
  );
  expect(
    screen.queryByRole("alert", { name: "Could not load the file preview" }),
  ).toBeNull();
  screen.getByRole("button", { name: "Try again" }).click();
  await screen.waitFor(() =>
    expect(screen.getByRole("option", { name: "src/index.ts" })).toBeDefined(),
  );
  expect(attempt).toBe(2);
});

test("workspace preview is latest-wins and a failed preview can retry", async () => {
  let resolveFirst:
    | ((value: { path: string; text: string }) => void)
    | undefined;
  let attempts = 0;
  const screen = renderComponent(() => (
    <WorkspacePanel
      cwd="/work/project"
      loadFiles={async () => ["first.md", "second.md"]}
      readFile={async (_cwd, path) => {
        if (path === "first.md") {
          return new Promise((resolve) => {
            resolveFirst = resolve;
          });
        }
        attempts += 1;
        if (attempts === 1) throw new Error("preview temporarily unavailable");
        return { path, text: "Second preview" };
      }}
      addContext={() => {}}
      close={() => {}}
    />
  ));

  await screen.waitFor(() =>
    expect(screen.getByRole("option", { name: "first.md" })).toBeDefined(),
  );
  screen.getByRole("option", { name: "first.md" }).click();
  screen.getByRole("option", { name: "second.md" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("alert", { name: "Could not load the file preview" })
        .text,
    ).toContain("temporarily unavailable"),
  );

  resolveFirst?.({ path: "first.md", text: "Stale preview" });
  await Promise.resolve();
  expect(JSON.stringify(screen.snapshot())).not.toContain("Stale preview");

  screen.getByRole("button", { name: "Try again" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Add to context" }),
    ).toBeDefined(),
  );
  expect(JSON.stringify(screen.snapshot())).toContain("Second preview");
  expect(attempts).toBe(2);
});
