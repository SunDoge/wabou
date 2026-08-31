import { renderComponent } from "@wabou/test/component";
import { Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  ComposerContextFiles,
  WorkspaceContextPicker,
} from "../../apps/pi-agent/ui/composer-context";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((accept) => {
    resolve = accept;
  });
  return { promise, resolve };
}

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

test("discards a stale file list when the active workspace changes", async () => {
  const first = deferred<readonly string[]>();
  const second = deferred<readonly string[]>();
  const loadFiles = vi.fn((cwd: string) =>
    cwd === "/first" ? first.promise : second.promise,
  );
  let changeWorkspace!: (cwd: string) => void;
  const App = () => {
    const [cwd, setCwd] = createSignal("/first");
    changeWorkspace = setCwd;
    return (
      <WorkspaceContextPicker
        cwd={cwd()}
        paths={[]}
        change={() => {}}
        loadFiles={loadFiles}
      />
    );
  };
  const screen = renderComponent(App);

  screen.getByRole("button", { name: "Add context file" }).click();
  await screen.waitFor(() => expect(loadFiles).toHaveBeenCalledWith("/first"));
  changeWorkspace("/second");
  screen.flush();
  await screen.waitFor(() => expect(loadFiles).toHaveBeenCalledWith("/second"));

  second.resolve(["second.txt"]);
  first.resolve(["stale-first.txt"]);
  await screen.waitFor(() =>
    expect(screen.getByRole("option", { name: "second.txt" })).not.toBeNull(),
  );
  expect(screen.queryByRole("option", { name: "stale-first.txt" })).toBeNull();
});

test("keeps workspace file picker failures visible and retryable", async () => {
  let attempt = 0;
  const screen = renderComponent(() => (
    <WorkspaceContextPicker
      cwd="/workspace"
      paths={[]}
      change={() => {}}
      loadFiles={async () => {
        attempt += 1;
        if (attempt === 1) throw new Error("index unavailable");
        return ["README.md"];
      }}
    />
  ));

  screen.getByRole("button", { name: "Add context file" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("alert", { name: "Could not load workspace files" })
        .text,
    ).toContain("index unavailable"),
  );
  screen.getByRole("button", { name: "Try again" }).click();
  await screen.waitFor(() =>
    expect(screen.getByRole("option", { name: "README.md" })).toBeDefined(),
  );
  expect(attempt).toBe(2);
});
