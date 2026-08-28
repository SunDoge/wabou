import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { getLayoutNode, queryLayoutNodes } from "@wabou/test/layout";
import { renderAppLayout } from "@wabou/test/layout/node";

const directory = await mkdtemp(join(tmpdir(), "wabou-pi-agent-layout-"));

try {
  const snapshot = await renderAppLayout({
    app: "apps/pi-agent",
    out: join(directory, "snapshot.json"),
    width: 1_200,
    height: 800,
    scaleFactor: 2,
    waitMs: 100,
    skipBuild: process.env.WABOU_LAYOUT_SKIP_BUILD === "1",
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
} finally {
  await rm(directory, { recursive: true, force: true });
}
