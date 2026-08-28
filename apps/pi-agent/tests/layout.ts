import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { getLayoutNode, queryLayoutNodes } from "@wabou/test/layout";
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

  getLayoutNode(snapshot, { name: "Search agents and sessions" });
  getLayoutNode(snapshot, { text: "Choose a project" });
  getLayoutNode(snapshot, { role: "textbox", name: "Workspace" });

  await renderLayoutFixtures({
    app: "apps/pi-agent",
    cases: [
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
    ],
    mode: "layout-test",
    command: [resolve("target/release/wabou")],
  });
} finally {
  await rm(directory, { recursive: true, force: true });
}
