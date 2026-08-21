import { resolve } from "node:path";
import { getLayoutNode } from "@wabou/test/layout";
import { renderLayoutFixtures } from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/release/wabou")];
const selected = process.argv.slice(2).filter(Boolean);
const checks = ["visible-overflow", "sibling-collision"] as const;
const overrides = {
  // Carousel tracks and message reactions deliberately extend past their
  // logical content box; their component-specific clipping is tested lower.
  "widgets/Carousel": { checks: ["sibling-collision"] as const },
  "widgets/Message": { checks: ["sibling-collision"] as const },
};
const fixtureCase = (id: string) => {
  const override = overrides[id as keyof typeof overrides];
  return { id, checks: override?.checks ?? checks };
};

const report = await renderLayoutFixtures({
  app: "apps/gallery",
  mode: "layout-test",
  command,
  skipBuild: process.env.WABOU_LAYOUT_SKIP_BUILD === "1",
  cases: selected.length === 0 ? "all" : selected.map(fixtureCase),
  checks: selected.length === 0 ? checks : undefined,
  overrides: selected.length === 0 ? overrides : undefined,
});

const effect = report.cases.find(({ id }) => id === "effect/synchronous");
if (effect) getLayoutNode(effect.snapshot, { name: "effect status", text: "ready" });

const narrow = report.cases.find(({ id }) => id === "narrow");
const wide = report.cases.find(({ id }) => id === "wide");
if (narrow && wide) {
  if (
    narrow.snapshot.status.viewportWidth !== 640 ||
    narrow.snapshot.status.viewportHeight !== 480 ||
    wide.snapshot.status.viewportWidth !== 960 ||
    wide.snapshot.status.viewportHeight !== 720
  )
    throw new Error("fixture-owned viewport metadata was not applied");
  if (getLayoutNode(narrow.snapshot, { name: "narrow" }).rect.width !== 120)
    throw new Error("narrow fixture width was not evaluated by Taffy");
  if (getLayoutNode(wide.snapshot, { name: "wide" }).rect.width !== 320)
    throw new Error("wide fixture width was not evaluated by Taffy");
  for (const fixture of [narrow, wide]) {
    const ownerCount = fixture.snapshot.nodes.find(
      (node) => node.text === "1" && node.tag === "text",
    );
    if (!ownerCount)
      throw new Error(
        `fixture \`${fixture.id}\` leaked its preceding Solid owner`,
      );
  }
}

console.log(
  `[wabou] validated ${report.cases.length} component layout fixtures in ${report.totalDurationMs.toFixed(1)}ms`,
);
