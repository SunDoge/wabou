import type { Dialog } from "@wabou/core";
import { dispatchHostMessageForTest } from "@wabou/core/testing";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { Button, Text } from "@wabou/ui";
import { createSignal, Show } from "solid-js";
import { expect, test, vi } from "vitest";
import type {
  FileEntry,
  RestorePlanSummary,
  SnapshotDiff,
  SnapshotEntry,
} from "../../apps/timestow/ui/api";
import { FileDetails } from "../../apps/timestow/ui/file-details";
import {
  formatDetailedTimestamp,
  formatFileKind,
} from "../../apps/timestow/ui/format";
import type { ProfileStore } from "../../apps/timestow/ui/profile-store";
import { OPERATION_PROGRESS_TOPIC } from "../../apps/timestow/ui/operation-progress";
import { RepositoryCheckDialog } from "../../apps/timestow/ui/repository-check";
import { BackupScheduleDialog } from "../../apps/timestow/ui/schedule-dialog";
import {
  TimestowSessionProvider,
  useTimestowSession,
} from "../../apps/timestow/ui/session";
import { BackupConnectionForm } from "../../apps/timestow/ui/setup";
import {
  SessionErrorBanner,
  TimestowCloseGuard,
  TimestowSidebar,
} from "../../apps/timestow/ui/shell";
import { createSnapshotBrowserCache } from "../../apps/timestow/ui/snapshot-browser-cache";
import {
  formatSnapshotTime,
  SnapshotDetails,
} from "../../apps/timestow/ui/snapshot-details";
import {
  snapshotComparisonLabel,
  SnapshotDiffPanel,
} from "../../apps/timestow/ui/snapshot-diff";
import { SnapshotFileTree } from "../../apps/timestow/ui/snapshot-tree";
import {
  formatModified,
  SnapshotBrowserEmptyState,
  SnapshotFileRow,
  SnapshotHistory,
  SnapshotFileListEmptyState,
  SnapshotPathBreadcrumb,
  SnapshotWorkspaceHeader,
  snapshotAfterRefresh,
  snapshotDisplayTitle,
  snapshotHistoryMetadata,
  snapshotMatchesQuery,
  snapshotPathSegments,
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

test("window close asks before interrupting an active backup or extraction", () => {
  const quit = vi.fn();
  const [active, setActive] = createSignal(true);
  const screen = renderComponent(() => (
    <TimestowCloseGuard active={active()} onQuit={quit}>
      <Text>Backup workspace</Text>
    </TimestowCloseGuard>
  ));
  const app = screen.getByRole("group", { name: "Timestow window" });

  app.emit("windowcloserequested");
  expect(
    screen.getByRole("alertdialog", {
      name: "Quit while an operation is running",
    }).text,
  ).toContain("may leave the operation incomplete");
  screen.getByRole("button", { name: "Keep working" }).click();
  expect(
    screen.queryByRole("alertdialog", {
      name: "Quit while an operation is running",
    }),
  ).toBeNull();

  app.emit("windowcloserequested");
  screen.getByRole("button", { name: "Quit anyway" }).click();
  expect(quit).toHaveBeenCalledTimes(1);

  setActive(false);
  app.emit("windowcloserequested");
  expect(
    screen.queryByRole("alertdialog", {
      name: "Quit while an operation is running",
    }),
  ).toBeNull();
  screen.dispose();
});

test("session surfaces profile metadata recovery without blocking startup", async () => {
  const store: ProfileStore = {
    load: async () => ({
      profiles: [],
      recoveryNotice:
        "Timestow isolated damaged metadata for old-photos and kept the original data in recovery storage.",
    }),
    save: async () => {},
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({ unlockedProfileIds: [] }),
    },
  });
  const RecoveryStatus = () => {
    const session = useTimestowSession();
    return (
      <Show when={!session.loading() && session.error()}>
        {(message) => <Text role="alert">{message()}</Text>}
      </Show>
    );
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <RecoveryStatus />
      </TimestowSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("alert").text).toContain("old-photos");
  });
  expect(fixture.callsTo("rustic.status")).toHaveLength(1);
  screen.dispose();
});

test("session keeps an operation active through rustic phase boundaries", async () => {
  const store: ProfileStore = {
    load: async () => ({ profiles: [] }),
    save: async () => {},
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({ unlockedProfileIds: [] }),
    },
  });
  const OperationStatus = () => {
    const session = useTimestowSession();
    return (
      <Text role="status">
        {session.hasActiveOperations() ? "Operation running" : "Idle"}
      </Text>
    );
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <OperationStatus />
      </TimestowSessionProvider>
    ),
    { host: fixture.host },
  );
  const emitProgress = (state: "running" | "phaseComplete" | "completed") =>
    dispatchHostMessageForTest(
      OPERATION_PROGRESS_TOPIC,
      JSON.stringify({
        profileId: "photos",
        operation: "restore",
        operationId: "restore:one",
        state,
        unit: "spinner",
        title: "Extracting",
        current: 0,
      }),
    );

  expect(screen.getByRole("status").text).toBe("Idle");
  emitProgress("running");
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Operation running");
  });
  emitProgress("phaseComplete");
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Operation running");
  });
  emitProgress("completed");
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Idle");
  });
  screen.dispose();
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
        confirmationSecret="timestow:test:confirmation"
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
  expect(screen.getByRole("button", { name: "Create backup" }).disabled).toBe(
    true,
  );
  password.emit("secretstatechange", { hasValue: true });
  expect(screen.getByRole("button", { name: "Create backup" }).disabled).toBe(
    true,
  );
  const confirmation = screen.getByRole("textbox", {
    name: "Confirm repository password",
  });
  expect(confirmation.tag).toBe("password-input");
  expect(confirmation.attribute("secret")).toBe("timestow:test:confirmation");
  expect(confirmation.value).toBeNull();
  confirmation.emit("secretstatechange", { hasValue: true });
  expect(screen.getByRole("button", { name: "Create backup" }).disabled).toBe(
    false,
  );

  screen.getByRole("button", { name: "Open an existing repository" }).click();
  screen.flush();

  expect(screen.queryByRole("button", { name: "Create backup" })).toBeNull();
  expect(
    screen.queryByRole("textbox", { name: "Confirm repository password" }),
  ).toBeNull();
  expect(screen.getByRole("button", { name: "Open repository" }).disabled).toBe(
    false,
  );
  screen.getByRole("button", { name: "Open repository" }).click();
  expect(submit).toHaveBeenCalledTimes(1);
});

test("repository setup locks secret editors while connecting", () => {
  const screen = renderComponent(() => (
    <BackupConnectionForm
      mode="create"
      name="Photos"
      path="/data/backups/photos"
      passwordSecret="timestow:test"
      confirmationSecret="timestow:test:confirmation"
      pending="create"
      onModeChange={() => {}}
      onNameChange={() => {}}
      onPathChange={() => {}}
      onSubmit={() => {}}
    />
  ));

  expect(
    screen.getByRole("textbox", { name: "Repository password" }).disabled,
  ).toBe(true);
  expect(
    screen.getByRole("textbox", { name: "Confirm repository password" })
      .disabled,
  ).toBe(true);
  expect(screen.getByRole("button", { name: "Creating…" }).disabled).toBe(true);
});

test("backup workspace keeps configuration and primary actions distinct", () => {
  const refresh = vi.fn();
  const backup = vi.fn();
  const [showBackupAction, setShowBackupAction] = createSignal(true);
  const screen = renderComponent(
    () => (
      <SnapshotWorkspaceHeader
        name="Home archive"
        repositoryPath="/data/backups/home"
        sources={["/data/photos"]}
        backingUp={false}
        showBackupAction={showBackupAction()}
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

  setShowBackupAction(false);
  screen.flush();
  expect(screen.queryByRole("button", { name: "Back up now" })).toBeNull();
  expect(
    screen.getByRole("button", { name: "Refresh snapshots" }),
  ).toBeDefined();
});

test("repository verification reports its scope and native result", async () => {
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      checkRepository: async () => ({
        healthy: true,
        findings: [],
        stats: {
          repositorySize: 734_003_200,
          uniqueDataSize: 1_932_735_488,
          snapshotCount: 24,
          packCount: 83,
        },
      }),
    },
  });
  const screen = renderComponent(
    () => <RepositoryCheckDialog profileId="photos" />,
    { host: fixture.host },
  );

  screen.getByRole("button", { name: "Check repository" }).click();
  expect(
    screen.getByRole("dialog", { name: "Check repository" }).text,
  ).toContain("does not read every stored data byte");
  await screen.waitFor(() => {
    expect(
      screen.getByRole("alert", {
        name: "Repository structure is healthy",
      }),
    ).toBeDefined();
    expect(
      screen.getByRole("group", { name: "Repository statistics" }).text,
    ).toContain("700.0 MB");
  });
  expect(fixture.callsTo("rustic.checkRepository")[0]?.args[0]).toEqual({
    profileId: "photos",
  });
});

test("repository verification exposes integrity findings and request failures", async () => {
  let attempt = 0;
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      checkRepository: async () => {
        attempt += 1;
        if (attempt > 1) throw new Error("repository key is unavailable");
        return {
          healthy: false,
          findings: ["pack 42 is missing"],
        };
      },
    },
  });
  const screen = renderComponent(
    () => <RepositoryCheckDialog profileId="photos" />,
    { host: fixture.host },
  );

  screen.getByRole("button", { name: "Check repository" }).click();
  await screen.waitFor(() => {
    expect(
      screen.getByRole("alert", { name: "Repository problems found" }).text,
    ).toContain("1 finding");
    expect(
      screen.getByRole("dialog", { name: "Check repository" }).text,
    ).toContain("pack 42 is missing");
  });

  screen.getByRole("button", { name: "Check again" }).click();
  await screen.waitFor(() => {
    expect(
      screen.getByRole("alert", { name: "Repository check failed" }).text,
    ).toContain("repository key is unavailable");
  });
  expect(
    screen.getByRole("dialog", { name: "Check repository" }).text,
  ).not.toContain("pack 42 is missing");
});

test("repository verification discards results from a previous profile", async () => {
  const completions = new Map<
    string,
    (result: { healthy: boolean; findings: string[] }) => void
  >();
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      checkRepository: ({ profileId }: { profileId: string }) =>
        new Promise<{ healthy: boolean; findings: string[] }>((resolve) => {
          completions.set(profileId, resolve);
        }),
    },
  });
  const App = () => {
    const [profileId, setProfileId] = createSignal("photos");
    return (
      <>
        <Button
          aria-label="Select documents backup"
          onClick={() => setProfileId("documents")}
        />
        <RepositoryCheckDialog profileId={profileId()} defaultOpen />
      </>
    );
  };
  const screen = renderComponent(App, { host: fixture.host });

  screen.getByRole("button", { name: "Check now" }).click();
  await screen.waitFor(() => {
    expect(completions.has("photos")).toBe(true);
  });
  screen.getByRole("button", { name: "Select documents backup" }).click();
  completions.get("photos")?.({
    healthy: false,
    findings: ["old repository finding"],
  });
  await screen.waitFor(() => {
    expect(
      screen.getByRole("dialog", { name: "Check repository" }).text,
    ).not.toContain("old repository finding");
  });

  screen.getByRole("button", { name: "Check now" }).click();
  await screen.waitFor(() => {
    expect(completions.has("documents")).toBe(true);
  });
  completions.get("documents")?.({ healthy: true, findings: [] });
  await screen.waitFor(() => {
    expect(
      screen.getByRole("alert", { name: "Repository structure is healthy" }),
    ).toBeDefined();
  });
  expect(
    fixture.callsTo("rustic.checkRepository").map((call) => call.args[0]),
  ).toEqual([{ profileId: "photos" }, { profileId: "documents" }]);
});

test("empty snapshot workspace only offers actions the user can take", () => {
  const backup = vi.fn();
  const screen = renderComponent(() => (
    <SnapshotBrowserEmptyState
      loading={false}
      loadFailed={false}
      hasSnapshots={false}
      sourceCount={2}
      backingUp={false}
      onBackup={backup}
    />
  ));

  expect(
    screen.queryByRole("heading", { name: "Select a snapshot" }),
  ).toBeNull();
  expect(
    screen.getByRole("heading", { name: "Create your first snapshot" }),
  ).toBeDefined();
  screen.getByRole("button", { name: "Back up now" }).click();
  expect(backup).toHaveBeenCalledTimes(1);
});

test("snapshot workspace distinguishes setup, loading, and load failure", () => {
  const [state, setState] = createSignal({ loading: false, loadFailed: false });
  const screen = renderComponent(() => (
    <SnapshotBrowserEmptyState
      loading={state().loading}
      loadFailed={state().loadFailed}
      hasSnapshots={false}
      sourceCount={0}
      backingUp={false}
      onBackup={() => {}}
    />
  ));

  expect(
    screen.getByRole("heading", {
      name: "Choose folders to back up",
    }),
  ).toBeDefined();
  setState({ loading: true, loadFailed: false });
  screen.flush();
  expect(
    screen.getByRole("status", {
      name: "Loading snapshots",
    }),
  ).toBeDefined();
  setState({ loading: false, loadFailed: true });
  screen.flush();
  expect(
    screen.getByRole("alert", {
      name: "Snapshot history unavailable",
    }),
  ).toBeDefined();
});

test("empty file lists distinguish search results from empty directories", () => {
  const clearSearch = vi.fn();
  const [state, setState] = createSignal({
    searchActive: true,
    query: "invoice",
    path: "Documents",
  });
  const screen = renderComponent(() => (
    <SnapshotFileListEmptyState
      searchActive={state().searchActive}
      query={state().query}
      path={state().path}
      onClearSearch={clearSearch}
    />
  ));

  expect(
    screen.getByRole("status", { name: "No matching files" }).text,
  ).toContain("invoice");
  screen.getByRole("button", { name: "Clear search" }).click();
  expect(clearSearch).toHaveBeenCalledTimes(1);

  setState({ searchActive: false, query: "", path: "Documents" });
  screen.flush();
  expect(
    screen.getByRole("status", { name: "This folder is empty" }),
  ).toBeDefined();

  setState({ searchActive: false, query: "", path: "" });
  screen.flush();
  expect(
    screen.getByRole("status", { name: "This snapshot is empty" }),
  ).toBeDefined();
});

test("snapshot timestamps stay compact in the table", () => {
  expect(formatModified("2026-09-02T04:18:35.321355Z")).toBe(
    "2026-09-02 04:18",
  );
  expect(formatModified(undefined)).toBe("—");
});

test("detail metadata uses readable types and explicit time zones", () => {
  expect(formatDetailedTimestamp("2026-09-02T04:18:35.321355Z")).toBe(
    "2026-09-02 04:18:35 UTC",
  );
  expect(formatDetailedTimestamp("2026-09-02T12:18:35+0800")).toBe(
    "2026-09-02 12:18:35 UTC+08:00",
  );
  expect(formatDetailedTimestamp("recorded by legacy rustic")).toBe(
    "recorded by legacy rustic",
  );
  expect(
    (["directory", "file", "symlink", "special"] as const).map(formatFileKind),
  ).toEqual(["Folder", "File", "Symbolic link", "Special file"]);
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

test("snapshot titles prefer user labels and fall back to a short stable id", () => {
  const snapshot = {
    id: "f21dc6d86a8b42b4aaf81ff39aa15c8f",
    time: "2026-09-02T04:18:35Z",
    hostname: "workstation",
    paths: ["/data/photos"],
    filesNew: 1,
    filesChanged: 0,
    label: "  Before cleanup  ",
    tags: [],
    deleteProtected: false,
  };

  expect(snapshotDisplayTitle(snapshot)).toBe("Before cleanup");
  expect(snapshotDisplayTitle({ ...snapshot, label: "   " })).toBe(
    "Snapshot f21dc6d8",
  );
  expect(snapshotHistoryMetadata(snapshot)).toBe(
    "2026-09-02 04:18 · workstation",
  );
  expect(snapshotHistoryMetadata({ ...snapshot, hostname: "" })).toBe(
    "2026-09-02 04:18 · Unknown host",
  );
});

test("snapshot breadcrumbs expose every ancestor as a direct navigation target", () => {
  expect(snapshotPathSegments("Documents/Finance/2026")).toEqual([
    { label: "Root", path: "" },
    { label: "Documents", path: "Documents" },
    { label: "Finance", path: "Documents/Finance" },
    { label: "2026", path: "Documents/Finance/2026" },
  ]);

  const navigate = vi.fn<(path: string) => void>();
  const screen = renderComponent(() => (
    <SnapshotPathBreadcrumb
      path="Documents/Finance/2026"
      onNavigate={navigate}
    />
  ));
  screen.getByRole("link", { name: "Open Finance" }).click();
  expect(navigate).toHaveBeenCalledWith("Documents/Finance");
  expect(screen.getByRole("link", { name: "2026" }).disabled).toBe(true);
});

test("long snapshot histories filter by user-facing metadata", () => {
  const snapshots = Array.from({ length: 8 }, (_, index) => ({
    id: `snapshot-${index}`,
    time: `2026-09-${String(index + 1).padStart(2, "0")}T04:18:00Z`,
    hostname: `workstation-${index}`,
    paths: ["/data/photos"],
    filesNew: index,
    filesChanged: 0,
    label: `Backup ${index}`,
    tags: index === 7 ? ["milestone"] : [],
    deleteProtected: false,
  }));
  const [query, setQuery] = createSignal("");
  const select = vi.fn();
  const screen = renderComponent(() => (
    <SnapshotHistory
      loading={false}
      loadFailed={false}
      snapshots={snapshots}
      query={query()}
      onQueryChange={setQuery}
      onSelect={select}
    />
  ));

  expect(snapshotMatchesQuery(snapshots[7]!, "milestone")).toBe(true);
  screen
    .getByRole("textbox", { name: "Filter snapshots" })
    .input("workstation-7");
  const matchingSnapshot = screen.getByRole("button", {
    name: "Open snapshot Backup 7",
  });
  expect(matchingSnapshot.text).toContain("2026-09-08 04:18 · workstation-7");
  expect(matchingSnapshot.text).not.toContain("snapshot-7");
  expect(
    screen.queryByRole("button", { name: "Open snapshot Backup 0" }),
  ).toBeNull();

  screen.getByRole("button", { name: "Clear snapshot filter" }).click();
  screen.getByRole("button", { name: "Open snapshot Backup 0" }).click();
  expect(select).toHaveBeenCalledWith(snapshots[0]);
});

test("snapshot history distinguishes repository failure from empty data", () => {
  const screen = renderComponent(() => (
    <SnapshotHistory
      loading={false}
      loadFailed
      snapshots={[]}
      query=""
      onQueryChange={() => {}}
      onSelect={() => {}}
    />
  ));

  expect(
    screen.getByRole("alert", { name: "History unavailable" }),
  ).toBeDefined();
  expect(screen.roots[0]?.text).not.toContain("No snapshots yet");
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
  expect(details.text).toContain("2026-09-02 04:18:35 UTC");
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
      __wabouCapabilityVersion: 14,
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

  expect(snapshotComparisonLabel(parent)).toBe(
    "Snapshot parent-s · 2026-09-01 04:18",
  );
  expect(snapshotComparisonLabel({ ...parent, label: "Before cleanup" })).toBe(
    "Before cleanup · 2026-09-01 04:18",
  );
  expect(
    screen.getByRole("combobox", { name: "Comparison snapshot" }).text,
  ).toContain("Snapshot parent-s · 2026-09-01 04:18");

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

test("snapshot comparison failures can be retried in place", async () => {
  const current: SnapshotEntry = {
    id: "current",
    parentId: "parent",
    time: "2026-09-02T04:18:35Z",
    hostname: "workstation",
    paths: ["/data/photos"],
    filesNew: 1,
    filesChanged: 0,
    label: "Current",
    tags: [],
    deleteProtected: false,
  };
  const parent: SnapshotEntry = {
    ...current,
    id: "parent",
    parentId: undefined,
    time: "2026-09-01T04:18:35Z",
    label: "Parent",
  };
  let attempts = 0;
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      diffSnapshots: async () => {
        attempts += 1;
        if (attempts === 1) throw new Error("repository index is busy");
        return {
          entries: [],
          summary: {
            added: 0,
            removed: 0,
            modified: 0,
            metadata: 0,
            typeChanged: 0,
          },
          totalEntries: 0,
          truncated: false,
        };
      },
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
    expect(
      screen.getByRole("alert", { name: "Could not compare snapshots" }).text,
    ).toContain("repository index is busy");
  });
  screen.getByRole("button", { name: "Retry comparison" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status", { name: "No changes" })).toBeDefined();
  });
  expect(attempts).toBe(2);
});

test("obsolete snapshot comparisons cannot populate a new selection", async () => {
  const parent: SnapshotEntry = {
    id: "parent",
    time: "2026-09-01T04:18:35Z",
    hostname: "workstation",
    paths: ["/data/photos"],
    filesNew: 0,
    filesChanged: 0,
    label: "Parent",
    tags: [],
    deleteProtected: false,
  };
  const current: SnapshotEntry = {
    ...parent,
    id: "current",
    parentId: parent.id,
    time: "2026-09-02T04:18:35Z",
    label: "Current",
  };
  const solo: SnapshotEntry = {
    ...current,
    id: "solo",
    parentId: undefined,
    label: "Only snapshot",
  };
  const resolvers: Array<(result: SnapshotDiff) => void> = [];
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      diffSnapshots: () =>
        new Promise<SnapshotDiff>((resolve) => resolvers.push(resolve)),
    },
  });
  const [selection, setSelection] = createSignal<{
    snapshot: SnapshotEntry;
    snapshots: readonly SnapshotEntry[];
  }>({ snapshot: current, snapshots: [current, parent] });
  const screen = renderComponent(
    () => (
      <SnapshotDiffPanel
        profileId="photos"
        snapshot={selection().snapshot}
        snapshots={selection().snapshots}
      />
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(fixture.callsTo("rustic.diffSnapshots").length).toBeGreaterThan(0);
  });
  setSelection({ snapshot: solo, snapshots: [solo] });
  screen.flush();
  expect(
    screen.getByRole("status", { name: "Create another snapshot to compare" }),
  ).toBeDefined();

  for (const resolve of resolvers) {
    resolve({
      entries: [
        {
          name: "stale.txt",
          path: "stale.txt",
          kind: "file",
          change: "added",
          currentSize: 12,
        },
      ],
      summary: {
        added: 1,
        removed: 0,
        modified: 0,
        metadata: 0,
        typeChanged: 0,
      },
      totalEntries: 1,
      truncated: false,
    });
  }
  await screen.waitFor(() => {
    expect(
      screen.getByRole("status", {
        name: "Create another snapshot to compare",
      }),
    ).toBeDefined();
  });
  expect(screen.queryByRole("row", { name: "stale.txt" })).toBeNull();
});

test("snapshot browser cache preserves navigation while invalidating listings", () => {
  const cache = createSnapshotBrowserCache();
  const docs = [
    { name: "guide.md", path: "docs/guide.md", kind: "file" as const, size: 8 },
  ];

  cache.remember("snapshot-a", "docs", { entries: docs, total: 1 });
  cache.remember("snapshot-b", "photos", { entries: [], total: 0 });
  cache.rememberSelection("profile-a", "snapshot-a");
  cache.rememberSelection("profile-b", "snapshot-b");

  expect(cache.lastPath("snapshot-a")).toBe("docs");
  expect(cache.listing("snapshot-a", "docs")).toEqual({
    entries: docs,
    total: 1,
  });
  expect(cache.lastPath("snapshot-b")).toBe("photos");
  expect(cache.lastPath("snapshot-c")).toBe("");
  expect(cache.selectedSnapshot("profile-a")).toBe("snapshot-a");

  cache.replaceSnapshot("profile-a", "snapshot-a", "snapshot-a-updated");
  expect(cache.selectedSnapshot("profile-a")).toBe("snapshot-a-updated");
  expect(cache.lastPath("snapshot-a-updated")).toBe("docs");
  expect(cache.listing("snapshot-a-updated", "docs")).toEqual({
    entries: docs,
    total: 1,
  });
  expect(cache.lastPath("snapshot-a")).toBe("");
  expect(cache.listing("snapshot-a", "docs")).toBeUndefined();
  expect(cache.selectedSnapshot("profile-b")).toBe("snapshot-b");

  cache.removeSnapshot("profile-b", "snapshot-b");
  expect(cache.selectedSnapshot("profile-b")).toBeUndefined();
  expect(cache.lastPath("snapshot-b")).toBe("");
  expect(cache.listing("snapshot-b", "photos")).toBeUndefined();
  expect(cache.selectedSnapshot("profile-a")).toBe("snapshot-a-updated");

  cache.clearListings();
  expect(cache.listing("snapshot-a-updated", "docs")).toBeUndefined();
  expect(cache.lastPath("snapshot-a-updated")).toBe("docs");
  expect(cache.selectedSnapshot("profile-a")).toBe("snapshot-a-updated");

  cache.clear();
  expect(cache.listing("snapshot-a-updated", "docs")).toBeUndefined();
  expect(cache.lastPath("snapshot-a-updated")).toBe("");
  expect(cache.selectedSnapshot("profile-a")).toBeUndefined();
});

test("timestow session preserves snapshot navigation across workspace remounts", async () => {
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({ unlockedProfileIds: [] }),
    },
  });
  const store: ProfileStore = {
    load: async () => ({ profiles: [] }),
    save: async () => {},
    setActive: async () => {},
    remove: async () => {},
  };
  const [mounted, setMounted] = createSignal(true);
  const Workspace = () => {
    const session = useTimestowSession();
    return (
      <>
        <Button
          aria-label="Remember snapshot location"
          onClick={() => {
            session.snapshotBrowser.rememberSelection(
              "profile-a",
              "snapshot-a",
            );
            session.snapshotBrowser.remember("snapshot-a", "docs", {
              entries: [],
              total: 0,
            });
          }}
        />
        <Text role="status">
          {session.snapshotBrowser.selectedSnapshot("profile-a") ?? "none"}:
          {session.snapshotBrowser.lastPath("snapshot-a")}
        </Text>
      </>
    );
  };
  const screen = renderComponent(
    () => (
      <TimestowSessionProvider store={store}>
        <Button
          aria-label="Toggle workspace"
          onClick={() => setMounted((current) => !current)}
        />
        <Show when={mounted()}>
          <Workspace />
        </Show>
      </TimestowSessionProvider>
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("none:");
  });
  screen.getByRole("button", { name: "Remember snapshot location" }).click();
  screen.getByRole("button", { name: "Toggle workspace" }).click();
  screen.getByRole("button", { name: "Toggle workspace" }).click();
  screen.flush();
  expect(screen.getByRole("status").text).toBe("snapshot-a:docs");
});

test("snapshot file tree loads child directories only when expanded", async () => {
  const selected = vi.fn();
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
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

test("snapshot file tree retries a failed directory without clearing loaded files", async () => {
  let docsAttempts = 0;
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      listFiles: async (request: { path: string }) => {
        if (request.path === "docs") {
          docsAttempts += 1;
          if (docsAttempts === 1) {
            throw new Error("The repository connection was interrupted.");
          }
          return {
            entries: [
              {
                name: "guide.md",
                path: "docs/guide.md",
                kind: "file" as const,
                size: 8,
              },
            ],
            total: 1,
            offset: 0,
            hasMore: false,
          };
        }
        return {
          entries: [
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
          total: 2,
          offset: 0,
          hasMore: false,
        };
      },
    },
  });
  const screen = renderComponent(
    () => (
      <SnapshotFileTree
        profileId="profile"
        snapshotId="snapshot"
        onSelect={() => {}}
      />
    ),
    { host: fixture.host },
  );

  await screen.waitFor(() => {
    expect(screen.getByRole("treeitem", { name: "docs" })).toBeDefined();
  });
  screen.getByRole("treeitem", { name: "docs" }).click();
  await screen.waitFor(() => {
    expect(
      screen.getByRole("alert", { name: "Snapshot tree load failed" }).text,
    ).toContain("Couldn’t load docs");
  });
  expect(screen.getByRole("treeitem", { name: "README.md" })).toBeDefined();

  screen.getByRole("button", { name: "Retry loading docs" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("treeitem", { name: "guide.md" })).toBeDefined();
  });
  expect(
    screen.queryByRole("alert", { name: "Snapshot tree load failed" }),
  ).toBeNull();
  expect(docsAttempts).toBe(2);
});

test("file details preview and extract through the native rustic capability", async () => {
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
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
        filesToModify: 2,
        filesUnchanged: 0,
        directoriesToRestore: 0,
        directoriesToModify: 1,
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

  expect(screen.roots[0]?.text).toContain("File");
  expect(screen.roots[0]?.text).toContain("2026-09-02 04:18:35 UTC");
  screen.getByRole("button", { name: "Open preview" }).click();
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
  const overwriteWarning = screen.getByRole("alert", {
    name: "Existing content will change",
  });
  expect(overwriteWarning.text).toContain("will be replaced");
  expect(
    screen.getByRole("dialog", { name: "Extract settings.toml" }).text,
  ).toContain("Files replaced2");
  screen.getByRole("button", { name: "Extract" }).click();
  await screen.waitFor(() => {
    expect(fixture.callsTo("rustic.restorePath")).toHaveLength(1);
  });
  screen.getByRole("button", { name: "Open extracted item" }).click();
  await screen.waitFor(() => {
    expect(fixture.callsTo("rustic.openPath")).toHaveLength(2);
  });

  expect(fixture.callsTo("rustic.previewPath")).toHaveLength(1);
  expect(fixture.callsTo("rustic.openPath")[1]?.args[0]).toEqual({
    path: "/tmp/export/settings.toml",
  });
  expect(fixture.callsTo("rustic.previewRestore")).toHaveLength(1);
  expect(fixture.callsTo("rustic.restorePath")[0]?.args[0]).toEqual({
    profileId: "profile",
    snapshotId: "snapshot",
    path: "home/me/settings.toml",
    destination: "/tmp/export",
    operationId: expect.stringMatching(/^restore:\d+:\d+$/),
  });
});

test("extract reports only the active restore operation progress", async () => {
  const operationStarted = vi.fn();
  const operationEnded = vi.fn();
  let completeRestore:
    | ((result: { destination: string; plan: RestorePlanSummary }) => void)
    | undefined;
  const restore = new Promise<{
    destination: string;
    plan: RestorePlanSummary;
  }>((resolve) => {
    completeRestore = resolve;
  });
  const plan: RestorePlanSummary = {
    restoreSize: 4_096,
    matchedSize: 0,
    filesToRestore: 1,
    filesToModify: 0,
    filesUnchanged: 0,
    directoriesToRestore: 0,
    directoriesToModify: 0,
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      previewRestore: async () => plan,
      restorePath: () => restore,
    },
  });
  const screen = renderComponent(
    () => (
      <FileDetails
        profileId="photos"
        snapshotId="snapshot"
        entry={{
          name: "cover.jpg",
          path: "chapter/cover.jpg",
          kind: "file",
          size: 4_096,
        }}
        onOperationStart={operationStarted}
        onOperationEnd={operationEnded}
      />
    ),
    { host: fixture.host, platform: { dialog } },
  );

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
  const request = fixture.callsTo("rustic.restorePath")[0]?.args[0] as {
    operationId: string;
  };
  expect(operationStarted).toHaveBeenCalledWith(request.operationId);
  expect(operationEnded).not.toHaveBeenCalled();

  dispatchHostMessageForTest(
    OPERATION_PROGRESS_TOPIC,
    JSON.stringify({
      profileId: "photos",
      operation: "restore",
      operationId: "another-restore",
      state: "running",
      unit: "bytes",
      title: "Wrong extraction",
      current: 2_048,
      total: 4_096,
    }),
  );
  expect(
    screen.queryByRole("progressbar", { name: "Extraction progress" }),
  ).toBeNull();

  dispatchHostMessageForTest(
    OPERATION_PROGRESS_TOPIC,
    JSON.stringify({
      profileId: "photos",
      operation: "restore",
      operationId: request.operationId,
      state: "running",
      unit: "bytes",
      title: "Restoring file contents",
      current: 2_048,
      total: 4_096,
    }),
  );
  await screen.waitFor(() => {
    expect(
      screen.getByRole("progressbar", { name: "Extraction progress" })
        .numericValue,
    ).toBe(2_048);
  });
  expect(
    screen
      .getAllByRole("status")
      .some((status) => status.text.includes("2.0 KB of 4.0 KB")),
  ).toBe(true);

  completeRestore?.({ destination: "/tmp/export/cover.jpg", plan });
  await screen.waitFor(() => {
    expect(
      screen.getByRole("button", { name: "Open extracted item" }),
    ).toBeDefined();
  });
  expect(operationEnded).toHaveBeenCalledWith(request.operationId);
  screen.dispose();
});

test("a preview result cannot leak into another selected file", async () => {
  const entries: FileEntry[] = [
    {
      name: "first.txt",
      path: "docs/first.txt",
      kind: "file",
      size: 10,
      modified: "2026-09-04T08:00:00Z",
    },
    {
      name: "second.txt",
      path: "docs/second.txt",
      kind: "file",
      size: 20,
      modified: "2026-09-04T08:01:00Z",
    },
  ];
  const completions = new Map<
    string,
    (result: { destination: string; plan: RestorePlanSummary }) => void
  >();
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      previewPath: ({ path }: { path: string }) =>
        new Promise<{ destination: string; plan: RestorePlanSummary }>(
          (resolve) => completions.set(path, resolve),
        ),
      openPath: async () => {},
    },
  });
  const App = () => {
    const [entry, setEntry] = createSignal(entries[0]);
    return (
      <>
        <Button
          aria-label="Select second file"
          onClick={() => setEntry(entries[1])}
        />
        <FileDetails
          profileId="profile"
          snapshotId="snapshot"
          entry={entry()}
        />
      </>
    );
  };
  const screen = renderComponent(App, { host: fixture.host });

  screen.getByRole("button", { name: "Open preview" }).click();
  await screen.waitFor(() => {
    expect(completions.has("docs/first.txt")).toBe(true);
  });
  screen.getByRole("button", { name: "Select second file" }).click();
  completions.get("docs/first.txt")?.({
    destination: "/tmp/first.txt",
    plan: {
      restoreSize: 10,
      matchedSize: 0,
      filesToRestore: 1,
      filesToModify: 0,
      filesUnchanged: 0,
      directoriesToRestore: 0,
      directoriesToModify: 0,
    },
  });
  await screen.waitFor(() => {
    expect(screen.getByRole("region", { name: "File details" }).text).toContain(
      "second.txt",
    );
  });
  expect(
    screen.getByRole("region", { name: "File details" }).text,
  ).not.toContain("/tmp/first.txt");
  expect(fixture.callsTo("rustic.openPath")).toHaveLength(0);

  screen.getByRole("button", { name: "Open preview" }).click();
  await screen.waitFor(() => {
    expect(completions.has("docs/second.txt")).toBe(true);
  });
  completions.get("docs/second.txt")?.({
    destination: "/tmp/second.txt",
    plan: {
      restoreSize: 20,
      matchedSize: 0,
      filesToRestore: 1,
      filesToModify: 0,
      filesUnchanged: 0,
      directoriesToRestore: 0,
      directoriesToModify: 0,
    },
  });
  await screen.waitFor(() => {
    expect(screen.getByRole("region", { name: "File details" }).text).toContain(
      "/tmp/second.txt",
    );
  });
  expect(fixture.callsTo("rustic.openPath")).toHaveLength(1);
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
  expect(
    screen.getByRole("group", { name: "Backup folder selection" }),
  ).toBeDefined();
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

test("backup source dialog locks updates and keeps failures recoverable", async () => {
  let rejectUpdate!: (cause: Error) => void;
  const update = vi.fn(
    () =>
      new Promise<void>((_resolve, reject) => {
        rejectUpdate = reject;
      }),
  );
  const screen = renderComponent(
    () => <BackupSourcesDialog sources={["/data/photos"]} onChange={update} />,
    { platform: { dialog } },
  );

  screen.getByRole("button", { name: "Manage backup folders" }).click();
  screen.getByRole("button", { name: "Remove /data/photos" }).click();
  expect(update).toHaveBeenCalledWith([]);
  expect(
    screen.getByRole("button", { name: "Choose backup folder" }).disabled,
  ).toBe(true);
  expect(
    screen.getByRole("button", { name: "Remove /data/photos" }).disabled,
  ).toBe(true);

  rejectUpdate(new Error("repository state is read-only"));
  await screen.waitFor(() => {
    expect(
      screen.getByRole("alert", { name: "Could not update folders" }).text,
    ).toContain("repository state is read-only");
  });
  expect(
    screen.getByRole("button", { name: "Remove /data/photos" }).disabled,
  ).toBe(false);
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
      __wabouCapabilityVersion: 14,
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

test("renaming a backup persists presentation metadata without touching Rust", async () => {
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/repository",
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
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: [profile.id],
      }),
    },
  });
  const Controls = () => {
    const session = useTimestowSession();
    return (
      <>
        <Button
          aria-label="Rename Photos"
          onClick={() => void session.renameProfile(profile.id, "Home archive")}
        />
        <Text role="status">{session.activeProfile()?.name ?? "none"}</Text>
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
    expect(screen.getByRole("status").text).toBe("Photos");
  });
  screen.getByRole("button", { name: "Rename Photos" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("Home archive");
  });
  expect(save).toHaveBeenCalledWith(
    { ...profile, name: "Home archive" },
    { activate: false },
  );
  expect(fixture.callsTo("rustic.status")).toHaveLength(1);
});

test("forgetting a backup clears native credentials before durable profile metadata", async () => {
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/repository",
    sources: ["/data/photos"],
  };
  let finishNativeForget!: () => void;
  const nativeForget = new Promise<{ unlockedProfileIds: string[] }>(
    (resolve) => {
      finishNativeForget = () => resolve({ unlockedProfileIds: [] });
    },
  );
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: [profile.id],
      }),
      forgetProfile: () => nativeForget,
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
          onClick={() => {
            void session.forgetProfile(profile.id);
            void session.forgetProfile(profile.id);
          }}
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
  expect(fixture.callsTo("rustic.forgetProfile")).toHaveLength(1);
  finishNativeForget();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("none · 0");
  });
  expect(remove).toHaveBeenCalledWith(profile.id);
  expect(remove).toHaveBeenCalledTimes(1);
});

test("a backup stays available for retry when durable forgetting fails", async () => {
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/repository",
    sources: ["/data/photos"],
  };
  const remove = vi.fn<ProfileStore["remove"]>(async () => {
    throw new Error("database is read-only");
  });
  const store: ProfileStore = {
    load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
    save: async () => {},
    setActive: async () => {},
    remove,
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: [profile.id],
      }),
      forgetProfile: async () => ({ unlockedProfileIds: [] }),
    },
  });
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
          {" · "}
          {session.runtime().unlockedProfileIds.join(",") || "locked"}
          {" · "}
          {session.error() ?? "ok"}
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
    expect(screen.getByRole("status").text).toContain(
      "Photos · 1 · photos · ok",
    );
  });
  screen.getByRole("button", { name: "Forget Photos" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toContain(
      "none · 1 · locked · Photos is locked and disconnected",
    );
  });
  expect(screen.getByRole("status").text).toContain(
    "It may reappear after restart",
  );
  expect(remove).toHaveBeenCalledWith(profile.id);
  expect(fixture.callsTo("rustic.forgetProfile")).toHaveLength(1);
});

test("profile selection is JS-owned and does not wait for its durable write", async () => {
  let finishSelection!: () => void;
  const selectionWritten = new Promise<void>((resolve) => {
    finishSelection = resolve;
  });
  const setActive = vi.fn<ProfileStore["setActive"]>(
    async () => selectionWritten,
  );
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
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: profiles.map((profile) => profile.id),
      }),
    },
  });
  const Status = () => {
    const session = useTimestowSession();
    return (
      <>
        <Button
          aria-label="Select Documents"
          onClick={() => void session.activateProfile("documents")}
        />
        <Button
          aria-label="Select Photos"
          onClick={() => void session.activateProfile("photos")}
        />
        <Text role="status">{session.activeProfile()?.name ?? "none"}</Text>
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
    expect(screen.getByRole("status").text).toBe("Photos");
  });
  screen.getByRole("button", { name: "Select Documents" }).click();
  expect(screen.getByRole("status").text).toBe("Documents");
  await screen.waitFor(() => {
    expect(setActive).toHaveBeenCalledWith("documents");
  });
  screen.getByRole("button", { name: "Select Photos" }).click();
  expect(screen.getByRole("status").text).toBe("Photos");
  expect(setActive).toHaveBeenCalledTimes(1);
  expect(fixture.callsTo("rustic.selectProfile")).toHaveLength(0);
  finishSelection();
  await screen.waitFor(() => {
    expect(setActive).toHaveBeenCalledTimes(2);
  });
  expect(setActive.mock.calls.map(([profileId]) => profileId)).toEqual([
    "documents",
    "photos",
  ]);
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
      __wabouCapabilityVersion: 14,
      status: async () => ({ unlockedProfileIds: [] }),
      createProfile: async (request: { id: string }) => ({
        unlockedProfileIds: [request.id],
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
              confirmationSlot: "timestow:test:confirmation",
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
      confirmationSlot: "timestow:test:confirmation",
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

test("a profile remains usable when durable metadata cannot be saved", async () => {
  const store: ProfileStore = {
    load: async () => ({ profiles: [] }),
    save: async () => {
      throw new Error("database is read-only");
    },
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({ unlockedProfileIds: [] }),
      createProfile: async (request: { id: string }) => ({
        unlockedProfileIds: [request.id],
      }),
    },
  });
  const Create = () => {
    const session = useTimestowSession();
    return (
      <>
        <Button
          aria-label="Create temporary backup"
          onClick={() =>
            void session.connectProfile("create", {
              name: "Photos",
              repositoryPath: "/data/backups/photos",
              passwordSlot: "timestow:test",
              confirmationSlot: "timestow:test:confirmation",
              sources: ["/data/photos"],
            })
          }
        />
        <Text role="status">
          {session.activeProfile()?.name ?? "none"} · {session.error() ?? "ok"}
        </Text>
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
    expect(screen.getByRole("status").text).toBe("none · ok");
  });
  screen.getByRole("button", { name: "Create temporary backup" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toContain(
      "Photos · Photos is connected, but Timestow could not save this backup profile: database is read-only.",
    );
  });
  expect(screen.getByRole("status").text).toContain(
    "It remains available until the app closes",
  );
  expect(fixture.callsTo("rustic.createProfile")).toHaveLength(1);
});

test("backup names remain unambiguous while reconnecting keeps its identity", async () => {
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/backups/photos",
    sources: ["/data/photos"],
  };
  const store: ProfileStore = {
    load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
    save: async () => {},
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({ unlockedProfileIds: [] }),
      createProfile: async () => ({ unlockedProfileIds: [] }),
      openProfile: async () => ({
        unlockedProfileIds: [profile.id],
      }),
    },
  });
  const Controls = () => {
    const session = useTimestowSession();
    const [result, setResult] = createSignal("pending");
    return (
      <>
        <Button
          aria-label="Create duplicate backup"
          onClick={() =>
            void session
              .connectProfile("create", {
                name: " photos ",
                repositoryPath: "/data/backups/duplicate",
                passwordSlot: "duplicate",
                confirmationSlot: "duplicate:confirmation",
              })
              .catch((cause) =>
                setResult(
                  cause instanceof Error ? cause.message : String(cause),
                ),
              )
          }
        />
        <Button
          aria-label="Reconnect Photos"
          onClick={() =>
            void session
              .connectProfile("open", {
                id: profile.id,
                name: profile.name,
                repositoryPath: profile.repositoryPath,
                passwordSlot: "photos",
                sources: profile.sources,
              })
              .then(() => setResult("reconnected"))
          }
        />
        <Text role="status">{result()}</Text>
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
    expect(screen.getByRole("status").text).toBe("pending");
  });
  screen.getByRole("button", { name: "Create duplicate backup" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe(
      "a backup named photos already exists",
    );
  });
  expect(fixture.callsTo("rustic.createProfile")).toHaveLength(0);

  screen.getByRole("button", { name: "Reconnect Photos" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("reconnected");
  });
  expect(fixture.callsTo("rustic.openProfile")).toHaveLength(1);
});

test("the UI reflects native backup folders when persistence fails", async () => {
  const profile = {
    id: "photos",
    name: "Photos",
    repositoryPath: "/data/backups/photos",
    sources: ["/data/photos"],
  };
  const store: ProfileStore = {
    load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
    save: async () => {
      throw new Error("database is read-only");
    },
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: [profile.id],
      }),
      setSources: async () => ({
        unlockedProfileIds: [profile.id],
      }),
    },
  });
  const Controls = () => {
    const session = useTimestowSession();
    return (
      <>
        <Button
          aria-label="Use document folder"
          onClick={() =>
            void session.updateSources(profile.id, ["/data/documents"])
          }
        />
        <Text role="status">
          {session.activeProfile()?.sources.join(",") ?? "none"} ·{" "}
          {session.error() ?? "ok"}
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
    expect(screen.getByRole("status").text).toBe("/data/photos · ok");
  });
  screen.getByRole("button", { name: "Use document folder" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toContain(
      "/data/documents · Photos now uses the updated folders",
    );
  });
  expect(screen.getByRole("status").text).toContain(
    "They remain active until the app closes",
  );
  expect(fixture.callsTo("rustic.setSources")).toHaveLength(1);
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
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: [profile.id],
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

test("identifies the backup that failed on schedule and offers recovery", async () => {
  const nextRunAt = new Date(Date.now() + 1_000).toISOString();
  const profile = {
    id: "photos",
    name: "Family photos",
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
    save: async () => {},
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: [profile.id],
      }),
      runBackup: async () => {
        throw new Error("repository is unavailable");
      },
    },
  });
  const Status = () => {
    const session = useTimestowSession();
    return <Text role="status">{session.error() ?? "ok"}</Text>;
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
    expect(screen.getByRole("status").text).toBe("ok");
  });
  await screen.advanceTime(1_000);
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe(
      "Automatic backup for Family photos failed: repository is unavailable. Open this backup and try again.",
    );
  });
  expect(fixture.callsTo("rustic.runBackup")).toHaveLength(1);
});

test("a later scheduled success cannot hide an earlier backup failure", async () => {
  const nextRunAt = new Date(Date.now() + 1_000).toISOString();
  const profiles = [
    {
      id: "documents",
      name: "Documents",
      repositoryPath: "/data/backups/documents",
      sources: ["/data/documents"],
      schedule: {
        enabled: true,
        intervalMinutes: 60 as const,
        nextRunAt,
      },
    },
    {
      id: "photos",
      name: "Photos",
      repositoryPath: "/data/backups/photos",
      sources: ["/data/photos"],
      schedule: {
        enabled: true,
        intervalMinutes: 60 as const,
        nextRunAt,
      },
    },
  ];
  const store: ProfileStore = {
    load: async () => ({ profiles, activeProfileId: profiles[0]?.id }),
    save: async () => {},
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: profiles.map(({ id }) => id),
      }),
      runBackup: async ({ profileId }: { profileId: string }) => {
        if (profileId === "documents") {
          throw new Error("repository is unavailable");
        }
        return {
          snapshot: {
            id: "photos-snapshot",
            time: "2026-09-04T09:00:00.000Z",
            hostname: "workstation",
            paths: ["/data/photos"],
            filesNew: 1,
            filesChanged: 0,
            label: "",
            tags: [],
            deleteProtected: false,
          },
        };
      },
    },
  });
  const Status = () => {
    const session = useTimestowSession();
    return <Text role="status">{session.error() ?? "ok"}</Text>;
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
    expect(screen.getByRole("status").text).toBe("ok");
  });
  await screen.advanceTime(1_000);
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toContain(
      "Automatic backup for Documents failed: repository is unavailable",
    );
  });
  expect(fixture.callsTo("rustic.runBackup")).toHaveLength(2);
});

test("schedule persistence cannot reenter a running backup batch", async () => {
  const nextRunAt = new Date(Date.now() + 1_000).toISOString();
  const profiles = [
    {
      id: "documents",
      name: "Documents",
      repositoryPath: "/data/backups/documents",
      sources: ["/data/documents"],
      schedule: {
        enabled: true,
        intervalMinutes: 60 as const,
        nextRunAt,
      },
    },
    {
      id: "photos",
      name: "Photos",
      repositoryPath: "/data/backups/photos",
      sources: ["/data/photos"],
      schedule: {
        enabled: true,
        intervalMinutes: 60 as const,
        nextRunAt,
      },
    },
  ];
  const completions = new Map<
    string,
    (result: { snapshot: SnapshotEntry }) => void
  >();
  const store: ProfileStore = {
    load: async () => ({ profiles, activeProfileId: profiles[0]?.id }),
    save: async () => {},
    setActive: async () => {},
    remove: async () => {},
  };
  const fixture = createTestHost({
    rustic: {
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: profiles.map(({ id }) => id),
      }),
      runBackup: ({ profileId }: { profileId: string }) =>
        new Promise<{ snapshot: SnapshotEntry }>((resolve) => {
          completions.set(profileId, resolve);
        }),
    },
  });
  const Status = () => {
    const session = useTimestowSession();
    return (
      <Text role="status">
        {session.error() ?? "ok"} · {session.lastBackup()?.profileId ?? "none"}
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
  const complete = (profileId: string) => {
    const resolve = completions.get(profileId);
    expect(resolve).toBeDefined();
    resolve?.({
      snapshot: {
        id: `${profileId}-snapshot`,
        time: "2026-09-04T09:00:00.000Z",
        hostname: "workstation",
        paths: [`/data/${profileId}`],
        filesNew: 1,
        filesChanged: 0,
        label: "",
        tags: [],
        deleteProtected: false,
      },
    });
  };

  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("ok · none");
  });
  await screen.advanceTime(1_000);
  await screen.waitFor(() => {
    expect(fixture.callsTo("rustic.runBackup")).toHaveLength(1);
  });
  complete("documents");
  await screen.waitFor(() => {
    expect(fixture.callsTo("rustic.runBackup")).toHaveLength(2);
  });

  await screen.advanceTime(0);
  expect(fixture.callsTo("rustic.runBackup")).toHaveLength(2);
  expect(screen.getByRole("status").text).toBe("ok · documents");

  complete("photos");
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("ok · photos");
  });
  await screen.advanceTime(0);
  expect(fixture.callsTo("rustic.runBackup")).toHaveLength(2);
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
      __wabouCapabilityVersion: 14,
      status: async () => ({
        unlockedProfileIds: [profile.id],
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
  expect(
    screen.getByRole("button", { name: "Backup schedule: Every 6 hours" }).text,
  ).toContain("Every 6 hours");
});

test("rustic sidebar exposes stable navigation and repository actions", async () => {
  const selectProfile = vi.fn<(profileId: string) => void>();
  let finishForget!: () => void;
  const forgetProfile = vi.fn(
    async (_profileId: string) =>
      new Promise<void>((resolve) => {
        finishForget = resolve;
      }),
  );
  const renameProfile = vi.fn(async (_profileId: string, _name: string) => {});
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
      onRenameProfile={renameProfile}
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
  screen.getByRole("menuitem", { name: "Rename backup…" }).click();
  const name = screen.getByRole("textbox", { name: "Backup name" });
  expect(name.value).toBe("Photos");
  name.input("Family archive");
  screen.getByRole("button", { name: "Rename" }).click();
  await screen.waitFor(() => {
    expect(renameProfile).toHaveBeenCalledWith("photos", "Family archive");
  });

  screen.getByRole("button", { name: "Photos" }).contextMenu();
  screen.getByRole("menuitem", { name: "Forget backup…" }).click();
  const confirmation = screen.getByRole("alertdialog", {
    name: "Forget backup",
  });
  expect(confirmation.text).toContain("will remain untouched");
  const forget = screen.getByRole("button", { name: "Forget Photos" });
  forget.click();
  expect(forgetProfile).toHaveBeenCalledWith("photos");
  expect(forget.text).toContain("Forgetting…");
  expect(forget.disabled).toBe(true);
  expect(forgetProfile).toHaveBeenCalledTimes(1);
  expect(
    screen.queryByRole("alertdialog", { name: "Forget backup" }),
  ).not.toBeNull();
  finishForget();
  await screen.waitFor(() => {
    expect(
      screen.queryByRole("alertdialog", { name: "Forget backup" }),
    ).toBeNull();
  });
});

test("forgetting a backup keeps its confirmation recoverable after failure", async () => {
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
      onCreate={() => {}}
      onSelectProfile={() => {}}
      onRenameProfile={() => {}}
      onForgetProfile={async () => {
        throw new Error("profile database is read-only");
      }}
    />
  ));

  screen.getByRole("button", { name: "Photos" }).contextMenu();
  screen.getByRole("menuitem", { name: "Forget backup…" }).click();
  screen.getByRole("button", { name: "Forget Photos" }).click();
  await screen.waitFor(() => {
    expect(
      screen.getByRole("alert", { name: "Could not forget backup" }).text,
    ).toContain("profile database is read-only");
  });
  expect(
    screen.queryByRole("alertdialog", { name: "Forget backup" }),
  ).not.toBeNull();
  expect(screen.getByRole("button", { name: "Forget Photos" }).disabled).toBe(
    false,
  );
});
