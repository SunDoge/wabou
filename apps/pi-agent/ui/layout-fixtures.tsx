import "@wabou/ui";
import "virtual:wabou-stylesheet";
import { defineLayoutFixtures } from "@wabou/test/layout/fixtures";
import { DiffViewer, View } from "@wabou/ui";
import { initialAgentState } from "./agent-state";
import { AppCommandPalette } from "./app-command-palette";
import { ModelControls } from "./model-controls";
import { type AppSettings, SettingsPage } from "./settings";
import { Sidebar } from "./sidebar";
import { createAgentWorkspace } from "./workspace";
import { WorkspacePanel } from "./workspace-panel";

const project = createAgentWorkspace(1);
project.name = "Documentation workspace";
project.cwd = "/work/wabou/documentation-and-examples";
project.provider = "anthropic";
project.model = "claude-sonnet-4-5";

const secondProject = createAgentWorkspace(2);
secondProject.name = "Release automation";
secondProject.cwd = "/work/wabou/release-automation";

const appSettings: AppSettings = {
  locale: "en",
  proxy: "http://127.0.0.1:7890",
  noProxy: "127.0.0.1,localhost",
  provider: "anthropic",
  model: "claude-sonnet-4-5",
  subagentsEnabled: true,
};

defineLayoutFixtures({
  "shell/command-palette": {
    width: 720,
    height: 520,
    render: () => (
      <View class="w-full h-full bg-canvas">
        <AppCommandPalette
          open
          label="Command palette"
          placeholder="Search actions"
          emptyText="No matching actions."
          close={() => {}}
          items={[
            {
              id: "new-session",
              label: "New session",
              description: "Start a clean conversation in this project.",
              shortcut: "⌘/Ctrl N",
            },
            {
              id: "search",
              label: "Search conversation",
              description: "Find text in the current conversation.",
              shortcut: "⌘/Ctrl F",
            },
            {
              id: "changes",
              label: "Code changes",
              description: "Review uncommitted repository changes.",
              disabled: true,
            },
          ]}
        />
      </View>
    ),
  },
  "workspace/files-panel": {
    width: 420,
    height: 720,
    render: () => (
      <WorkspacePanel
        cwd="/work/wabou"
        loadFiles={async () => ["README.md", "src/main.rs", "src/service.rs"]}
        readFile={async (_cwd, path) => ({
          path,
          text: "# Workspace preview\n\nA real layout fixture for the file inspector.",
        })}
        addContext={() => {}}
        close={() => {}}
      />
    ),
  },
  "workspace/diff-viewer": {
    width: 640,
    height: 520,
    render: () => (
      <DiffViewer
        defaultExpanded={["src/main.ts"]}
        files={[
          {
            path: "src/main.ts",
            status: "modified",
            additions: 2,
            deletions: 1,
            patch: "@@ -1,2 +1,3 @@\n-old\n+new\n+line",
          },
          {
            path: "README.md",
            status: "added",
            additions: 4,
            deletions: 0,
            patch: "@@ -0,0 +1,4 @@\n+# Wabou\n+\n+Native UI\n+",
          },
        ]}
      />
    ),
  },
  "shell/sidebar": {
    width: 300,
    height: 720,
    render: () => (
      <Sidebar
        agents={[project, secondProject]}
        sessions={[
          {
            agentId: project.id,
            sessionId: "session-one",
            sessionFile: "/tmp/session-one.jsonl",
            name: "Fix the persistent workspace resource loading loop",
            cwd: project.cwd,
            updatedAt: 2,
          },
          {
            agentId: project.id,
            sessionId: "session-two",
            sessionFile: "/tmp/session-two.jsonl",
            name: "Review release readiness",
            cwd: project.cwd,
            updatedAt: 1,
          },
        ]}
        activeId={project.id}
        select={() => {}}
        selectSession={() => {}}
        add={() => {}}
        newSession={() => {}}
        canCreateSession
        openSettings={() => {}}
      />
    ),
  },
  "shell/model-controls": {
    width: 480,
    height: 96,
    render: () => (
      <ModelControls
        models={[
          {
            provider: "anthropic",
            id: "claude-sonnet-4-5",
            name: "Claude Sonnet 4.5",
            reasoning: true,
          },
        ]}
        modelProvider="anthropic"
        modelId="claude-sonnet-4-5"
        thinking="medium"
        thinkingLevels={["off", "medium", "high"]}
        chooseModel={() => {}}
        chooseThinking={() => {}}
      />
    ),
  },
  "settings/project-and-application": {
    width: 760,
    height: 680,
    render: () => (
      <SettingsPage
        app={appSettings}
        updateApp={() => {}}
        project={project}
        state={{
          ...initialAgentState,
          connection: "ready",
          autoCompactionEnabled: true,
          steeringMode: "one-at-a-time",
          followUpMode: "one-at-a-time",
        }}
        updateProject={() => {}}
        close={() => {}}
        deleteProject={() => {}}
        setAutoCompaction={() => {}}
        setSteeringMode={() => {}}
        setFollowUpMode={() => {}}
      />
    ),
  },
});
