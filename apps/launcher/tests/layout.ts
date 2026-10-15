import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  assertLayoutRectContains,
  assertNoLayoutDiagnostics,
  getLayoutNode,
  queryLayoutNodes,
  siblingCollisionDiagnostics,
  styleDiagnostics,
  textCollisionDiagnostics,
  visibleOverflowDiagnostics,
  visualQualityDiagnostics,
} from "@wabou/test/layout";
import { renderAppLayout } from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/release/wabou-render")];
const directory = await mkdtemp(join(tmpdir(), "wabou-launcher-layout-"));

try {
  const snapshot = await renderAppLayout({
    app: "apps/launcher",
    out: join(directory, "launcher.json"),
    width: 720,
    height: 500,
    mode: "layout-test",
    command,
  });
  assertNoLayoutDiagnostics([
    ...styleDiagnostics(snapshot),
    ...visibleOverflowDiagnostics(snapshot),
    ...siblingCollisionDiagnostics(snapshot),
    ...textCollisionDiagnostics(snapshot),
    ...visualQualityDiagnostics(snapshot),
  ]);

  const dialog = getLayoutNode(snapshot, {
    role: "dialog",
    name: "Application launcher",
  });
  const search = getLayoutNode(snapshot, {
    role: "textbox",
    name: "Search applications",
  });
  const results = getLayoutNode(snapshot, {
    role: "listbox",
    name: "Application results",
  });
  assertLayoutRectContains(dialog.rect, search.rect, { label: "search field" });
  assertLayoutRectContains(dialog.rect, results.rect, { label: "result list" });
  const options = queryLayoutNodes(snapshot, { role: "option" });
  if (options.length !== 5)
    throw new Error(
      `launcher projected ${options.length} results instead of 5`,
    );
} finally {
  await rm(directory, { recursive: true, force: true });
}
