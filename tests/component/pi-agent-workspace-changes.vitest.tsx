import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { WorkspaceChangesPanel } from "../../apps/pi-agent/ui/workspace-changes-panel";

test("Pi workspace changes load as a collapsed localized summary", async () => {
  const load = vi.fn(async () => ({
    files: [
      {
        path: "src/main.rs",
        status: "modified" as const,
        additions: 3,
        deletions: 1,
        patch: "@@ -1 +1 @@\n-old\n+new",
      },
    ],
  }));
  const screen = renderComponent(() => (
    <WorkspaceChangesPanel cwd="/work/wabou" load={load} close={() => {}} />
  ));

  await screen.waitFor(() =>
    expect(screen.getByRole("button", { name: "src/main.rs" })).toBeDefined(),
  );
  expect(load).toHaveBeenCalledWith("/work/wabou");
  expect(load).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("region", { name: "Technical diff" }).text).toContain(
    "1 file changed",
  );
  expect(
    screen.queryByRole("textbox", { name: "Technical diff: src/main.rs" }),
  ).toBeNull();
});

test("Pi workspace changes expose a retryable inspector error", async () => {
  let attempts = 0;
  const load = vi.fn(async () => {
    attempts += 1;
    if (attempts === 1) throw new Error("repository unavailable");
    return { files: [] };
  });
  const screen = renderComponent(() => (
    <WorkspaceChangesPanel cwd="/work/wabou" load={load} close={() => {}} />
  ));

  await screen.waitFor(() =>
    expect(
      screen.getByRole("alert", { name: "Could not load code changes" }).text,
    ).toContain("repository unavailable"),
  );
  screen.getByRole("button", { name: "Try again" }).click();
  await screen.waitFor(() => expect(load).toHaveBeenCalledTimes(2));
  await screen.waitFor(() =>
    expect(JSON.stringify(screen.snapshot())).toContain(
      "No code changes in this project.",
    ),
  );
});
