import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  getLayoutNode,
  layoutName,
  layoutRectRight,
  queryLayoutNodes,
} from "@wabou/test/layout";
import { renderAppLayout, renderLayoutFixtures } from "@wabou/test/layout/node";

const directory = await mkdtemp(join(tmpdir(), "wabou-pi-agent-layout-"));

function assertFullWorkbenchLayout(
  fixture: Parameters<typeof getLayoutNode>[0],
  viewportWidth: number,
): void {
  const shell = getLayoutNode(fixture, {
    role: "region",
    name: "Pi Agent workbench",
  });
  const main = getLayoutNode(fixture, {
    role: "region",
    name: "Conversation workspace",
  });
  const toolbar = getLayoutNode(fixture, {
    role: "toolbar",
    name: "Conversation actions",
  });
  const composer = getLayoutNode(fixture, {
    role: "group",
    name: "Ask this agent to work in its repository…",
  });
  const model = getLayoutNode(fixture, {
    role: "combobox",
    name: "Choose model",
  });
  const thinking = getLayoutNode(fixture, {
    role: "combobox",
    name: "Thinking level",
  });

  if (Math.abs(shell.rect.width - viewportWidth) > 0.5) {
    throw new Error(
      `workbench did not fill viewport: width=${shell.rect.width}, viewport=${viewportWidth}`,
    );
  }
  if (Math.abs(main.rect.x - 240) > 0.5) {
    throw new Error(
      `workbench sidebar contract drifted: main x=${main.rect.x}`,
    );
  }
  if (Math.abs(main.rect.width - (viewportWidth - 240)) > 0.5) {
    throw new Error(
      `main pane did not consume remaining width: width=${main.rect.width}`,
    );
  }
  const mainRight = main.rect.x + main.rect.width;
  for (const [name, node] of [
    ["conversation toolbar", toolbar],
    ["composer", composer],
    ["model control", model],
    ["thinking control", thinking],
  ] as const) {
    const right = node.rect.x + node.rect.width;
    if (node.rect.x < main.rect.x - 0.5 || right > mainRight + 0.5) {
      throw new Error(
        `${name} escaped main pane: x=${node.rect.x}, right=${right}, pane=${main.rect.x}..${mainRight}`,
      );
    }
  }
  if (model.rect.width < 176 || thinking.rect.width < 112) {
    throw new Error(
      `composer controls lost readable widths: model=${model.rect.width}, thinking=${thinking.rect.width}`,
    );
  }
}

try {
  const snapshot = await renderAppLayout({
    app: "apps/pi-agent",
    out: join(directory, "snapshot.json"),
    width: 1_200,
    height: 800,
    scaleFactor: 2,
    waitMs: 100,
    withHost: true,
    command: [resolve("target/release/wabou")],
  });

  const renderFailures = queryLayoutNodes(snapshot, {
    role: "alert",
    name: "Pi Agent failed to render",
  });
  if (renderFailures.length > 0) {
    throw new Error(
      `Pi Agent entered its root error boundary: ${renderFailures[0]?.text ?? "unknown error"}`,
    );
  }

  if (
    queryLayoutNodes(snapshot, { name: "Search agents and sessions" }).length >
    0
  ) {
    throw new Error("empty onboarding exposed an inactive session search");
  }
  getLayoutNode(snapshot, { text: "Start your first coding agent" });
  getLayoutNode(snapshot, { role: "textbox", name: "Workspace" });

  await renderLayoutFixtures({
    app: "apps/pi-agent",
    cases: [
      {
        id: "shell/full-workbench",
        width: 1_200,
        height: 800,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => assertFullWorkbenchLayout(fixture, 1_200),
      },
      {
        id: "shell/full-workbench-minimum",
        width: 720,
        height: 640,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => assertFullWorkbenchLayout(fixture, 720),
      },
      {
        id: "shell/content-column-wide",
        width: 1_200,
        height: 120,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          const column = getLayoutNode(fixture, {
            role: "region",
            name: "Readable workbench content",
          });
          if (Math.abs(column.rect.width - 1_024) > 0.5) {
            throw new Error(
              `workbench content column did not honor its readable width: width=${column.rect.width}`,
            );
          }
          if (Math.abs(column.rect.x - 88) > 0.5) {
            throw new Error(
              `workbench content column was not centered: x=${column.rect.x}`,
            );
          }
        },
      },
      {
        id: "conversation/composer-autocomplete",
        width: 384,
        height: 176,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "listbox", name: "Commands" });
          getLayoutNode(fixture, { role: "option", name: "/review" });
          getLayoutNode(fixture, {
            role: "option",
            name: "apps/pi-agent/ui/a-deliberately-long-workspace-file-name.tsx",
          });
        },
      },
      {
        id: "shell/agent-activity",
        width: 520,
        height: 48,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "status" });
          getLayoutNode(fixture, { text: "Retrying 2/3…" });
          getLayoutNode(fixture, { text: "3 queued" });
        },
      },
      {
        id: "shell/command-palette",
        width: 720,
        height: 520,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "dialog", name: "Command palette" });
          getLayoutNode(fixture, { role: "textbox", name: "Command palette" });
          getLayoutNode(fixture, { role: "option", name: "New session" });
        },
      },
      {
        id: "shell/conversation-context",
        width: 520,
        height: 48,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "group",
            name: "Documentation workspace, feat/readable-conversation-chrome, Explain why this renderer keeps the native boundary explicit",
          });
          getLayoutNode(fixture, { text: "…eadable-conversation-chrome" });
        },
      },
      {
        id: "shell/conversation-header",
        width: 480,
        height: 48,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "toolbar",
            name: "Conversation actions",
          });
          getLayoutNode(fixture, { role: "button", name: "Toggle terminal" });
          getLayoutNode(fixture, { role: "button", name: "Workspace files" });
          getLayoutNode(fixture, { role: "button", name: "Code changes" });
          getLayoutNode(fixture, { role: "button", name: "New session" });
        },
      },
      {
        id: "conversation/prompt-markdown",
        width: 560,
        height: 320,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "region", name: "User message" });
          getLayoutNode(fixture, { text: "Review request" });
        },
      },
      {
        id: "conversation/complete-turn",
        width: 680,
        height: 620,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "region", name: "User message" });
          const response = getLayoutNode(fixture, {
            role: "region",
            name: "Assistant response",
          });
          const activity = getLayoutNode(fixture, {
            role: "button",
            name: "Worked · 1 tool call",
          });
          const copy = getLayoutNode(fixture, {
            role: "button",
            name: "Copy assistant response",
          });
          if (response.rect.width < 600) {
            throw new Error(
              `assistant response did not fill the conversation column: width=${response.rect.width}`,
            );
          }
          const responseRight = response.rect.x + response.rect.width;
          const copyRight = copy.rect.x + copy.rect.width;
          if (Math.abs(copyRight - responseRight) > 16) {
            throw new Error(
              `assistant copy action detached from response end: response right=${responseRight}, copy right=${copyRight}`,
            );
          }
          if (Math.abs(activity.rect.x - response.rect.x) > 12) {
            throw new Error(
              `tool activity disclosure did not align with assistant prose: activity x=${activity.rect.x}, response x=${response.rect.x}`,
            );
          }
          if (
            queryLayoutNodes(fixture, {
              role: "list",
              name: "Recent tool activity",
            }).length > 0
          ) {
            throw new Error(
              "collapsed tool activity rendered duplicate details",
            );
          }
        },
      },
      {
        id: "conversation/composer-narrow",
        width: 420,
        height: 240,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          const editor = getLayoutNode(fixture, {
            role: "textbox",
            name: "Ask this agent to work in its repository…",
          });
          const usage = getLayoutNode(fixture, {
            role: "status",
            name: "Session usage",
          });
          const model = getLayoutNode(fixture, {
            role: "combobox",
            name: "Choose model",
          });
          const thinking = getLayoutNode(fixture, {
            role: "combobox",
            name: "Thinking level",
          });
          const hint = getLayoutNode(fixture, {
            text: "Wabou · Enter to send · Shift+Enter for newline",
          });
          if (model.rect.width < 176) {
            throw new Error(
              `model control lost its readable width: width=${model.rect.width}`,
            );
          }
          if (thinking.rect.width < 112) {
            throw new Error(
              `thinking control lost its readable width: width=${thinking.rect.width}`,
            );
          }
          if (usage.rect.y >= editor.rect.y || usage.rect.width < 120) {
            throw new Error(
              `session usage did not retain its composer status row: usage=${usage.rect.x},${usage.rect.y} ${usage.rect.width}x${usage.rect.height}; editor y=${editor.rect.y}`,
            );
          }
          const composer = getLayoutNode(fixture, {
            role: "group",
            name: "Ask this agent to work in its repository…",
          });
          if (
            hint.rect.x < composer.contentRect.x - 0.5 ||
            hint.rect.y < composer.contentRect.y - 0.5 ||
            hint.rect.x + hint.rect.width >
              composer.contentRect.x + composer.contentRect.width + 0.5 ||
            hint.rect.y + hint.rect.height >
              composer.contentRect.y + composer.contentRect.height + 0.5
          ) {
            throw new Error(
              `composer hint escaped its surface: hint=${hint.rect.x},${hint.rect.y} ${hint.rect.width}x${hint.rect.height}; composer=${composer.contentRect.x},${composer.contentRect.y} ${composer.contentRect.width}x${composer.contentRect.height}`,
            );
          }
          const thinkingLabel = getLayoutNode(fixture, { text: "medium" });
          const paintedWidth = thinkingLabel.textMetrics?.lineBox.width ?? 0;
          if (paintedWidth < 40) {
            throw new Error(
              `thinking value was visually truncated: painted width=${paintedWidth}`,
            );
          }
          getLayoutNode(fixture, { role: "button", name: "Send" });
        },
      },
      {
        id: "conversation/turn-navigator",
        width: 560,
        height: 420,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "group",
            name: "Conversation turns",
          });
          const turns = queryLayoutNodes(fixture, { role: "button" }).filter(
            (node) => layoutName(node).startsWith("Jump to turn "),
          );
          if (turns.length !== 12) {
            throw new Error(
              `expected 12 conversation turns, got ${turns.length}`,
            );
          }
          const current = turns.filter(
            (node) => new Map(node.attrs).get("aria-current") === "step",
          );
          if (current.length !== 1) {
            throw new Error(
              `expected one active conversation turn, got ${current.length}`,
            );
          }
        },
      },
      {
        id: "workspace/files-panel",
        width: 420,
        height: 720,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "region", name: "Workspace files" });
          getLayoutNode(fixture, {
            role: "textbox",
            name: "Search workspace files",
          });
        },
      },
      {
        id: "skills/catalog",
        width: 960,
        height: 720,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, { text: "Skills" });
          getLayoutNode(fixture, { role: "textbox", name: "Search skills" });
          getLayoutNode(fixture, {
            role: "button",
            name: "Frontend design review",
          });
          getLayoutNode(fixture, {
            role: "region",
            name: "Frontend design review",
          });
        },
      },
      {
        id: "skills/error",
        width: 720,
        height: 560,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "alert",
            name: "Could not load skills",
          });
          getLayoutNode(fixture, { role: "button", name: "Try again" });
        },
      },
      {
        id: "workspace/diff-viewer",
        width: 640,
        height: 520,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "textbox",
            name: "Technical diff: src/main.ts",
          });
        },
      },
      {
        id: "shell/sidebar",
        width: 300,
        height: 720,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          const search = getLayoutNode(fixture, {
            role: "textbox",
            name: "Search agents and sessions",
          });
          const newThread = getLayoutNode(fixture, {
            role: "button",
            name: "New thread",
          });
          const projects = getLayoutNode(fixture, {
            role: "region",
            name: "Projects",
          });
          const applicationTitle = getLayoutNode(fixture, {
            text: "Pi Agent",
          });
          if (newThread.rect.height < 36 || newThread.rect.width < 220) {
            throw new Error(
              `primary sidebar action lost its control surface: ${newThread.rect.width}x${newThread.rect.height}`,
            );
          }
          if (
            applicationTitle.computed.fontWeight !== 600 ||
            applicationTitle.rect.height < 20
          ) {
            throw new Error(
              `sidebar identity lost its title hierarchy: weight=${applicationTitle.computed.fontWeight}; height=${applicationTitle.rect.height}`,
            );
          }
          if (
            search.rect.x < newThread.rect.x - 0.5 ||
            search.rect.x + search.rect.width >
              newThread.rect.x + newThread.rect.width + 0.5
          ) {
            throw new Error(
              `sidebar search escaped the primary action column: search=${search.rect.x}..${search.rect.x + search.rect.width}; action=${newThread.rect.x}..${newThread.rect.x + newThread.rect.width}`,
            );
          }
          const searchBottom = search.rect.y + search.rect.height;
          if (projects.rect.y < searchBottom - 0.5) {
            throw new Error(
              `project scroll region overlaps fixed sidebar controls: projects y=${projects.rect.y}; search bottom=${searchBottom}`,
            );
          }
          getLayoutNode(fixture, {
            role: "button",
            name: "Documentation workspace",
          });
          getLayoutNode(fixture, {
            role: "button",
            name: "Polish the onboarding copy",
          });
          getLayoutNode(fixture, { text: "Today" });
          getLayoutNode(fixture, { text: "Previous 7 days" });
          getLayoutNode(fixture, { text: "Older" });
          getLayoutNode(fixture, {
            role: "status",
            name: "Compacting context…",
          });
        },
      },
      {
        id: "shell/model-controls",
        width: 480,
        height: 96,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "combobox", name: "Choose model" });
          getLayoutNode(fixture, { role: "combobox", name: "Thinking level" });
        },
      },
      {
        id: "settings/project-and-application",
        width: 760,
        height: 680,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, { text: "Settings" });
          getLayoutNode(fixture, { role: "textbox", name: "Project name" });
          getLayoutNode(fixture, { role: "textbox", name: "Workspace" });
          const tabs = queryLayoutNodes(fixture, { role: "tab" });
          if (tabs.length !== 2)
            throw new Error(
              `expected two settings scope tabs, found ${tabs.length}`,
            );
          getLayoutNode(fixture, { text: "Project settings" });
          getLayoutNode(fixture, { text: "Application settings" });
          getLayoutNode(fixture, { text: "Danger zone" });
        },
      },
      {
        id: "settings/application-defaults",
        width: 760,
        height: 680,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, { text: "Settings" });
          getLayoutNode(fixture, {
            role: "textbox",
            name: "Default provider",
          });
          getLayoutNode(fixture, {
            role: "textbox",
            name: "Default proxy URL",
          });
          getLayoutNode(fixture, { text: "Runtime" });
        },
      },
      {
        id: "settings/project-narrow",
        width: 480,
        height: 680,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          const section = getLayoutNode(fixture, {
            role: "group",
            name: "Workspace",
          });
          const name = getLayoutNode(fixture, {
            role: "textbox",
            name: "Project name",
          });
          if (name.rect.x > section.contentRect.x + 24) {
            throw new Error(
              `narrow settings kept a fixed label column: section x=${section.contentRect.x}, field x=${name.rect.x}`,
            );
          }
          if (name.rect.width < 300) {
            throw new Error(
              `narrow settings field lost usable width: ${name.rect.width}`,
            );
          }
        },
      },
      {
        id: "conversation/welcome-narrow",
        width: 480,
        height: 420,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, { text: "Build wabou with Pi" });
          const group = getLayoutNode(fixture, {
            role: "group",
            name: "Starter prompts",
          });
          const suggestions = queryLayoutNodes(fixture, { role: "button" });
          if (suggestions.length !== 3) {
            throw new Error(
              `expected three starter prompts, found ${suggestions.length}`,
            );
          }
          for (const suggestion of suggestions) {
            if (suggestion.rect.width < 176) {
              throw new Error(
                `starter prompt lost readable width: ${suggestion.rect.width}`,
              );
            }
            if (
              suggestion.rect.x < group.contentRect.x - 0.5 ||
              layoutRectRight(suggestion.rect) >
                layoutRectRight(group.contentRect) + 0.5
            ) {
              throw new Error("starter prompt escaped its responsive group");
            }
          }
        },
      },
    ],
    mode: "layout-test",
    command: [resolve("target/release/wabou")],
  });
} finally {
  await rm(directory, { recursive: true, force: true });
}
