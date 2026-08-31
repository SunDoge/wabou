import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  getLayoutNode,
  layoutName,
  layoutRectRight,
  queryLayoutNodes,
} from "@wabou/test/layout";
import {
  type LayoutFixtureCase,
  renderLayoutFixtures,
} from "@wabou/test/layout/node";

const directory = await mkdtemp(join(tmpdir(), "wabou-pi-agent-layout-"));
const selected = process.argv.slice(2).filter(Boolean);
const selectCases = (...cases: LayoutFixtureCase[]): LayoutFixtureCase[] =>
  selected.length === 0
    ? cases
    : cases.filter(({ id }) => selected.includes(id));

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
  const editor = getLayoutNode(fixture, {
    role: "textbox",
    name: "Ask this agent to work in its repository…",
  });
  const model = getLayoutNode(fixture, {
    role: "button",
    name: "Choose model",
  });
  const activeSession = getLayoutNode(fixture, {
    role: "button",
    name: "Review the retained renderer boundary",
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
  ] as const) {
    const right = node.rect.x + node.rect.width;
    if (node.rect.x < main.rect.x - 0.5 || right > mainRight + 0.5) {
      throw new Error(
        `${name} escaped main pane: x=${node.rect.x}, right=${right}, pane=${main.rect.x}..${mainRight}`,
      );
    }
  }
  if (model.rect.width > 224) {
    throw new Error(
      `composer model summary consumed too much primary toolbar space: width=${model.rect.width}`,
    );
  }
  // The native shell prevents the workbench from becoming narrower than its
  // declared application minimum, so the primary composer remains one row.
  // Measure authored content rather than counting the surface border.
  const compactContentLimit = 112;
  if (
    editor.rect.height > 48 ||
    composer.contentRect.height > compactContentLimit
  ) {
    throw new Error(
      `empty workbench composer lost its compact density: editor=${editor.rect.height}, content=${composer.contentRect.height}, limit=${compactContentLimit}`,
    );
  }
  if (
    !activeSession.attrs.some(
      ([name, value]) => name === "aria-selected" && value === "true",
    )
  ) {
    throw new Error("full workbench fixture did not expose its active session");
  }
  getLayoutNode(fixture, { text: "Renderer boundary" });
}

try {
  await renderLayoutFixtures({
    app: "apps/pi-agent",
    cases: selectCases(
      {
        id: "shell/full-workbench",
        width: 1_200,
        height: 800,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => assertFullWorkbenchLayout(fixture, 1_200),
      },
      {
        id: "shell/full-workbench-minimum",
        width: 1_180,
        height: 680,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => assertFullWorkbenchLayout(fixture, 1_180),
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
          getLayoutNode(fixture, {
            role: "textbox",
            name: "Command palette",
          });
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
          const navigation = getLayoutNode(fixture, {
            role: "toolbar",
            name: "Session navigation",
          });
          const actions = getLayoutNode(fixture, {
            role: "toolbar",
            name: "Conversation actions",
          });
          const actionButtons = [
            "Toggle terminal",
            "Workspace files",
            "Code changes",
            "New session",
            "Session actions",
          ].map((name) => getLayoutNode(fixture, { role: "button", name }));
          if (
            actionButtons.some(
              (button) =>
                button.rect.y !== actionButtons[0]?.rect.y ||
                button.rect.height !== actionButtons[0]?.rect.height,
            )
          ) {
            throw new Error(
              "conversation toolbar controls lost shared geometry",
            );
          }
          if (navigation.rect.x + navigation.rect.width > actions.rect.x) {
            throw new Error(
              "session navigation overlapped conversation actions",
            );
          }
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
            name: "Toggle tool activity",
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
          const activityCenter = activity.rect.x + activity.rect.width / 2;
          const responseCenter = response.rect.x + response.rect.width / 2;
          if (Math.abs(activityCenter - responseCenter) > 12) {
            throw new Error(
              `tool activity did not center on the conversation column: activity center=${activityCenter}, response center=${responseCenter}`,
            );
          }
          const activityContent = fixture.nodes.find(
            (node) =>
              node.id.lo === activity.parentId?.lo &&
              node.id.hi === activity.parentId?.hi,
          );
          const activityBoundary = fixture.nodes.find(
            (node) =>
              node.id.lo === activityContent?.parentId?.lo &&
              node.id.hi === activityContent?.parentId?.hi,
          );
          if (!activityBoundary) {
            throw new Error(
              "tool activity lost its labeled separator boundary",
            );
          }
          const rules = fixture.nodes.filter(
            (node) =>
              node.parentId?.lo === activityBoundary.id.lo &&
              node.parentId?.hi === activityBoundary.id.hi &&
              node.classes.includes("h-px"),
          );
          if (
            rules.length !== 2 ||
            rules.some((rule) => rule.rect.width < 80)
          ) {
            throw new Error(
              `tool activity boundary must retain two visible rules: ${rules.map((rule) => rule.rect.width).join(", ")}`,
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
        id: "conversation/turn-navigator",
        width: 560,
        height: 420,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "toolbar",
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
        id: "conversation/workspace-status-narrow",
        width: 480,
        height: 80,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          const status = getLayoutNode(fixture, {
            role: "status",
            name: "Workspace status",
          });
          if (layoutRectRight(status.rect) > 480.5) {
            throw new Error(
              `workspace status escaped viewport: right=${layoutRectRight(status.rect)}`,
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
          getLayoutNode(fixture, {
            role: "listbox",
            name: "Workspace files",
          });
          const file = getLayoutNode(fixture, {
            role: "option",
            name: "src/main.rs",
          });
          if (Math.abs(file.rect.height - 48) > 0.5) {
            throw new Error(
              `workspace file row lost its native target height: height=${file.rect.height}`,
            );
          }
        },
      },
      {
        id: "workspace/files-panel-error",
        width: 360,
        height: 520,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "alert",
            name: "Could not load the file preview",
          });
        },
      },
      {
        id: "workspace/changes-panel",
        width: 420,
        height: 720,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "region", name: "Code changes" });
          getLayoutNode(fixture, { text: "src/renderer.ts" });
        },
      },
      {
        id: "workspace/changes-panel-error",
        width: 360,
        height: 520,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "alert",
            name: "Could not load code changes",
          });
          const retry = queryLayoutNodes(fixture, {
            role: "button",
            name: "Try again",
          });
          if (retry.length !== 1) {
            const buttons = queryLayoutNodes(fixture, { role: "button" }).map(
              (node) => ({ text: node.text, attrs: node.attrs }),
            );
            throw new Error(
              `retry action missing from error state: ${JSON.stringify(buttons)}`,
            );
          }
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
        id: "skills/catalog-narrow",
        width: 520,
        height: 680,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "textbox", name: "Search skills" });
          getLayoutNode(fixture, {
            role: "button",
            name: "Frontend design review",
          });
          if (
            queryLayoutNodes(fixture, {
              role: "region",
              name: "Frontend design review",
            }).length > 0
          ) {
            throw new Error(
              "compact skills page opened detail before selection",
            );
          }
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
          const searchSurface = fixture.nodes.find(
            (node) =>
              node.id.lo === search.parentId?.lo &&
              node.id.hi === search.parentId?.hi,
          );
          if (!searchSurface)
            throw new Error("sidebar search surface is missing");
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
            !newThread.classes.includes("bg-transparent") ||
            !searchSurface.classes.includes("bg-transparent") ||
            !searchSurface.classes.includes("border-transparent")
          ) {
            throw new Error(
              `sidebar chrome must stay quiet until interaction: action=${newThread.classes.join(" ")}; search=${searchSurface.classes.join(" ")}`,
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
          const activeProject = getLayoutNode(fixture, {
            role: "button",
            name: "Documentation workspace",
          });
          if (
            !activeProject.classes.includes("bg-selected") ||
            activeProject.classes.includes("shadow-xs")
          ) {
            throw new Error(
              `selected sidebar destinations must use a flat selection fill: ${activeProject.classes.join(" ")}`,
            );
          }
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
          const model = getLayoutNode(fixture, {
            role: "combobox",
            name: "Choose model",
          });
          const thinking = getLayoutNode(fixture, {
            role: "combobox",
            name: "Thinking level",
          });
          for (const trigger of [model, thinking]) {
            if (Math.abs(trigger.rect.height - 32) > 0.5) {
              throw new Error(
                `header picker must use compact 32px geometry; got ${trigger.rect.height}px`,
              );
            }
            if (!trigger.classes.includes("bg-transparent")) {
              throw new Error(
                `header picker must use the quiet ghost surface; classes=${trigger.classes.join(" ")}`,
              );
            }
          }
        },
      },
      {
        id: "terminal/panel",
        width: 720,
        height: 256,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { role: "region", name: "Terminal panel" });
          getLayoutNode(fixture, {
            role: "tablist",
            name: "Terminal sessions",
          });
          getLayoutNode(fixture, { role: "textbox", name: "Terminal 1" });
        },
      },
      {
        id: "extension/select-dialog",
        width: 640,
        height: 480,
        waitMs: 20,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            role: "dialog",
            name: "Choose an environment",
          });
          getLayoutNode(fixture, {
            role: "listbox",
            name: "Choose an environment",
          });
          getLayoutNode(fixture, { role: "option", name: "Development" });
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
          const language = getLayoutNode(fixture, {
            role: "radiogroup",
            name: "Language",
          });
          const languageOptions = queryLayoutNodes(fixture, { role: "radio" });
          if (languageOptions.length !== 2) {
            throw new Error(
              `expected two segmented language choices, found ${languageOptions.length}`,
            );
          }
          if (
            languageOptions.some(
              (option) =>
                option.rect.height !== languageOptions[0]?.rect.height ||
                option.rect.y !== languageOptions[0]?.rect.y,
            )
          ) {
            throw new Error("segmented language choices lost shared geometry");
          }
          if (
            languageOptions.some(
              (option) =>
                option.rect.x < language.rect.x ||
                option.rect.x + option.rect.width >
                  language.rect.x + language.rect.width,
            )
          ) {
            throw new Error("segmented language choice escaped its group");
          }
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
          const widths = new Set(
            suggestions.map((suggestion) => suggestion.rect.width.toFixed(1)),
          );
          const columns = new Set(
            suggestions.map((suggestion) => suggestion.rect.x.toFixed(1)),
          );
          if (widths.size !== 1 || columns.size !== 1) {
            throw new Error(
              `narrow starter prompts should form one balanced column: widths=${[...widths].join(",")}; x=${[...columns].join(",")}`,
            );
          }
        },
      },
      {
        id: "workspace/setup",
        width: 720,
        height: 620,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          const workspace = getLayoutNode(fixture, {
            role: "textbox",
            name: "Workspace",
          });
          const settings = getLayoutNode(fixture, {
            role: "button",
            name: "Review settings",
          });
          const start = getLayoutNode(fixture, {
            role: "button",
            name: "Start agent",
          });
          if (workspace.rect.width < 480) {
            throw new Error(
              `workspace picker lost readable width: ${workspace.rect.width}`,
            );
          }
          if (settings.rect.y >= start.rect.y) {
            throw new Error(
              "setup settings summary no longer precedes the primary action",
            );
          }
          if (start.rect.width < 480) {
            throw new Error(
              `setup primary action lost its full-width hierarchy: ${start.rect.width}`,
            );
          }
        },
      },
      {
        id: "workspace/setup-error",
        width: 620,
        height: 560,
        checks: ["visible-overflow", "text-collision", "visual-quality"],
        assert: (fixture) => {
          getLayoutNode(fixture, {
            text: "The agent runtime could not start. Your workspace and settings were preserved; review the output below and try again.",
          });
          const output = getLayoutNode(fixture, {
            role: "group",
            name: "Runtime output",
          });
          const start = getLayoutNode(fixture, {
            role: "button",
            name: "Start agent",
          });
          if (output.rect.y <= start.rect.y + start.rect.height) {
            throw new Error("runtime diagnostics overlapped the retry action");
          }
          if (start.rect.width < 420) {
            throw new Error(
              `setup retry action lost usable width: ${start.rect.width}`,
            );
          }
        },
      },
    ),
    mode: "layout-test",
    skipBuild: process.env.WABOU_LAYOUT_SKIP_BUILD === "1",
    command: [resolve("target/release/wabou")],
  });
} finally {
  await rm(directory, { recursive: true, force: true });
}
