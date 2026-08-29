import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  getLayoutNode,
  layoutName,
  queryLayoutNodes,
} from "@wabou/test/layout";
import { renderAppLayout, renderLayoutFixtures } from "@wabou/test/layout/node";

const directory = await mkdtemp(join(tmpdir(), "wabou-pi-agent-layout-"));

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
          getLayoutNode(fixture, {
            role: "button",
            name: "Worked, 1 tool call",
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
          if (Math.abs(copy.rect.x - response.rect.x) > 16) {
            throw new Error(
              `assistant copy action detached from response: response x=${response.rect.x}, copy x=${copy.rect.x}`,
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
          getLayoutNode(fixture, {
            role: "textbox",
            name: "Ask this agent to work in its repository…",
          });
          getLayoutNode(fixture, { role: "combobox", name: "Choose model" });
          getLayoutNode(fixture, { role: "combobox", name: "Thinking level" });
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
          getLayoutNode(fixture, {
            role: "textbox",
            name: "Search agents and sessions",
          });
          getLayoutNode(fixture, {
            role: "button",
            name: "Documentation workspace",
          });
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
        checks: ["visible-overflow", "text-collision"],
        assert: (fixture) => {
          getLayoutNode(fixture, { text: "Settings" });
          getLayoutNode(fixture, { role: "textbox", name: "Project name" });
          getLayoutNode(fixture, { role: "textbox", name: "Workspace" });
        },
      },
    ],
    mode: "layout-test",
    command: [resolve("target/release/wabou")],
  });
} finally {
  await rm(directory, { recursive: true, force: true });
}
