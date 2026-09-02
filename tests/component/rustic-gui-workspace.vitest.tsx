import type { Dialog } from "@wabou/core";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  RusticSessionProvider,
  useRusticSession,
} from "../../apps/rustic-gui/ui/session";
import { RusticSidebar } from "../../apps/rustic-gui/ui/shell";
import { BackupSourcesPanel } from "../../apps/rustic-gui/ui/workspace-components";

const dialog: Dialog = {
  open: async () => null,
  save: async () => null,
  pickDirectory: async () => null,
  message: async () => "ok",
};

test("backup sources add normalized unique paths and remove existing paths", () => {
  const changes = vi.fn<(sources: string[]) => void>();
  const App = () => {
    const [sources, setSources] = createSignal<string[]>(["/data/photos"]);
    return (
      <BackupSourcesPanel
        sources={sources()}
        onChange={(next) => {
          changes(next);
          setSources(next);
        }}
      />
    );
  };
  const screen = renderComponent(App, { platform: { dialog } });
  const input = screen.getByRole("textbox", { name: "Backup folder" });

  input.input("  /data/documents  ");
  screen.getByRole("button", { name: "Add folder" }).click();
  expect(changes).toHaveBeenLastCalledWith(["/data/photos", "/data/documents"]);

  screen.getByRole("button", { name: "Remove /data/photos" }).click();
  expect(changes).toHaveBeenLastCalledWith(["/data/documents"]);
});

test("backup sources disable every mutating action during a backup", () => {
  const screen = renderComponent(
    () => (
      <BackupSourcesPanel
        sources={["/data/photos"]}
        disabled
        onChange={() => {}}
      />
    ),
    { platform: { dialog } },
  );

  expect(
    screen.getByRole("button", { name: "Choose backup folder" }).disabled,
  ).toBe(true);
  expect(screen.getByRole("button", { name: "Add folder" }).disabled).toBe(
    true,
  );
  expect(
    screen.getByRole("button", { name: "Remove /data/photos" }).disabled,
  ).toBe(true);
});

test("rustic session boots its Solid 2 effect and publishes host status", async () => {
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 1,
      status: async () => ({
        connected: true,
        repositoryPath: "/data/repository",
        sources: ["/data/photos"],
      }),
    },
  });
  const Status = () => {
    const session = useRusticSession();
    return (
      <Text role="status">{session.status().repositoryPath ?? "none"}</Text>
    );
  };
  const screen = renderComponent(
    () => (
      <RusticSessionProvider>
        <Status />
      </RusticSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("/data/repository");
  });
  expect(fixture.callsTo("rustic.status")).toHaveLength(1);
});

test("rustic sidebar exposes stable navigation and repository status", () => {
  const navigate = vi.fn<(to: "/" | "/snapshots") => void>();
  const screen = renderComponent(() => (
    <RusticSidebar
      active="/snapshots"
      connected
      repositoryPath="/data/backups/rustic"
      onNavigate={navigate}
    />
  ));

  expect(screen.getByRole("button", { name: "Snapshots" }).selected).toBe(true);
  expect(screen.getByRole("status", { name: "Repository open" })).toBeTruthy();
  expect(screen.roots[0]?.text).toContain("/data/backups/rustic");

  screen.getByRole("button", { name: "Repository" }).click();
  expect(navigate).toHaveBeenCalledWith("/");
});
