import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  assertLayoutRectContains,
  assertNoLayoutDiagnostics,
  getLayoutNode,
  layoutName,
  layoutRectBottom,
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
  : [resolve("target/release/wabou")];
const selected = process.argv.slice(2).filter(Boolean);
const cases = [
  { id: "meow-starter/overview-normal", width: 1_000, height: 680 },
  { id: "meow-starter/overview-minimum", width: 820, height: 560 },
].filter(({ id }) => selected.length === 0 || selected.includes(id));

if (cases.length === 0)
  throw new Error("no matching Meow Starter layout cases");

const directory = await mkdtemp(join(tmpdir(), "wabou-meow-layout-"));
try {
  for (const [index, fixture] of cases.entries()) {
    const snapshot = await renderAppLayout({
      app: "apps/meow-starter",
      out: join(directory, `${index}.json`),
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

    const root = getLayoutNode(snapshot, {
      role: "group",
      name: "Primary navigation",
    });
    const overview = getLayoutNode(snapshot, {
      role: "group",
      name: "Meow Starter overview",
    });
    const capabilities = getLayoutNode(snapshot, {
      role: "group",
      name: "Runtime capabilities",
    });
    const tasks = getLayoutNode(snapshot, {
      role: "group",
      name: "Starter tasks",
    });
    const quickInfo = getLayoutNode(snapshot, {
      role: "group",
      name: "Quick information",
    });
    if (Math.abs(root.rect.width - 216) > 1)
      throw new Error(
        `${fixture.id}: sidebar width drifted to ${root.rect.width}`,
      );
    assertLayoutRectContains(overview.rect, capabilities.rect, {
      label: `${fixture.id} capability grid`,
    });
    if (queryLayoutNodes(snapshot, { text: "meow-starter" }).length < 2)
      throw new Error(`${fixture.id}: brand hierarchy is incomplete`);
    if (queryLayoutNodes(snapshot, { role: "button" }).length < 14)
      throw new Error(`${fixture.id}: interactive controls were not projected`);

    if (fixture.width === 1_000) {
      if (
        queryLayoutNodes(snapshot, { role: "group" }).filter((node) =>
          layoutName(node).endsWith(" capability"),
        ).length !== 4
      )
        throw new Error(
          "normal overview did not project four capability cards",
        );
      if (Math.abs(tasks.rect.y - quickInfo.rect.y) > 1)
        throw new Error(
          "normal overview lower panels lost their shared top edge",
        );
      if (Math.abs(tasks.rect.height - quickInfo.rect.height) > 1)
        throw new Error("normal overview lower panels lost equal height");
      if (layoutRectBottom(tasks.rect) > fixture.height - 12)
        throw new Error("normal overview no longer fits in the initial window");
    } else {
      if (capabilities.rect.width > overview.rect.width + 1)
        throw new Error("minimum capability grid escaped the content column");
      if (tasks.rect.width < 300 || quickInfo.rect.width < 300)
        throw new Error(
          "minimum lower panels compressed below their readable width",
        );
    }
  }
} finally {
  await rm(directory, { recursive: true, force: true });
}
