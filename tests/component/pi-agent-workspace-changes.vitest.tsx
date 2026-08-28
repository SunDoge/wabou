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
  expect(screen.getByRole("region", { name: "Technical diff" }).text).toContain(
    "1 file changed",
  );
  expect(
    screen.queryByRole("textbox", { name: "Technical diff: src/main.rs" }),
  ).toBeNull();
});
