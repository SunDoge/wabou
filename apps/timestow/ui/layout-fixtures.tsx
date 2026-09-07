import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  defineComponentFixtures,
  defineLayoutFixtures,
} from "@wabou/test/layout/fixtures";
import {
  BaseRootRoute,
  BaseRoute,
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  createDataRouter,
  createMemoryHistory,
  HostProvider,
  PageHeader,
  RouterProvider,
  Text,
  useHost,
  View,
} from "@wabou/ui";
import { Show } from "solid-js";
import type {
  FileEntry,
  RestorePlanSummary,
  RusticCapability,
  SnapshotEntry,
} from "./api";
import type { ProfileStore } from "./profile-store";
import { TimestowSessionProvider, useTimestowSession } from "./session";
import { BackupConnectionForm } from "./setup";
import { AppShell, TimestowSidebar } from "./shell";
import { SnapshotsPage, SnapshotWorkspaceHeader } from "./snapshots";

const profile = {
  id: "home",
  name: "Home archive",
  repositoryPath: "/data/backups/home-archive",
  sources: ["/data/photos", "/data/documents"],
};

const newestSnapshot: SnapshotEntry = {
  id: "a7f3cc920b0f4721b1cf8a2d385aa971",
  time: "2026-09-08T00:42:00Z",
  hostname: "workstation",
  paths: [...profile.sources],
  filesNew: 18,
  filesChanged: 4,
  parentId: "4ce10adbfa3745b296e59d369f724450",
  label: "Before photo cleanup",
  description: "Automatic layout fixture",
  tags: ["home"],
  deleteProtected: true,
};

const previousSnapshot: SnapshotEntry = {
  ...newestSnapshot,
  id: "4ce10adbfa3745b296e59d369f724450",
  time: "2026-09-07T18:15:00Z",
  parentId: undefined,
  label: "Evening backup",
  filesNew: 6,
  filesChanged: 1,
  deleteProtected: false,
};

const rootFiles: readonly FileEntry[] = [
  {
    name: "Documents",
    path: "Documents",
    kind: "directory",
    size: 0,
    modified: "2026-09-08T00:39:00Z",
  },
  {
    name: "Photos",
    path: "Photos",
    kind: "directory",
    size: 0,
    modified: "2026-09-08T00:40:00Z",
  },
  {
    name: "backup-notes.md",
    path: "backup-notes.md",
    kind: "file",
    size: 18_240,
    modified: "2026-09-08T00:41:00Z",
  },
];

const emptyPlan: RestorePlanSummary = {
  restoreSize: 0,
  matchedSize: 0,
  filesToRestore: 0,
  filesToModify: 0,
  filesUnchanged: 0,
  directoriesToRestore: 0,
  directoriesToModify: 0,
};

const fixtureStatus = {
  unlockedProfileIds: [profile.id],
  activeProfileId: profile.id,
};

const fixtureRustic: RusticCapability = {
  __wabouCapabilityVersion: 5,
  status: () => fixtureStatus,
  createProfile: () => fixtureStatus,
  openProfile: () => fixtureStatus,
  selectProfile: () => fixtureStatus,
  setSources: () => fixtureStatus,
  runBackup: () => ({ snapshot: newestSnapshot }),
  listSnapshots: () => [newestSnapshot, previousSnapshot],
  listFiles: ({ path }) => (path ? [] : [...rootFiles]),
  searchFiles: () => [],
  diffSnapshots: () => ({
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
  }),
  updateSnapshot: (request) => ({
    ...newestSnapshot,
    label: request.label,
    description: request.description,
    tags: request.tags,
    deleteProtected: request.deleteProtected,
  }),
  previewRestore: () => emptyPlan,
  restorePath: ({ destination }) => ({ destination, plan: emptyPlan }),
  previewPath: ({ path }) => ({ destination: path, plan: emptyPlan }),
  openPath: () => {},
};

const fixtureStore: ProfileStore = {
  load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
  save: async () => {},
  setActive: async () => {},
};

function NewBackupFixture() {
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View
          role="region"
          aria-label="Timestow setup workspace"
          class="w-full h-full min-w-0 min-h-0 flex flex-row bg-canvas text-primary"
        >
          <TimestowSidebar
            active="new"
            profiles={[]}
            unlockedProfileIds={[]}
            onCreate={() => {}}
            onSelectProfile={() => {}}
          />
          <View class="min-w-0 min-h-0 flex-1 px-6 py-5">
            <View class="w-full max-w-3xl mx-auto flex flex-col gap-5">
              <PageHeader
                title="Create a backup"
                description="Choose what this backup is called and where its encrypted snapshots are stored."
              />
              <BackupConnectionForm
                mode="create"
                name="Photos and documents"
                path="/data/backups/a-deliberately-long-repository-name"
                password="fixture-secret"
                onModeChange={() => {}}
                onNameChange={() => {}}
                onPathChange={() => {}}
                onPasswordChange={() => {}}
                onSubmit={() => {}}
              />
            </View>
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

function WorkspaceHeaderFixture() {
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full bg-canvas p-4">
          <View class="w-full min-w-0 flex flex-col gap-3 border border-subtle bg-surface px-6 py-4">
            <SnapshotWorkspaceHeader
              name="Home archive"
              repositoryPath="/data/backups/a-deliberately-long-home-archive-repository"
              sources={["/data/photos", "/data/documents"]}
              backingUp={false}
              scheduleControl={<Button variant="outline">Schedule</Button>}
              onSourcesChange={() => {}}
              onRefresh={() => {}}
              onBackup={() => {}}
            />
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

function LoadedWorkspaceRouter() {
  const session = useTimestowSession();
  const root = new BaseRootRoute({ component: AppShell });
  const setup = new BaseRoute({
    getParentRoute: () => root,
    path: "/",
    component: () => <Text>Setup</Text>,
  });
  const snapshots = new BaseRoute({
    getParentRoute: () => root,
    path: "snapshots",
    component: SnapshotsPage,
  });
  const router = createDataRouter({
    routeTree: root.addChildren([setup, snapshots]),
    history: createMemoryHistory({ initialEntries: ["/snapshots"] }),
    context: {},
    defaultPendingMs: 0,
  });
  return (
    <Show
      when={!session.loading()}
      fallback={<Text role="status">Loading workspace</Text>}
    >
      <RouterProvider router={router} />
    </Show>
  );
}

function FullWorkspaceFixture() {
  const inheritedHost = useHost();
  return (
    <HostProvider
      value={
        { ...inheritedHost, rustic: fixtureRustic } as typeof inheritedHost
      }
    >
      <TimestowSessionProvider store={fixtureStore}>
        <LoadedWorkspaceRouter />
      </TimestowSessionProvider>
    </HostProvider>
  );
}

defineLayoutFixtures(
  defineComponentFixtures({
    "timestow/setup-wide": {
      width: 1_200,
      height: 760,
      render: NewBackupFixture,
    },
    "timestow/setup-minimum": {
      width: 900,
      height: 620,
      render: NewBackupFixture,
    },
    "timestow/workspace-header-minimum": {
      width: 676,
      height: 176,
      render: WorkspaceHeaderFixture,
    },
    "timestow/workspace-wide": {
      width: 1_440,
      height: 900,
      waitMs: 100,
      render: FullWorkspaceFixture,
    },
    "timestow/workspace-minimum": {
      width: 900,
      height: 620,
      waitMs: 100,
      render: FullWorkspaceFixture,
    },
  }),
  { colorTheme: false },
);
