import type { Dialog } from "@wabou/core";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { Button, Text } from "@wabou/ui";
import { createSignal, Show } from "solid-js";
import { expect, test, vi } from "vitest";
import type { FileEntry } from "../../apps/timestow/ui/api";
import { FileDetails } from "../../apps/timestow/ui/file-details";
import type { ProfileStore } from "../../apps/timestow/ui/profile-store";
import { BackupScheduleDialog } from "../../apps/timestow/ui/schedule-dialog";
import {
  TimestowSessionProvider,
  useTimestowSession,
} from "../../apps/timestow/ui/session";
import { BackupConnectionForm } from "../../apps/timestow/ui/setup";
import {
  SessionErrorBanner,
  TimestowSidebar,
} from "../../apps/timestow/ui/shell";
import { createSnapshotBrowserCache } from "../../apps/timestow/ui/snapshot-browser-cache";
import {
  formatSnapshotTime,
  SnapshotDetails,
} from "../../apps/timestow/ui/snapshot-details";
import { SnapshotDiffPanel } from "../../apps/timestow/ui/snapshot-diff";
import { SnapshotFileTree } from "../../apps/timestow/ui/snapshot-tree";
import {
  formatModified,
  SnapshotFileRow,
  SnapshotWorkspaceHeader,
  snapshotAfterRefresh,
} from "../../apps/timestow/ui/snapshots";
import { SortableTableHead } from "../../apps/timestow/ui/sortable-table-head";
import {
  BackupSourcesDialog,
  BackupSourcesPanel,
} from "../../apps/timestow/ui/workspace-components";

const dialog: Dialog = {
  open: async () => null,
  save: async () => null,
  pickDirectory: async () => null,
  message: async () => "ok",
};

test("session errors remain visible and dismissible outside a page", () => {
  const dismiss = vi.fn();
  const screen = renderComponent(() => (
    <SessionErrorBanner
      message="The scheduled backup could not open its repository."
      onDismiss={dismiss}
    />
  ));

  expect(screen.getByRole("alert", { name: "Timestow error" }).text).toContain(
    "The scheduled backup could not open its repository.",
  );
  screen.getByRole("button", { name: "Dismiss Timestow error" }).click();
  expect(dismiss).toHaveBeenCalledTimes(1);
});

test("repository setup chooses a mode before exposing one primary action", () => {
  const submit = vi.fn();
  const App = () => {
    const [mode, setMode] = createSignal<"create" | "open">("create");
    return (
      <BackupConnectionForm
        mode={mode()}
        name="Photos"
        path="/data/backups/photos"
        passwordSecret="timestow:test"
        onModeChange={setMode}
        onNameChange={() => {}}
        onPathChange={() => {}}
        onSubmit={submit}
      />
    );
  };
  const screen = renderComponent(App, { platform: { dialog } });

  expect(screen.getAllByRole("button", { name: "Create backup" })).toHaveLength(
    1,
  );
  expect(screen.queryByRole("button", { name: "Open repository" })).toBeNull();
  const password = screen.getByRole("textbox", {
    name: "Repository password",
  });
  expect(password.tag).toBe("password-input");
  expect(password.attribute("secret")).toBe("timestow:test");
  expect(password.value).toBeNull();

  screen.getByRole("button", { name: "Open an existing repository" }).click();
  screen.flush();

  expect(screen.queryByRole("button", { name: "Create backup" })).toBeNull();
  screen.getByRole("button", { name: "Open repository" }).click();
  expect(submit).toHaveBeenCalledTimes(1);
});

test("backup workspace keeps configuration and primary actions distinct", () => {
  const refresh = vi.fn();
  const backup = vi.fn();
  const screen = renderComponent(
    () => (
      <SnapshotWorkspaceHeader
        name="Home archive"
        repositoryPath="/data/backups/home"
        sources={["/data/photos"]}
        backingUp={false}
        scheduleControl={<Button aria-label="Schedule backup" />}
        onSourcesChange={() => {}}
        onRefresh={refresh}
        onBackup={backup}
      />
    ),
    { platform: { dialog } },
  );

  expect(
    screen.getByRole("toolbar", { name: "Backup workspace actions" }),
  ).toBeDefined();
  screen.getByRole("button", { name: "Refresh snapshots" }).click();
  screen.getByRole("button", { name: "Back up now" }).click();
  expect(refresh).toHaveBeenCalledTimes(1);
  expect(backup).toHaveBeenCalledTimes(1);
});

test("snapshot timestamps stay compact in the table", () => {
  expect(formatModified("2026-09-02T04:18:35.321355Z")).toBe(
    "2026-09-02 04:18",
  );
  expect(formatModified(undefined)).toBe("—");
});

test("snapshot refresh never carries a selection into an empty profile", () => {
  const current = {
    id: "current",
    time: "2026-09-08T00:42:00Z",
    hostname: "workstation",
    paths: ["/data/photos"],
    filesNew: 1,
    filesChanged: 0,
    label: "Current",
    tags: [],
    deleteProtected: false,
  };

  expect(snapshotAfterRefresh([], current.id, true)).toBeUndefined();
  expect(snapshotAfterRefresh([], current.id, false)).toBeUndefined();
  expect(snapshotAfterRefresh([current], undefined, true)).toBe(current);
  expect(snapshotAfterRefresh([current], current.id, false)).toBe(current);
});

test("sortable table headers use a quiet readable surface", () => {
  const [direction, setDirection] = createSignal<"asc" | "desc">("asc");
  const screen = renderComponent(() => (
    <SortableTableHead
      label="Name"
      direction={direction}
      onToggle={() => setDirection("desc")}
    />
  ));
  const sorted = screen.getByRole("columnheader", {
    name: "Name, sorted ascending",
  });
  expect(sorted.text).toBe("Name");
  sorted.click();
  expect(
    screen.getByRole("columnheader", { name: "Name, sorted descending" }).text,
  ).toBe("Name");
  const header = screen.getByRole("columnheader", {
    name: "Name, sorted descending",
  });

  expect(header.className).toContain("bg-transparent");
  expect(header.className).toContain("text-secondary");
  expect(header.className).not.toContain("bg-accent");
  header.hover();
  expect(header.className).toContain("bg-control-hover");
  expect(header.className).toContain("text-primary");
});

test("snapshot summaries stay compact while details preserve full metadata", () => {
  const snapshot = {
    id: "f21dc6d86a8b42b4aaf81ff39aa15c8f",
    time: "2026-09-02T04:18:35.321355Z",
    hostname: "workstation",
    paths: ["/data/photos", "/data/documents"],
    filesNew: 42,
    filesChanged: 7,
    label: "Nightly",
    description: "Before migration",
    tags: ["archive", "photos"],
    deleteProtected: true,
  };

  expect(formatSnapshotTime(snapshot.time)).toBe("2026-09-02 04:18");
  const screen = renderComponent(() => <SnapshotDetails snapshot={snapshot} />);
  screen.getByRole("button", { name: "Details" }).click();
  screen.flush();

  const details = screen.getByRole("dialog", { name: "Snapshot details" });
  expect(details.text).toContain(snapshot.time);
  expect(details.text).toContain("workstation");
  expect(details.text).toContain("/data/photos");
  expect(details.text).toContain(snapshot.id);
});

test("snapshot details edit label, tags, description, and deletion protection", async () => {
  const save = vi.fn(async () => {});
  const snapshot = {
    id: "snapshot-editable",
    time: "2026-09-02T04:18:35Z",
    hostname: "workstation",
    paths: ["/data/photos"],
    filesNew: 12,
    filesChanged: 3,
    label: "Nightly",
    description: "Before cleanup",
    tags: ["photos"],
    deleteProtected: false,
  };
  const screen = renderComponent(() => (
    <SnapshotDetails snapshot={snapshot} onSave={save} />
  ));

  screen.getByRole("button", { name: "Details" }).click();
  screen
    .getByRole("textbox", { name: "Snapshot label" })
    .input("Before migration");
  screen
    .getByRole("textbox", { name: "Snapshot tags" })
    .input("archive, photos, archive");
  screen
    .getByRole("textbox", { name: "Snapshot description" })
    .input("Verified before migration");
  screen
    .getByRole("checkbox", { name: "Protect this snapshot from deletion" })
    .click();
  screen.getByRole("button", { name: "Save changes" }).click();

  await screen.waitFor(() => {
    expect(save).toHaveBeenCalledWith({
      label: "Before migration",
      description: "Verified before migration",
      tags: ["archive", "photos"],
      deleteProtected: true,
    });
  });
});

test("snapshot deletion requires confirmation and explains pruning separately", async () => {
  const remove = vi.fn(async () => {});
  const snapshot = {
    id: "snapshot-removable",
    time: "2026-09-02T04:18:35Z",
    hostname: "workstation",
    paths: ["/data/photos"],
    filesNew: 12,
    filesChanged: 3,
    label: "Before cleanup",
    description: undefined,
    tags: [],
    deleteProtected: false,
  };
  const screen = renderComponent(() => (
    <SnapshotDetails snapshot={snapshot} onDelete={remove} />
  ));

  screen.getByRole("button", { name: "Details" }).click();
  screen.getByRole("button", { name: "Delete snapshot" }).click();
  const confirmation = screen.getByRole("alertdialog", {
    name: "Delete snapshot",
  });
  expect(confirmation.text).toContain("reclaimed separately");
  expect(confirmation.text).toContain("cannot be undone");
  screen.getByRole("button", { name: "Delete snapshot" }).click();

  await screen.waitFor(() => expect(remove).toHaveBeenCalledOnce());
  expect(
    screen.queryByRole("alertdialog", { name: "Delete snapshot" }),
  ).toBeNull();
});

test("protected snapshots expose deletion as unavailable", () => {
  const screen = renderComponent(() => (
    <SnapshotDetails
      snapshot={{
        id: "snapshot-protected",
        time: "2026-09-02T04:18:35Z",
        hostname: "workstation",
        paths: ["/data/photos"],
        filesNew: 0,
        filesChanged: 0,
        label: "Protected",
        tags: [],
        deleteProtected: true,
      }}
      onSave={() => {}}
      onDelete={() => {}}
    />
  ));

  screen.getByRole("button", { name: "Details" }).click();
  expect(
    screen.getByRole("button", {
      name: "Protected snapshot cannot be deleted",
    }).disabled,
  ).toBe(true);
  expect(
    screen.getByRole("dialog", { name: "Snapshot details" }).text,
  ).toContain("Turn off protection and save");
});

test("snapshot deletion failures remain actionable in the confirmation", async () => {
  const screen = renderComponent(() => (
    <SnapshotDetails
      snapshot={{
        id: "snapshot-failing",
        time: "2026-09-02T04:18:35Z",
        hostname: "workstation",
        paths: ["/data/photos"],
        filesNew: 0,
        filesChanged: 0,
        label: "Before cleanup",
        tags: [],
        deleteProtected: false,
      }}
      onDelete={async () => {
        throw new Error("repository is append-only");
      }}
    />
  ));

  screen.getByRole("button", { name: "Details" }).click();
  screen.getByRole("button", { name: "Delete snapshot" }).click();
  screen.getByRole("button", { name: "Delete snapshot" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("alert").text).toBe("repository is append-only");
  });
  expect(
    screen.getByRole("alertdialog", { name: "Delete snapshot" }),
  ).toBeDefined();
});

test("snapshot changes compare against the recorded parent and can include metadata", async () => {
  const current = {
    id: "current-snapshot",
    parentId: "parent-snapshot",
    time: "2026-09-02T04:18:35Z",
    hostname: "workstation",
    paths: ["/data/photos"],
    filesNew: 1,
    filesChanged: 1,
    label: "",
    description: undefined,
    tags: [],
    deleteProtected: false,
  };
  const parent = {
    ...current,
    id: "parent-snapshot",
    parentId: undefined,
    time: "2026-09-01T04:18:35Z",
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      diffSnapshots: async (request: { includeMetadata?: boolean }) => ({
        entries: [
          {
            name: "z-last.txt",
            path: "docs/z-last.txt",
            kind: "file" as const,
            change: "added" as const,
            currentSize: 18,
          },
          {
            name: request.includeMetadata ? "mode.txt" : "new.txt",
            path: request.includeMetadata ? "docs/mode.txt" : "docs/new.txt",
            kind: "file" as const,
            change: request.includeMetadata
              ? ("metadata" as const)
              : ("added" as const),
            currentSize: 12,
          },
        ],
        summary: {
          added: request.includeMetadata ? 0 : 1,
          removed: 0,
          modified: 0,
          metadata: request.includeMetadata ? 1 : 0,
          typeChanged: 0,
        },
        totalEntries: 10_000,
        truncated: true,
      }),
    },
  });
  const screen = renderComponent(
    () => (
      <SnapshotDiffPanel
        profileId="photos"
        snapshot={current}
        snapshots={[current, parent]}
      />
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("row", { name: "docs/new.txt" })).toBeDefined();
  });
  expect(
    screen.getByRole("label", { name: "Showing first 2 changes" }),
  ).toBeDefined();
  expect(fixture.callsTo("rustic.diffSnapshots")[0]?.args[0]).toEqual({
    profileId: "photos",
    snapshotId: "current-snapshot",
    baseSnapshotId: "parent-snapshot",
    includeMetadata: false,
    path: "",
    limit: 250,
  });

  expect(
    screen
      .getAllByRole("row")
      .map((row) => row.name)
      .filter((name) => name.startsWith("docs/")),
  ).toEqual(["docs/new.txt", "docs/z-last.txt"]);
  screen.getByRole("columnheader", { name: "Path, sorted ascending" }).click();
  screen.flush();
  expect(
    screen.getByRole("columnheader", { name: "Path, sorted descending" }),
  ).toBeDefined();
  expect(
    screen
      .getAllByRole("row")
      .map((row) => row.name)
      .filter((name) => name.startsWith("docs/")),
  ).toEqual(["docs/z-last.txt", "docs/new.txt"]);

  screen.getByRole("checkbox", { name: "Metadata changes" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("row", { name: "docs/mode.txt" })).toBeDefined();
  });
  expect(fixture.callsTo("rustic.diffSnapshots")).toHaveLength(2);
});

test("snapshot browser cache restores each snapshot path independently", () => {
  const cache = createSnapshotBrowserCache();
  const docs = [
    { name: "guide.md", path: "docs/guide.md", kind: "file" as const, size: 8 },
  ];

  cache.remember("snapshot-a", "docs", { entries: docs, total: 1 });
  cache.remember("snapshot-b", "photos", { entries: [], total: 0 });

  expect(cache.lastPath("snapshot-a")).toBe("docs");
  expect(cache.listing("snapshot-a", "docs")).toEqual({
    entries: docs,
    total: 1,
  });
  expect(cache.lastPath("snapshot-b")).toBe("photos");
  expect(cache.lastPath("snapshot-c")).toBe("");

  cache.clear();
  expect(cache.listing("snapshot-a", "docs")).toBeUndefined();
  expect(cache.lastPath("snapshot-a")).toBe("");
});

test("snapshot file tree loads child directories only when expanded", async () => {
  const selected = vi.fn();
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      listFiles: async (request: { path: string; offset?: number }) => {
        let entries: FileEntry[];
        if (request.path === "docs") {
          entries = [
            {
              name: "guide.md",
              path: "docs/guide.md",
              kind: "file" as const,
              size: 8,
            },
          ];
        } else if (request.offset) {
          entries = [
            {
              name: "LICENSE",
              path: "LICENSE",
              kind: "file" as const,
              size: 14,
            },
          ];
        } else {
          entries = [
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
          ];
        }
        const total = request.path === "docs" ? 1 : 3;
        return {
          entries,
          total,
          offset: request.offset ?? 0,
          hasMore: (request.offset ?? 0) + entries.length < total,
        };
      },
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
    offset: 0,
    limit: 250,
  });
  expect(selected).toHaveBeenCalledWith(
    expect.objectContaining({ path: "docs" }),
  );

  screen.getByRole("treeitem", { name: "Load more (1 remaining)" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("treeitem", { name: "LICENSE" })).toBeDefined();
  });
  expect(fixture.callsTo("rustic.listFiles")[2]?.args[0]).toEqual({
    profileId: "profile",
    snapshotId: "snapshot",
    path: "",
    offset: 2,
    limit: 250,
  });
});

test("file details preview and extract through the native rustic capability", async () => {
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
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

test("backup sources add manual paths with Enter and remove existing paths", () => {
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
  input.press("Enter");
  expect(changes).toHaveBeenLastCalledWith(["/data/photos", "/data/documents"]);

  screen.getByRole("button", { name: "Remove /data/photos" }).click();
  expect(changes).toHaveBeenLastCalledWith(["/data/documents"]);
});

test("backup sources add a directory as soon as the native picker commits it", async () => {
  const changes = vi.fn<(sources: string[]) => void>();
  const screen = renderComponent(
    () => <BackupSourcesPanel sources={[]} onChange={changes} />,
    {
      platform: {
        dialog: {
          ...dialog,
          pickDirectory: async () => "/data/photos",
        },
      },
    },
  );

  screen.getByRole("button", { name: "Choose backup folder" }).click();
  await screen.waitFor(() => {
    expect(changes).toHaveBeenCalledWith(["/data/photos"]);
  });
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
  expect(
    screen.getByRole("button", { name: "Remove /data/photos" }).disabled,
  ).toBe(true);
});

test("backup source configuration stays behind one explicit workspace action", () => {
  const screen = renderComponent(
    () => (
      <BackupSourcesDialog
        sources={["/data/photos", "/data/documents"]}
        onChange={() => {}}
      />
    ),
    { platform: { dialog } },
  );

  const manage = screen.getByRole("button", { name: "Manage backup folders" });
  expect(manage.text).toContain("2 folders");
  expect(screen.queryByRole("textbox", { name: "Backup folder" })).toBeNull();

  manage.click();
  expect(
    screen.getByRole("dialog", { name: "Manage backup folders" }),
  ).toBeDefined();
  expect(screen.getByRole("textbox", { name: "Backup folder" })).toBeDefined();
});

test("list rows give directory double click priority over single selection", async () => {
  const select = vi.fn();
  const open = vi.fn();
  const screen = renderComponent(
    () => (
      <SnapshotFileRow
        entry={{
          name: "docs",
          path: "docs",
          kind: "directory",
          size: 0,
        }}
        selected={false}
        searchActive={false}
        onSelect={select}
        onOpenDirectory={open}
      />
    ),
    { clock: "fake" },
  );
  const row = screen.getByRole("row", { name: "docs" });

  expect(screen.queryByRole("button", { name: "Open" })).toBeNull();

  row.click();
  expect(select).not.toHaveBeenCalled();
  await screen.advanceTime(410);
  expect(select).toHaveBeenCalledWith(
    expect.objectContaining({ path: "docs" }),
  );
  expect(open).not.toHaveBeenCalled();

  select.mockClear();
  row.click();
  row.click();
  row.emit("dblclick");
  expect(open).toHaveBeenCalledWith(expect.objectContaining({ path: "docs" }));
  await screen.advanceTime(410);
  expect(select).not.toHaveBeenCalled();
});

test("list rows expose low-frequency directory actions from a context menu", () => {
  const select = vi.fn();
  const open = vi.fn();
  const screen = renderComponent(() => (
    <SnapshotFileRow
      entry={{
        name: "docs",
        path: "docs",
        kind: "directory",
        size: 0,
      }}
      selected={false}
      searchActive={false}
      onSelect={select}
      onOpenDirectory={open}
    />
  ));

  screen.getByRole("row", { name: "docs" }).contextMenu();
  expect(select).toHaveBeenCalledWith(
    expect.objectContaining({ path: "docs" }),
  );
  screen.getByRole("menuitem", { name: "Open folder" }).click();
  expect(open).toHaveBeenCalledWith(expect.objectContaining({ path: "docs" }));
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
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      status: async () => ({
        unlockedProfileIds: [],
      }),
    },
  });
  const Status = () => {
    const session = useTimestowSession();
    return <Text role="status">{session.pendingUnlock()?.name ?? "none"}</Text>;
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <Status />
      </TimestowSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Photos");
  });
  expect(fixture.callsTo("rustic.status")).toHaveLength(1);
});

test("forgetting a backup clears native credentials before durable profile metadata", async () => {
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/repository",
    sources: ["/data/photos"],
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      status: async () => ({
        unlockedProfileIds: [profile.id],
        activeProfileId: profile.id,
      }),
      forgetProfile: async () => ({ unlockedProfileIds: [] }),
    },
  });
  const remove = vi.fn<ProfileStore["remove"]>(async () => {
    expect(fixture.callsTo("rustic.forgetProfile")).toHaveLength(1);
  });
  const store: ProfileStore = {
    load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
    save: async () => {},
    setActive: async () => {},
    remove,
  };
  const Controls = () => {
    const session = useTimestowSession();
    return (
      <>
        <Button
          aria-label="Forget Photos"
          onClick={() => void session.forgetProfile(profile.id)}
        />
        <Text role="status">
          {session.activeProfile()?.name ?? "none"} ·{" "}
          {session.profiles().length}
        </Text>
      </>
    );
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <Controls />
      </TimestowSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Photos · 1");
  });
  screen.getByRole("button", { name: "Forget Photos" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("none · 0");
  });
  expect(remove).toHaveBeenCalledWith(profile.id);
});

test("a failed native profile switch leaves the current profile selected", async () => {
  const setActive = vi.fn<ProfileStore["setActive"]>(async () => {});
  const profiles = [
    {
      id: "photos",
      name: "Photos",
      repositoryPath: "/data/photos-repository",
      sources: ["/data/photos"],
    },
    {
      id: "documents",
      name: "Documents",
      repositoryPath: "/data/documents-repository",
      sources: ["/data/documents"],
    },
  ];
  const store: ProfileStore = {
    load: async () => ({ profiles, activeProfileId: "photos" }),
    save: async () => {},
    setActive,
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      status: async () => ({
        unlockedProfileIds: profiles.map((profile) => profile.id),
        activeProfileId: "photos",
      }),
      selectProfile: async () => {
        throw new Error("native repository switch failed");
      },
    },
  });
  const Status = () => {
    const session = useTimestowSession();
    const [failure, setFailure] = createSignal("none");
    return (
      <>
        <Button
          aria-label="Select Documents"
          onClick={() =>
            void session
              .activateProfile("documents")
              .catch((cause) =>
                setFailure(
                  cause instanceof Error ? cause.message : String(cause),
                ),
              )
          }
        />
        <Text role="status">
          {session.activeProfile()?.name ?? "none"} · {failure()}
        </Text>
      </>
    );
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <Status />
      </TimestowSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Photos · none");
  });
  screen.getByRole("button", { name: "Select Documents" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe(
      "Photos · native repository switch failed",
    );
  });
  expect(setActive).not.toHaveBeenCalled();
});

test("creating a profile unlocks Rust before persisting credential-free metadata", async () => {
  const save = vi.fn<ProfileStore["save"]>(async () => {});
  const store: ProfileStore = {
    load: async () => ({ profiles: [] }),
    save,
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      status: async () => ({ unlockedProfileIds: [] }),
      createProfile: async (request: { id: string }) => ({
        unlockedProfileIds: [request.id],
        activeProfileId: request.id,
      }),
    },
  });
  const Create = () => {
    const session = useTimestowSession();
    return (
      <>
        <Button
          aria-label="Create Photos backup"
          onClick={() =>
            void session.connectProfile("create", {
              name: "Photos",
              repositoryPath: "/data/backups/photos",
              passwordSlot: "timestow:test",
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
      <TimestowSessionProvider store={store}>
        <Create />
      </TimestowSessionProvider>
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
  expect(fixture.callsTo("rustic.createProfile")[0]?.args[0]).toEqual(
    expect.objectContaining({
      passwordSlot: "timestow:test",
    }),
  );
  expect(
    fixture.callsTo("rustic.createProfile")[0]?.args[0],
  ).not.toHaveProperty("password");
  expect(save).toHaveBeenCalledWith(
    expect.objectContaining({
      name: "Photos",
      repositoryPath: "/data/backups/photos",
      sources: ["/data/photos"],
    }),
  );
  expect(JSON.stringify(save.mock.calls)).not.toContain("wabou-rustic-test");
});

test("runs a due profile backup in the background and records completion", async () => {
  const nextRunAt = new Date(Date.now() + 1_000).toISOString();
  const save = vi.fn<ProfileStore["save"]>(async () => {});
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/backups/photos",
    sources: ["/data/photos"],
    schedule: {
      enabled: true,
      intervalMinutes: 60 as const,
      nextRunAt,
    },
  };
  const store: ProfileStore = {
    load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
    save,
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      status: async () => ({
        unlockedProfileIds: [profile.id],
        activeProfileId: profile.id,
      }),
      runBackup: async () => ({
        snapshot: {
          id: "scheduled-snapshot",
          time: "2026-09-04T09:00:00.000Z",
          hostname: "workstation",
          paths: profile.sources,
          filesNew: 1,
          filesChanged: 0,
          label: "",
          tags: [],
          deleteProtected: false,
        },
      }),
    },
  });
  const Status = () => {
    const session = useTimestowSession();
    return (
      <Text role="status">
        {session.lastBackup()?.snapshot.id ?? "waiting"}
      </Text>
    );
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <Status />
      </TimestowSessionProvider>
    ),
    { host: fixture.host, clock: "fake" },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("waiting");
  });
  await screen.advanceTime(1_000);
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("scheduled-snapshot");
  });
  expect(fixture.callsTo("rustic.runBackup")).toHaveLength(1);
  expect(save).toHaveBeenCalledWith(
    expect.objectContaining({
      id: "photos",
      schedule: expect.objectContaining({
        enabled: true,
        lastRunAt: expect.any(String),
        nextRunAt: expect.any(String),
      }),
    }),
    { activate: false },
  );
});

test("schedule dialog explains the runtime boundary and exposes its controls", async () => {
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/backups/photos",
    sources: ["/data/photos"],
  };
  const save = vi.fn<ProfileStore["save"]>(async () => {});
  const store: ProfileStore = {
    load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
    save,
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 9,
      status: async () => ({
        unlockedProfileIds: [profile.id],
        activeProfileId: profile.id,
      }),
    },
  });
  const Schedule = () => {
    const session = useTimestowSession();
    return (
      <Show when={session.activeProfile()}>
        {(profile) => <BackupScheduleDialog profile={profile()} />}
      </Show>
    );
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <Schedule />
      </TimestowSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("button", { name: "Schedule" })).toBeDefined();
  });
  screen.getByRole("button", { name: "Schedule" }).click();
  expect(
    screen.getByRole("dialog", { name: "Backup schedule" }).text,
  ).toContain("while Timestow is running");
  const automatic = screen.getByRole("switch", {
    name: "Run backups automatically",
  });
  const frequency = screen.getByRole("combobox", {
    name: "Backup frequency",
  });
  expect(automatic.checked).toBe(false);
  expect(frequency.disabled).toBe(true);

  automatic.click();
  expect(frequency.disabled).toBe(false);
  frequency.click();
  screen.getByRole("option", { name: "Every 6 hours" }).click();
  screen.getByRole("button", { name: "Save schedule" }).click();
  await screen.waitFor(() => {
    expect(save).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "photos",
        schedule: expect.objectContaining({
          enabled: true,
          intervalMinutes: 360,
          nextRunAt: expect.any(String),
        }),
      }),
      { activate: false },
    );
  });
});

test("rustic sidebar exposes stable navigation and repository status", () => {
  const selectProfile = vi.fn<(profileId: string) => void>();
  const forgetProfile = vi.fn<(profileId: string) => void>();
  const create = vi.fn<() => void>();
  const screen = renderComponent(() => (
    <TimestowSidebar
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
      onForgetProfile={forgetProfile}
    />
  ));

  expect(screen.getByRole("button", { name: "Photos" }).selected).toBe(true);
  expect(screen.roots[0]?.text).toContain("Photos");

  screen.getByRole("button", { name: "Photos" }).click();
  expect(selectProfile).toHaveBeenCalledWith("photos");
  screen.getByRole("button", { name: "New backup" }).click();
  expect(create).toHaveBeenCalledOnce();

  screen.getByRole("button", { name: "Photos" }).contextMenu();
  screen.getByRole("menuitem", { name: "Forget backup…" }).click();
  const confirmation = screen.getByRole("alertdialog", {
    name: "Forget backup",
  });
  expect(confirmation.text).toContain("will remain untouched");
  screen.getByRole("button", { name: "Forget Photos" }).click();
  expect(forgetProfile).toHaveBeenCalledWith("photos");
});
