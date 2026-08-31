import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  defineComponentFixtures,
  defineLayoutFixtures,
} from "@wabou/test/layout/fixtures";
import {
  DiffViewer,
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerViewport,
  MotionConfigProvider,
  Text,
  View,
  Workbench,
  WorkbenchContentColumn,
  WorkbenchHeader,
  WorkbenchMain,
} from "@wabou/ui";
import { createComponent, For } from "solid-js";
import { AgentActivityStatus } from "./agent-activity";
import { type AgentItem, initialAgentState } from "./agent-state";
import type { PiSkill } from "./api";
import { AppCommandPalette } from "./app-command-palette";
import { ComposerAutocompleteList } from "./composer-autocomplete-list";
import { ConversationItem, ConversationList } from "./conversation";
import { ConversationComposer } from "./conversation-composer";
import { ConversationContext } from "./conversation-context";
import { ConversationHeader } from "./conversation-header";
import { ConversationNavigator } from "./conversation-navigator";
import { ConversationWelcome } from "./conversation-welcome";
import { ConversationWorkspaceStatus } from "./conversation-workspace-status";
import { ExtensionUiDialog } from "./extension-ui";
import { ModelControls } from "./model-controls";
import { type AppSettings, SettingsPage } from "./settings";
import { Sidebar } from "./sidebar";
import { SkillsPage } from "./skills-page";
import { AgentTerminalPanel } from "./terminal-panel";
import { createAgentWorkspace } from "./workspace";
import { WorkspaceChangesPanel } from "./workspace-changes-panel";
import { WorkspacePanel } from "./workspace-panel";
import { WorkspaceSetup } from "./workspace-setup";

const project = createAgentWorkspace(1);
project.name = "Documentation workspace";
project.cwd = "/work/wabou/documentation-and-examples";
project.provider = "anthropic";
project.model = "claude-sonnet-4-5";

const secondProject = createAgentWorkspace(2);
secondProject.name = "Release automation";
secondProject.cwd = "/work/wabou/release-automation";
secondProject.state.connection = "running";
secondProject.state.activity = { kind: "compacting" };

const appSettings: AppSettings = {
  locale: "en",
  proxy: "http://127.0.0.1:7890",
  noProxy: "127.0.0.1,localhost",
  provider: "anthropic",
  model: "claude-sonnet-4-5",
  subagentsEnabled: true,
};

const longConversation: readonly AgentItem[] = Array.from(
  { length: 12 },
  (_, index) => ({
    id: `turn-${index + 1}`,
    kind: "user" as const,
    text: `Review step ${index + 1} and keep the native boundary explicit.`,
  }),
);

const completeTurn: readonly AgentItem[] = [
  {
    id: "request",
    kind: "user",
    text: "Review the current renderer and explain the highest-risk boundary before changing it.",
  },
  {
    id: "read",
    kind: "tool",
    name: "read",
    state: "success",
    input: JSON.stringify({
      path: "crates/wabou-runtime/src/applier/frame_source.rs",
    }),
    output: "Loaded the retained frame source and invalidation path.",
  },
  {
    id: "answer",
    kind: "assistant",
    text: "## Finding\n\nSemantic-only updates were not scheduling a projection frame. The fix keeps layout untouched and invalidates only the semantic projection.\n\n- No extra layout pass\n- Existing visual frame remains stable\n- Native semantics receive the completed update",
  },
];

function CompleteTurnFixture() {
  return <ConversationList items={completeTurn} />;
}

const shellSessions = [
  {
    agentId: project.id,
    sessionId: "session-one",
    sessionFile: "/tmp/session-one.jsonl",
    name: "Review the retained renderer boundary",
    cwd: project.cwd,
    updatedAt: 1_787_907_300,
  },
] as const;

const skillFixtures: readonly PiSkill[] = [
  {
    id: "project:frontend-review",
    name: "Frontend design review",
    description: "Review native application surfaces before implementation.",
    scope: "project",
    source: "shared",
    path: "/work/wabou/.agents/skills/frontend-design-review",
    content:
      "# Frontend design review\n\nStart from the primary task, then verify hierarchy, spacing, states, and layout resilience.\n\n## Evidence\n\n- Component behavior\n- Layout geometry\n- Native interaction",
  },
  {
    id: "user:release",
    name: "Release checklist",
    description: "Prepare a repeatable release without pushing automatically.",
    scope: "user",
    source: "pi",
    path: "/home/user/.pi/agent/skills/release",
    content:
      "# Release checklist\n\nRun verification and report the tag command.",
  },
];

function FullWorkbenchFixture() {
  const state = {
    ...initialAgentState,
    connection: "ready" as const,
    sessionId: "session-one",
    modelProvider: "anthropic",
    modelId: "claude-sonnet-4-5",
    thinking: "medium" as const,
    availableThinkingLevels: ["low", "medium", "high"] as const,
    models: [
      {
        provider: "anthropic",
        id: "claude-sonnet-4-5",
        name: "Claude Sonnet 4.5",
        reasoning: true,
        contextWindow: 200_000,
      },
    ],
    items: [
      {
        id: "request",
        kind: "user" as const,
        text: "Review the retained renderer boundary and explain the safest fix.",
      },
      {
        id: "read-runtime",
        kind: "tool" as const,
        name: "read",
        state: "success" as const,
        input: JSON.stringify({
          path: "crates/wabou-runtime/src/applier/frame_source.rs",
        }),
        output: "Loaded the frame source and invalidation path.",
        turnDurationMs: 8_400,
      },
      {
        id: "answer",
        kind: "assistant" as const,
        text: "## Renderer boundary\n\nSemantic-only updates were skipping projection even though the retained scene remained valid. The safest fix is to invalidate **semantic projection** without forcing another layout pass.\n\n- Keep the existing visual frame stable\n- Schedule one projection update\n- Preserve the native accessibility snapshot\n\nThe focused component and layout contracts now cover this path.",
      },
    ],
  };
  const activeProject = { ...project, state };
  return (
    <Workbench role="region" aria-label="Pi Agent workbench">
      <Sidebar
        agents={[activeProject]}
        initialGrouping="recent"
        sessions={shellSessions}
        activeId={project.id}
        select={() => {}}
        selectSession={() => {}}
        add={() => {}}
        newSession={() => {}}
        canCreateSession
        nowSeconds={1_787_907_600}
        openSettings={() => {}}
      />
      <WorkbenchMain role="region" aria-label="Conversation workspace">
        <ConversationHeader
          project={activeProject.name}
          branch="feat/layout-contract"
          session="Review the retained renderer boundary"
          state={state}
          cwdAvailable
          repository
          terminalOpen={false}
          filesOpen={false}
          changesOpen={false}
          searchOpen={false}
          toggleTerminal={() => {}}
          toggleFiles={() => {}}
          toggleChanges={() => {}}
          toggleSearch={() => {}}
          newSession={() => {}}
          compactSession={() => {}}
          cloneSession={() => {}}
          exportSession={() => {}}
          abort={() => {}}
        />
        <MessageScroller class="flex-1 min-h-0">
          <MessageScrollerViewport>
            <MessageScrollerContent>
              <WorkbenchContentColumn class="px-6 py-5">
                <ConversationList items={state.items} />
              </WorkbenchContentColumn>
            </MessageScrollerContent>
          </MessageScrollerViewport>
        </MessageScroller>
        <ConversationComposer
          connection="ready"
          project={activeProject.name}
          cwd={activeProject.cwd}
          branch="feat/conversation-status"
          repository
          draft=""
          images={[]}
          contextFiles={[]}
          deliveryMode="followUp"
          models={state.models}
          modelProvider={state.modelProvider}
          modelId={state.modelId}
          thinking={state.thinking}
          thinkingLevels={state.availableThinkingLevels}
          commands={[]}
          statuses={[]}
          widgets={[]}
          changeDraft={() => {}}
          changeImages={() => {}}
          changeContextFiles={() => {}}
          changeDeliveryMode={() => {}}
          chooseModel={() => {}}
          chooseThinking={() => {}}
          loadWorkspaceFiles={async () => []}
          submit={() => {}}
        />
      </WorkbenchMain>
    </Workbench>
  );
}

defineLayoutFixtures(
  defineComponentFixtures(
    {
      "shell/full-workbench": {
        width: 1_200,
        height: 800,
        render: FullWorkbenchFixture,
      },
      "shell/full-workbench-minimum": {
        width: 1_180,
        height: 680,
        render: FullWorkbenchFixture,
      },
      "shell/content-column-wide": {
        width: 1_200,
        height: 120,
        render: () => (
          <View class="w-full h-full bg-canvas">
            <WorkbenchContentColumn
              role="region"
              aria-label="Readable workbench content"
              class="max-w-5xl h-20 bg-surface"
            />
          </View>
        ),
      },
      "conversation/composer-autocomplete": {
        width: 384,
        height: 176,
        render: () => (
          <View class="w-full h-full bg-canvas p-3">
            <View class="w-full min-w-0 rounded-xl border border-subtle bg-input shadow-md p-1.5">
              <ComposerAutocompleteList
                label="Commands"
                highlighted="command:review"
                rows={[
                  {
                    kind: "command",
                    id: "command:review",
                    label: "/review",
                    description: "Review the current workspace changes",
                  },
                  {
                    kind: "file",
                    id: "file:long",
                    label:
                      "apps/pi-agent/ui/a-deliberately-long-workspace-file-name.tsx",
                    description: "Workspace file",
                  },
                ]}
                highlight={() => {}}
                choose={() => {}}
              />
            </View>
          </View>
        ),
      },
      "shell/agent-activity": {
        width: 520,
        height: 48,
        render: () => (
          <WorkbenchHeader class="w-full h-full bg-canvas justify-between">
            <View class="min-w-0 flex-1 overflow-hidden flex flex-row items-center gap-1">
              <Text class="min-w-0 flex-1 truncate text-sm">
                Documentation workspace with a deliberately long conversation
                title
              </Text>
              <AgentActivityStatus
                state={{
                  ...initialAgentState,
                  connection: "running",
                  activity: { kind: "retrying", attempt: 2, maxAttempts: 3 },
                  queue: { steering: 1, followUp: 2 },
                }}
              />
            </View>
            <View class="w-24 h-8 flex-none bg-control rounded-md" />
          </WorkbenchHeader>
        ),
      },
      "shell/conversation-context": {
        width: 520,
        height: 48,
        render: () => (
          <WorkbenchHeader class="w-full h-full bg-canvas justify-between">
            <ConversationContext
              project="Documentation workspace"
              branch="feat/readable-conversation-chrome"
              session="Explain why this renderer keeps the native boundary explicit"
              state={initialAgentState}
            />
            <View class="w-24 h-8 flex-none bg-control rounded-md" />
          </WorkbenchHeader>
        ),
      },
      "shell/conversation-header": {
        width: 480,
        height: 48,
        render: () => (
          <ConversationHeader
            project="Documentation workspace"
            branch="feature/readable-conversation-header-with-long-actions"
            session="Explain why this renderer keeps the native boundary explicit"
            state={{
              ...initialAgentState,
              connection: "ready",
              sessionId: "session-1",
              items: [
                { id: "request", kind: "user", text: "Inspect the renderer" },
              ],
            }}
            cwdAvailable
            repository
            terminalOpen={false}
            filesOpen={false}
            changesOpen={false}
            searchOpen={false}
            toggleTerminal={() => {}}
            toggleFiles={() => {}}
            toggleChanges={() => {}}
            toggleSearch={() => {}}
            newSession={() => {}}
            compactSession={() => {}}
            cloneSession={() => {}}
            exportSession={() => {}}
            abort={() => {}}
          />
        ),
      },
      "conversation/prompt-markdown": {
        width: 560,
        height: 320,
        render: () => (
          <View class="w-full h-full bg-canvas p-6">
            <ConversationItem
              animate={false}
              item={{
                id: "prompt-markdown",
                kind: "user",
                text: "# Review request\n\n- inspect `src/runtime.rs`\n- run **focused tests**\n\nKeep the native boundary explicit while fixing the long layout path.",
              }}
            />
          </View>
        ),
      },
      "conversation/complete-turn": {
        width: 680,
        height: 620,
        render: () => (
          <View class="w-full h-full bg-canvas p-5">
            <CompleteTurnFixture />
          </View>
        ),
      },
      "conversation/complete-turn-narrow": {
        width: 420,
        height: 700,
        render: () => (
          <View class="w-full h-full bg-canvas p-4">
            <CompleteTurnFixture />
          </View>
        ),
      },
      "conversation/turn-navigator": {
        width: 560,
        height: 420,
        render: () => (
          <View class="w-full h-full bg-canvas p-4">
            <MessageScroller>
              <MessageScrollerViewport>
                <MessageScrollerContent class="max-w-lg mx-auto px-5 py-4">
                  <For each={longConversation}>
                    {(item) => (
                      <MessageScrollerItem anchor={item.id}>
                        <ConversationItem item={item} animate={false} />
                      </MessageScrollerItem>
                    )}
                  </For>
                </MessageScrollerContent>
              </MessageScrollerViewport>
              <ConversationNavigator items={longConversation} />
            </MessageScroller>
          </View>
        ),
      },
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
            loadFiles={async () => [
              "README.md",
              "src/main.rs",
              "src/service.rs",
            ]}
            readFile={async (_cwd, path) => ({
              path,
              text: "# Workspace preview\n\nA real layout fixture for the file inspector.",
            })}
            addContext={() => {}}
            close={() => {}}
          />
        ),
      },
      "workspace/changes-panel": {
        width: 420,
        height: 720,
        render: () => (
          <WorkspaceChangesPanel
            cwd="/work/wabou"
            load={async () => ({
              files: [
                {
                  path: "src/renderer.ts",
                  status: "modified",
                  additions: 2,
                  deletions: 1,
                  patch: "@@ -1,2 +1,3 @@\n-old\n+new\n+line",
                },
              ],
            })}
            close={() => {}}
          />
        ),
      },
      "skills/catalog": {
        width: 960,
        height: 720,
        render: () => (
          <SkillsPage
            cwd={project.cwd}
            project={project.name}
            load={async () => skillFixtures}
            close={() => {}}
          />
        ),
      },
      "skills/catalog-narrow": {
        width: 520,
        height: 680,
        render: () => (
          <SkillsPage
            cwd={project.cwd}
            project={project.name}
            load={async () => skillFixtures}
            close={() => {}}
          />
        ),
      },
      "skills/error": {
        width: 720,
        height: 560,
        render: () => (
          <SkillsPage
            cwd={project.cwd}
            project={project.name}
            load={async () => {
              throw new Error("permission denied");
            }}
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
            initialGrouping="projects"
            sessions={[
              {
                agentId: project.id,
                sessionId: "session-one",
                sessionFile: "/tmp/session-one.jsonl",
                name: "Fix the persistent workspace resource loading loop",
                cwd: project.cwd,
                updatedAt: 1_787_907_300,
              },
              {
                agentId: project.id,
                sessionId: "session-two",
                sessionFile: "/tmp/session-two.jsonl",
                name: "Review release readiness",
                cwd: project.cwd,
                updatedAt: 1_787_648_400,
              },
              {
                agentId: project.id,
                sessionId: "session-older",
                sessionFile: "/tmp/session-older.jsonl",
                name: "Audit the retained renderer",
                cwd: project.cwd,
                updatedAt: 1_787_043_600,
              },
              {
                agentId: secondProject.id,
                sessionId: "session-docs",
                sessionFile: "/tmp/session-docs.jsonl",
                name: "Polish the onboarding copy",
                cwd: secondProject.cwd,
                updatedAt: 1_787_907_540,
              },
            ]}
            activeId={project.id}
            select={() => {}}
            selectSession={() => {}}
            add={() => {}}
            newSession={() => {}}
            canCreateSession
            nowSeconds={1_787_907_600}
            openSettings={() => {}}
          />
        ),
      },
      "shell/model-controls": {
        width: 480,
        height: 96,
        render: () => (
          <View class="w-full h-full flex items-center bg-canvas p-4">
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
          </View>
        ),
      },
      "terminal/panel": {
        width: 720,
        height: 256,
        render: () => (
          <View class="w-full h-full bg-canvas">
            <AgentTerminalPanel
              cwd="/work/wabou"
              open
              close={() => {}}
              dispose={() => {}}
            />
          </View>
        ),
      },
      "extension/select-dialog": {
        width: 640,
        height: 480,
        render: () => (
          <View class="w-full h-full bg-canvas">
            <ExtensionUiDialog
              request={{
                agentId: "agent-1",
                id: "environment",
                method: "select",
                title: "Choose an environment",
                message: "Select where the agent should run the next task.",
                options: ["Local workspace", "Development", "Production"],
              }}
              respond={() => {}}
            />
          </View>
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
            canDeleteProject
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
      "settings/application-defaults": {
        width: 760,
        height: 680,
        render: () => (
          <SettingsPage
            app={appSettings}
            updateApp={() => {}}
            project={project}
            canDeleteProject
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
            defaultSection="application"
          />
        ),
      },
      "settings/project-narrow": {
        width: 480,
        height: 680,
        render: () => (
          <SettingsPage
            app={appSettings}
            updateApp={() => {}}
            project={project}
            canDeleteProject
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
      "conversation/welcome-narrow": {
        width: 480,
        height: 420,
        render: () => (
          <View class="w-full h-full min-w-0 px-6 bg-canvas">
            <ConversationWelcome
              workspace="/work/wabou"
              choosePrompt={() => {}}
            />
          </View>
        ),
      },
      "conversation/workspace-status-narrow": {
        width: 480,
        height: 80,
        render: () => (
          <View class="w-full h-full min-w-0 px-3 bg-canvas flex items-center">
            <ConversationWorkspaceStatus
              project="Documentation and release workspace"
              branch="feat/native-runtime-projection"
              repository
              connection="running"
              runtimeLog="Applying a retained native editor update"
            />
          </View>
        ),
      },
      "workspace/setup": {
        width: 720,
        height: 620,
        render: () => (
          <View class="w-full h-full bg-canvas">
            <WorkspaceSetup
              path="/home/user/PiWorkspace"
              provider="anthropic"
              model="claude-sonnet-4-5"
              proxy="http://127.0.0.1:7890"
              updatePath={() => {}}
              start={async () => {}}
              openSettings={() => {}}
            />
          </View>
        ),
      },
      "workspace/setup-error": {
        width: 620,
        height: 560,
        render: () => (
          <View class="w-full h-full bg-canvas">
            <WorkspaceSetup
              path="/home/user/PiWorkspace/projects/a-repository-with-a-long-readable-name"
              provider="openai-codex"
              model="gpt-5.6-codex"
              proxy="http://127.0.0.1:7890"
              error="The agent runtime could not start. Your workspace and settings were preserved; review the output below and try again."
              runtimeLogs={[
                "Starting the Pi runtime…",
                "Loading workspace configuration…",
                "Runtime exited before the session became ready.",
              ]}
              updatePath={() => {}}
              start={async () => {}}
              openSettings={() => {}}
            />
          </View>
        ),
      },
    },
    {
      wrap: (render) =>
        createComponent(MotionConfigProvider, {
          reducedMotion: true,
          get children() {
            return render();
          },
        }),
    },
  ),
);
