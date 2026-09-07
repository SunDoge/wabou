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
import {
  FILE_PAGE_SIZE,
  type FileEntry,
  type RepositoryCheckResult,
  type RestorePlanSummary,
  type RusticCapability,
  type SnapshotEntry,
} from "./api";
import { BackupProgressStatus } from "./backup-progress";
import { FileDetails, RestorePlanReview } from "./file-details";
import type { ProfileStore } from "./profile-store";
import {
  RepositoryCheckDialog,
  RepositoryCheckResultView,
} from "./repository-check";
import { TimestowSessionProvider, useTimestowSession } from "./session";
import { BackupConnectionForm } from "./setup";
import { AppShell, SessionErrorBanner, TimestowSidebar } from "./shell";
import { SnapshotDiffPanel } from "./snapshot-diff";
import { SnapshotsPage, SnapshotWorkspaceHeader } from "./snapshots";

const profile = {
  id: "home",
  name: "Home archive",
  repositoryPath: "/data/backups/home-archive",
  sources: ["/data/photos", "/data/documents"],
  schedule: {
    enabled: true,
    intervalMinutes: 360 as const,
    nextRunAt: "2027-09-08T06:00:00Z",
  },
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

const manySnapshots: readonly SnapshotEntry[] = Array.from(
  { length: 12 },
  (_, index) => ({
    ...newestSnapshot,
    id: `snapshot-history-${String(index).padStart(2, "0")}`,
    time: `2026-09-${String(12 - index).padStart(2, "0")}T00:42:00Z`,
    parentId:
      index === 11
        ? undefined
        : `snapshot-history-${String(index + 1).padStart(2, "0")}`,
    label:
      index === 0
        ? "Before reorganizing the family photo archive"
        : `Nightly backup ${12 - index}`,
    deleteProtected: index === 0,
  }),
);

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
  __wabouCapabilityVersion: 12,
  status: () => fixtureStatus,
  createProfile: () => fixtureStatus,
  openProfile: () => fixtureStatus,
  selectProfile: () => fixtureStatus,
  forgetProfile: () => fixtureStatus,
  setSources: () => fixtureStatus,
  runBackup: () => ({ snapshot: newestSnapshot }),
  checkRepository: () => ({
    healthy: true,
    findings: [],
    stats: {
      repositorySize: 734_003_200,
      uniqueDataSize: 1_932_735_488,
      snapshotCount: 24,
      packCount: 83,
    },
  }),
  listSnapshots: () => [newestSnapshot, previousSnapshot],
  listFiles: ({ path, offset = 0, limit = FILE_PAGE_SIZE }) => {
    const all = path ? [] : [...rootFiles];
    const entries = all.slice(offset, offset + limit);
    return {
      entries,
      total: all.length,
      offset,
      hasMore: offset + entries.length < all.length,
    };
  },
  searchFiles: () => [],
  diffSnapshots: () => ({
    entries: [
      {
        name: "family-photo.jpg",
        path: "Photos/2026/family-photo.jpg",
        kind: "file",
        change: "added",
        currentSize: 4_238_112,
        currentModified: "2026-09-08T00:40:00Z",
      },
      {
        name: "taxes.pdf",
        path: "Documents/Finance/taxes.pdf",
        kind: "file",
        change: "modified",
        previousSize: 842_121,
        currentSize: 846_008,
        currentModified: "2026-09-08T00:41:00Z",
      },
    ],
    summary: {
      added: 1,
      removed: 0,
      modified: 1,
      metadata: 0,
      typeChanged: 0,
    },
    totalEntries: 2,
    truncated: false,
  }),
  updateSnapshot: (request) => ({
    ...newestSnapshot,
    label: request.label,
    description: request.description,
    tags: request.tags,
    deleteProtected: request.deleteProtected,
  }),
  deleteSnapshot: () => {},
  previewRestore: () => emptyPlan,
  restorePath: ({ destination }) => ({ destination, plan: emptyPlan }),
  previewPath: ({ path }) => ({ destination: path, plan: emptyPlan }),
  openPath: () => {},
};

const fixtureStore: ProfileStore = {
  load: async () => ({ profiles: [profile], activeProfileId: profile.id }),
  save: async () => {},
  setActive: async () => {},
  remove: async () => {},
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
            onRenameProfile={() => {}}
            onForgetProfile={() => {}}
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
                passwordSecret="timestow:fixture"
                confirmationSecret="timestow:fixture:confirmation"
                onModeChange={() => {}}
                onNameChange={() => {}}
                onPathChange={() => {}}
                onSubmit={() => {}}
              />
            </View>
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

function UnlockBackupFixture() {
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 min-h-0 flex flex-row bg-canvas text-primary">
          <TimestowSidebar
            active={profile.id}
            profiles={[profile]}
            unlockedProfileIds={[]}
            onCreate={() => {}}
            onSelectProfile={() => {}}
            onRenameProfile={() => {}}
            onForgetProfile={() => {}}
          />
          <View class="min-w-0 min-h-0 flex-1 px-6 py-5">
            <View class="w-full max-w-3xl mx-auto flex flex-col gap-5">
              <PageHeader
                title="Unlock backup"
                description="Enter the repository password to continue. Passwords are never stored in the profile database."
              />
              <BackupConnectionForm
                mode="open"
                name={profile.name}
                path={profile.repositoryPath}
                passwordSecret={`timestow:repository:${profile.id}`}
                confirmationSecret={`timestow:repository:${profile.id}:confirmation`}
                locked
                onModeChange={() => {}}
                onNameChange={() => {}}
                onPathChange={() => {}}
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
              repositoryControl={<Button variant="outline">Verify</Button>}
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

function WorkspaceFixture(props: { rustic?: RusticCapability }) {
  const inheritedHost = useHost();
  return (
    <HostProvider
      value={
        {
          ...inheritedHost,
          rustic: props.rustic ?? fixtureRustic,
        } as typeof inheritedHost
      }
    >
      <TimestowSessionProvider store={fixtureStore}>
        <LoadedWorkspaceRouter />
      </TimestowSessionProvider>
    </HostProvider>
  );
}

function FullWorkspaceFixture() {
  return <WorkspaceFixture />;
}

function PagedWorkspaceFixture() {
  return (
    <WorkspaceFixture
      rustic={{
        ...fixtureRustic,
        listFiles: ({ offset = 0 }) => ({
          entries: offset === 0 ? [...rootFiles] : [],
          total: 300,
          offset,
          hasMore: offset === 0,
        }),
      }}
    />
  );
}

function ManySnapshotsWorkspaceFixture() {
  return (
    <WorkspaceFixture
      rustic={{
        ...fixtureRustic,
        listSnapshots: () => [...manySnapshots],
      }}
    />
  );
}

function EmptyWorkspaceFixture() {
  return (
    <WorkspaceFixture
      rustic={{
        ...fixtureRustic,
        listSnapshots: () => [],
      }}
    />
  );
}

function WorkspaceErrorFixture() {
  return (
    <WorkspaceFixture
      rustic={{
        ...fixtureRustic,
        listSnapshots: () =>
          Promise.reject(new Error("Repository index could not be read.")),
      }}
    />
  );
}

function BackupProgressFixture() {
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 bg-surface px-4 py-3 text-primary">
          <BackupProgressStatus
            progress={{
              profileId: profile.id,
              state: "running",
              kind: "bytes",
              title:
                "Packing documents from a deliberately long source directory",
              current: 128 * 1024 * 1024,
              total: 512 * 1024 * 1024,
            }}
          />
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

function SessionErrorFixture() {
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 bg-surface text-primary">
          <SessionErrorBanner
            message="The scheduled backup could not open its repository. Unlock it and try again."
            onDismiss={() => {}}
          />
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

function RepositoryCheckFixture() {
  const inheritedHost = useHost();
  const result: RepositoryCheckResult = {
    healthy: true,
    findings: [],
    stats: {
      repositorySize: 734_003_200,
      uniqueDataSize: 1_932_735_488,
      snapshotCount: 24,
      packCount: 83,
    },
  };
  return (
    <HostProvider
      value={
        {
          ...inheritedHost,
          rustic: { ...fixtureRustic, checkRepository: () => result },
        } as typeof inheritedHost
      }
    >
      <ColorThemeProvider theme="light">
        <ComponentsProvider theme="light">
          <View class="w-full h-full min-w-0 min-h-0 bg-canvas p-4 text-primary">
            <RepositoryCheckDialog profileId={profile.id} defaultOpen />
          </View>
        </ComponentsProvider>
      </ColorThemeProvider>
    </HostProvider>
  );
}

function RepositoryCheckResultFixture() {
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 min-h-0 bg-canvas p-4 text-primary">
          <View class="w-full min-w-0 rounded-xl border border-subtle bg-surface p-5">
            <RepositoryCheckResultView
              result={{
                healthy: true,
                findings: [],
                stats: {
                  repositorySize: 734_003_200,
                  uniqueDataSize: 1_932_735_488,
                  snapshotCount: 24,
                  packCount: 83,
                },
              }}
            />
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

function SnapshotDiffFixture() {
  const inheritedHost = useHost();
  return (
    <HostProvider
      value={
        { ...inheritedHost, rustic: fixtureRustic } as typeof inheritedHost
      }
    >
      <ColorThemeProvider theme="light">
        <ComponentsProvider theme="light">
          <View class="w-full h-full min-w-0 min-h-0 bg-surface text-primary">
            <SnapshotDiffPanel
              profileId={profile.id}
              snapshot={newestSnapshot}
              snapshots={[newestSnapshot, previousSnapshot]}
            />
          </View>
        </ComponentsProvider>
      </ColorThemeProvider>
    </HostProvider>
  );
}

function FileDetailsFixture() {
  const inheritedHost = useHost();
  return (
    <HostProvider
      value={
        { ...inheritedHost, rustic: fixtureRustic } as typeof inheritedHost
      }
    >
      <ColorThemeProvider theme="light">
        <ComponentsProvider theme="light">
          <View class="w-full h-full min-w-0 min-h-0 border border-subtle bg-surface text-primary">
            <FileDetails
              profileId={profile.id}
              snapshotId={newestSnapshot.id}
              entry={{
                name: "backup-notes.md",
                path: "Documents/Archive/backup-notes.md",
                kind: "file",
                size: 18_240,
                modified: "2026-09-08T00:41:00Z",
              }}
            />
          </View>
        </ComponentsProvider>
      </ColorThemeProvider>
    </HostProvider>
  );
}

function RestorePlanFixture() {
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 min-h-0 bg-surface p-4 text-primary">
          <RestorePlanReview
            plan={{
              restoreSize: 18_240,
              matchedSize: 0,
              filesToRestore: 12,
              filesToModify: 3,
              filesUnchanged: 8,
              directoriesToRestore: 2,
              directoriesToModify: 1,
            }}
          />
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
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
    "timestow/unlock-minimum": {
      width: 900,
      height: 620,
      render: UnlockBackupFixture,
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
    "timestow/workspace-paged-directory": {
      width: 900,
      height: 620,
      waitMs: 100,
      render: PagedWorkspaceFixture,
    },
    "timestow/workspace-many-snapshots": {
      width: 900,
      height: 620,
      waitMs: 100,
      render: ManySnapshotsWorkspaceFixture,
    },
    "timestow/workspace-empty": {
      width: 900,
      height: 620,
      waitMs: 100,
      render: EmptyWorkspaceFixture,
    },
    "timestow/workspace-error": {
      width: 900,
      height: 620,
      waitMs: 100,
      render: WorkspaceErrorFixture,
    },
    "timestow/backup-progress-narrow": {
      width: 420,
      height: 88,
      render: BackupProgressFixture,
    },
    "timestow/session-error-narrow": {
      width: 420,
      height: 96,
      render: SessionErrorFixture,
    },
    "timestow/repository-check": {
      width: 520,
      height: 340,
      render: RepositoryCheckFixture,
    },
    "timestow/repository-check-result": {
      width: 480,
      height: 300,
      render: RepositoryCheckResultFixture,
    },
    "timestow/changes-wide": {
      width: 960,
      height: 620,
      waitMs: 100,
      render: SnapshotDiffFixture,
    },
    "timestow/changes-minimum": {
      width: 420,
      height: 480,
      waitMs: 100,
      render: SnapshotDiffFixture,
    },
    "timestow/file-details-rail": {
      width: 288,
      height: 620,
      waitMs: 100,
      render: FileDetailsFixture,
    },
    "timestow/restore-plan": {
      width: 480,
      height: 300,
      render: RestorePlanFixture,
    },
  }),
  { colorTheme: false },
);
