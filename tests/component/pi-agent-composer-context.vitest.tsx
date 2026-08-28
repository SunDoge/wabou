import { renderComponent } from "@wabou/test/component";
import { Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  ComposerContextFiles,
  WorkspaceContextPicker,
} from "../../apps/pi-agent/ui/composer-context";

test("searches workspace files and keeps selected context explicit", async () => {
  const loadFiles = vi.fn(async () => [
    "README.md",
    "src/main.rs",
    "src/service.rs",
  ]);
  const App = () => {
    const [paths, setPaths] = createSignal<readonly string[]>([]);
    return (
      <>
        <ComposerContextFiles paths={paths()} change={setPaths} />
        <WorkspaceContextPicker
          cwd="/workspace"
          paths={paths()}
          change={setPaths}
          loadFiles={loadFiles}
        />
        <Text role="status" aria-label="Selected context">
          {paths().join("|")}
        </Text>
      </>
    );
  };
  const screen = renderComponent(App);

  screen.getByRole("button", { name: "Add context file" }).click();
  await screen.waitFor(() =>
    expect(loadFiles).toHaveBeenCalledWith("/workspace"),
  );
  screen
    .getByRole("textbox", { name: "Search workspace files" })
    .input("service");
  screen.getByRole("option", { name: "src/service.rs" }).click();

  expect(screen.getByRole("status", { name: "Selected context" }).text).toBe(
    "src/service.rs",
  );
  expect(screen.getByRole("group", { name: "Context files" }).text).toContain(
    "src/service.rsWorkspace context",
  );
  screen.getByRole("button", { name: "Remove src/service.rs" }).click();
  expect(screen.queryByRole("group", { name: "Context files" })).toBeNull();
});
