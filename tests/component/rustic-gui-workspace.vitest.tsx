import type { Dialog } from "@wabou/core";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { Button, Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import type { ProfileStore } from "../../apps/rustic-gui/ui/profile-store";
import { FileDetails } from "../../apps/rustic-gui/ui/file-details";
import { SnapshotFileTree } from "../../apps/rustic-gui/ui/snapshot-tree";
import { formatModified } from "../../apps/rustic-gui/ui/snapshots";
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

test("snapshot timestamps stay compact in the table", () => {
  expect(formatModified("2026-09-02T04:18:35.321355Z")).toBe(
    "2026-09-02 04:18",
  );
  expect(formatModified(undefined)).toBe("—");
});

test("snapshot file tree loads child directories only when expanded", async () => {
  const selected = vi.fn();
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 3,
      listFiles: async (request: { path: string }) =>
        request.path === "docs"
          ? [
              {
                name: "guide.md",
                path: "docs/guide.md",
                kind: "file" as const,
                size: 8,
              },
            ]
          : [
              {
                name: "docs",
                path: "docs",
                kind: "directory" as const,
                size: 0,
              },
              {
                name: "README.md",
                path: "README.md",
                kind: "file" as const,
                size: 12,
              },
            ],
    },
  });
  const screen = renderComponent(
    () => (
      <SnapshotFileTree
        profileId="profile"
        snapshotId="snapshot"
        onSelect={selected}
      />
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("treeitem", { name: "docs" })).toBeDefined();
    expect(screen.getByRole("treeitem", { name: "README.md" })).toBeDefined();
  });
  expect(fixture.callsTo("rustic.listFiles")).toHaveLength(1);

  screen.getByRole("treeitem", { name: "docs" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("treeitem", { name: "guide.md" })).toBeDefined();
  });
  expect(fixture.callsTo("rustic.listFiles")).toHaveLength(2);
  expect(fixture.callsTo("rustic.listFiles")[1]?.args[0]).toEqual({
    profileId: "profile",
    snapshotId: "snapshot",
    path: "docs",
  });
  expect(selected).toHaveBeenCalledWith(expect.objectContaining({ path: "docs" }));
});

test("file details preview and extract through the native rustic capability", async () => {
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 3,
      previewPath: async () => ({
        destination: "/tmp/wabou-rustic-preview/42",
        plan: {
          restoreSize: 18,
          matchedSize: 0,
          filesToRestore: 1,
          filesToModify: 0,
          filesUnchanged: 0,
          directoriesToRestore: 0,
          directoriesToModify: 0,
        },
      }),
      previewRestore: async () => ({
        restoreSize: 18,
        matchedSize: 0,
        filesToRestore: 1,
        filesToModify: 0,
        filesUnchanged: 0,
        directoriesToRestore: 0,
        directoriesToModify: 0,
      }),
      restorePath: async () => ({
        destination: "/tmp/export/settings.toml",
        plan: {
          restoreSize: 18,
          matchedSize: 0,
          filesToRestore: 1,
          filesToModify: 0,
          filesUnchanged: 0,
          directoriesToRestore: 0,
          directoriesToModify: 0,
        },
      }),
      openPath: async () => {},
    },
  });
  const screen = renderComponent(
    () => (
      <FileDetails
        profileId="profile"
        snapshotId="snapshot"
        entry={{
          name: "settings.toml",
          path: "home/me/settings.toml",
          kind: "file",
          size: 18,
          modified: "2026-09-02T04:18:35Z",
        }}
      />
    ),
    { host: fixture.host, platform: { dialog } },
  );

  screen.getByRole("button", { name: "Preview temporary copy" }).click();
  await screen.waitFor(() => {
    expect(screen.roots[0]?.text).toContain("/tmp/wabou-rustic-preview/42");
  });

  screen.getByRole("button", { name: "Extract…" }).click();
  screen
    .getByRole("textbox", { name: "Extraction destination" })
    .input("/tmp/export");
  screen.getByRole("button", { name: "Review extraction" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("button", { name: "Extract" })).toBeDefined();
  });
  screen.getByRole("button", { name: "Extract" }).click();
  await screen.waitFor(() => {
    expect(fixture.callsTo("rustic.restorePath")).toHaveLength(1);
  });

  expect(fixture.callsTo("rustic.previewPath")).toHaveLength(1);
  expect(fixture.callsTo("rustic.openPath")).toHaveLength(1);
  expect(fixture.callsTo("rustic.previewRestore")).toHaveLength(1);
  expect(fixture.callsTo("rustic.restorePath")[0]?.args[0]).toEqual({
    profileId: "profile",
    snapshotId: "snapshot",
    path: "home/me/settings.toml",
    destination: "/tmp/export",
  });
});

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

test("rustic session hydrates durable profiles and exposes their locked state", async () => {
  const store: ProfileStore = {
    load: async () => ({
      profiles: [
        {
          id: "photos",
          name: "Photos",
          repositoryPath: "/data/repository",
          sources: ["/data/photos"],
        },
      ],
      activeProfileId: "photos",
    }),
    save: async () => {},
    setActive: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 3,
      status: async () => ({
        unlockedProfileIds: [],
      }),
    },
  });
  const Status = () => {
    const session = useRusticSession();
    return <Text role="status">{session.pendingUnlock()?.name ?? "none"}</Text>;
  };
  const screen = renderComponent(
    () => (
      <RusticSessionProvider store={store}>
        <Status />
      </RusticSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Photos");
  });
  expect(fixture.callsTo("rustic.status")).toHaveLength(1);
});

test("creating a profile unlocks Rust before persisting credential-free metadata", async () => {
  const save = vi.fn<ProfileStore["save"]>(async () => {});
  const store: ProfileStore = {
    load: async () => ({ profiles: [] }),
    save,
    setActive: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 3,
      status: async () => ({ unlockedProfileIds: [] }),
      createProfile: async (request: { id: string }) => ({
        unlockedProfileIds: [request.id],
        activeProfileId: request.id,
      }),
    },
  });
  const Create = () => {
    const session = useRusticSession();
    return (
      <>
        <Button
          aria-label="Create Photos backup"
          onClick={() =>
            void session.connectProfile("create", {
              name: "Photos",
              repositoryPath: "/data/backups/photos",
              password: "wabou-rustic-test",
              sources: ["/data/photos"],
            })
          }
        />
        <Text role="status">{session.activeProfile()?.name ?? "none"}</Text>
      </>
    );
  };
  const screen = renderComponent(
    () => (
      <RusticSessionProvider store={store}>
        <Create />
      </RusticSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("none");
  });
  screen.getByRole("button", { name: "Create Photos backup" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Photos");
  });
  expect(fixture.callsTo("rustic.createProfile")).toHaveLength(1);
  expect(save).toHaveBeenCalledWith(
    expect.objectContaining({
      name: "Photos",
      repositoryPath: "/data/backups/photos",
      sources: ["/data/photos"],
    }),
  );
  expect(JSON.stringify(save.mock.calls)).not.toContain("wabou-rustic-test");
});

test("rustic sidebar exposes stable navigation and repository status", () => {
  const selectProfile = vi.fn<(profileId: string) => void>();
  const create = vi.fn<() => void>();
  const screen = renderComponent(() => (
    <RusticSidebar
      active="photos"
      profiles={[
        {
          id: "photos",
          name: "Photos",
          repositoryPath: "/data/backups/rustic",
          sources: ["/data/photos"],
        },
      ]}
      unlockedProfileIds={["photos"]}
      onCreate={create}
      onSelectProfile={selectProfile}
    />
  ));

  expect(screen.getByRole("button", { name: "Photos" }).selected).toBe(true);
  expect(screen.roots[0]?.text).toContain("Photos");

  screen.getByRole("button", { name: "Photos" }).click();
  expect(selectProfile).toHaveBeenCalledWith("photos");
  screen.getByRole("button", { name: "New backup" }).click();
  expect(create).toHaveBeenCalledOnce();
});
