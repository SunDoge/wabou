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
    expect(screen.getByRole("button", { name: "src/index.ts" })).toBeDefined(),
  );
  expect(loadFiles).toHaveBeenCalledTimes(1);
  screen
    .getByRole("textbox", { name: "Search workspace files" })
    .input("index");
  expect(screen.queryByRole("button", { name: "README.md" })).toBeNull();

  screen.getByRole("button", { name: "src/index.ts" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Add to context" }),
    ).toBeDefined(),
  );
  expect(readFile).toHaveBeenCalledWith("/work/project", "src/index.ts");
  screen.getByRole("button", { name: "Add to context" }).click();
  expect(addContext).toHaveBeenCalledWith("src/index.ts");
});
