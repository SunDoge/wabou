import { resolve } from "node:path";
import { getLayoutNode } from "@wabou/test/layout";
import {
  renderLayoutFixtures,
  validateLayoutFixtureReport,
} from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/release/wabou")];

const report = await renderLayoutFixtures({
  app: "apps/gallery",
  mode: "layout-test",
  command,
  cases: "all",
});

const intentionalVisibleOverflow = new Set([
  // Carousel tracks and message reactions deliberately extend past their
  // logical content box; their component-specific clipping is tested lower.
  "widgets/Carousel",
  "widgets/Message",
]);
await validateLayoutFixtureReport(
  report,
  report.cases.map(({ id }) => ({
    id,
    checks: intentionalVisibleOverflow.has(id)
      ? (["sibling-collision"] as const)
      : (["visible-overflow", "sibling-collision"] as const),
  })),
);

const effect = report.cases.find(({ id }) => id === "effect/synchronous");
if (!effect) throw new Error("missing synchronous effect fixture");
getLayoutNode(effect.snapshot, { name: "effect status", text: "ready" });

const narrow = report.cases.find(({ id }) => id === "narrow");
const wide = report.cases.find(({ id }) => id === "wide");
if (!narrow || !wide) throw new Error("missing isolation fixtures");
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

console.log(
  `[wabou] validated ${report.cases.length} component layout fixtures`,
);
