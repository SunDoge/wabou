import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  assertNoLayoutDiagnostics,
  getLayoutNode,
  siblingCollisionDiagnostics,
  styleDiagnostics,
  textCollisionDiagnostics,
  visibleOverflowDiagnostics,
  visualQualityDiagnostics,
} from "@wabou/test/layout";
import { renderAppLayout } from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/debug/wabou-render")];

const directory = await mkdtemp(join(tmpdir(), "wabou-reader-layout-"));
try {
  for (const [index, viewport] of [
    { id: "normal", width: 1440, height: 900 },
    { id: "minimum", width: 980, height: 640 },
  ].entries()) {
    const snapshot = await renderAppLayout({
      app: "apps/reader",
      out: join(directory, `${viewport.id}.json`),
      width: viewport.width,
      height: viewport.height,
      skipBuild: index > 0,
      command,
    });
    assertNoLayoutDiagnostics([
      ...styleDiagnostics(snapshot),
      ...visibleOverflowDiagnostics(snapshot),
      ...siblingCollisionDiagnostics(snapshot),
      ...textCollisionDiagnostics(snapshot),
      ...visualQualityDiagnostics(snapshot),
    ]);

    const fieldNote = getLayoutNode(snapshot, {
      text: "WABOU READER · FIELD NOTE 01",
    });
    const notes = getLayoutNode(snapshot, { text: "Notes" });
    if (fieldNote.rect.x < 256 || fieldNote.rect.x >= notes.rect.x)
      throw new Error(`${viewport.id}: reading page lost its center pane`);
    if (notes.rect.x + notes.rect.width > viewport.width)
      throw new Error(`${viewport.id}: notes inspector escaped the window`);
  }
} finally {
  await rm(directory, { recursive: true, force: true });
}
