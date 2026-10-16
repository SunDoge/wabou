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
const cases = [
  { id: "target", width: 1440, height: 868 },
  { id: "minimum", width: 960, height: 640 },
];

const directory = await mkdtemp(
  join(tmpdir(), "wabou-gpui-benchmarks-layout-"),
);
try {
  for (const [index, fixture] of cases.entries()) {
    const snapshot = await renderAppLayout({
      app: "apps/gpui-benchmarks",
      out: join(directory, `${fixture.id}.json`),
      width: fixture.width,
      height: fixture.height,
      waitMs: 250,
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

    const applications = getLayoutNode(snapshot, {
      role: "list",
      name: "Benchmark applications",
    });
    const cards = queryLayoutNodes(snapshot, { role: "listitem" });
    for (const card of cards) {
      assertLayoutRectContains(applications.rect, card.rect, {
        label: `${fixture.id} ${card.semantic?.label ?? "application"} card`,
      });
    }
    if (cards.length !== 7)
      throw new Error(`${fixture.id}: expected seven application routes`);
    if (queryLayoutNodes(snapshot, { role: "button" }).length !== 7)
      throw new Error(`${fixture.id}: expected seven route controls`);
    if (fixture.id === "target") {
      const firstRow = cards.slice(0, 3);
      if (
        firstRow.length !== 3 ||
        firstRow.some((card) => card.rect.width < 280)
      )
        throw new Error("target viewport lost its three-column route grid");
      const topEdges = firstRow.map((card) => card.rect.y);
      if (Math.max(...topEdges) - Math.min(...topEdges) > 1)
        throw new Error("route cards lost their shared top edge");
    }
  }
} finally {
  await rm(directory, { recursive: true, force: true });
}
