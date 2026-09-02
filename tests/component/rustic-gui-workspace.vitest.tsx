import type { Dialog } from "@wabou/core";
import { Text } from "@wabou/ui";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import { BackupSourcesPanel } from "../../apps/rustic-gui/ui/workspace-components";
import {
  RusticSessionProvider,
  useRusticSession,
} from "../../apps/rustic-gui/ui/session";

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
