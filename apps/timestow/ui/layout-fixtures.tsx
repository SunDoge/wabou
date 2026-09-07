import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  defineComponentFixtures,
  defineLayoutFixtures,
} from "@wabou/test/layout/fixtures";
import {
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  PageHeader,
  View,
} from "@wabou/ui";
import { BackupConnectionForm } from "./setup";
import { TimestowSidebar } from "./shell";
import { SnapshotWorkspaceHeader } from "./snapshots";

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
  }),
  { colorTheme: false },
);
