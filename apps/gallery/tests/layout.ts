import { resolve } from "node:path";
import {
  assertLayoutRectContains,
  getLayoutNode,
  type LayoutSnapshot,
  layoutRectBottom,
  layoutRectRight,
  queryLayoutNodes,
} from "@wabou/test/layout";
import {
  type LayoutFixtureCase,
  renderLayoutFixtures,
} from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/release/wabou")];
const selected = process.argv.slice(2).filter(Boolean);
const checks = ["visible-overflow", "sibling-collision"] as const;

const assertClose = (actual: number, expected: number, label: string) => {
  if (Math.abs(actual - expected) > 1)
    throw new Error(`${label}: expected ${expected}, received ${actual}`);
};

const assertSidebarLayout = (snapshot: LayoutSnapshot) => {
  const boundary = getLayoutNode(snapshot, {
    name: "Sidebar fixture boundary",
  });
  const sidebar = getLayoutNode(snapshot, { name: "Fixture sidebar" });
  const navigation = getLayoutNode(snapshot, { name: "Fixture navigation" });
  const footer = getLayoutNode(snapshot, { name: "Fixture sidebar footer" });
  assertLayoutRectContains(boundary.contentRect, sidebar.rect, {
    label: "sidebar",
  });
  assertLayoutRectContains(sidebar.rect, navigation.rect, {
    label: "sidebar navigation",
  });
  assertLayoutRectContains(sidebar.rect, footer.rect, {
    label: "sidebar footer",
  });
  assertClose(layoutRectBottom(navigation.rect), footer.rect.y, "footer edge");
  assertClose(
    layoutRectBottom(footer.rect),
    layoutRectBottom(sidebar.rect),
    "sidebar bottom edge",
  );
  if (navigation.computed.overflowY !== "Scroll")
    throw new Error("sidebar content did not establish a scroll viewport");
};

const assertScrollAreaLayout = (snapshot: LayoutSnapshot) => {
  const viewport = getLayoutNode(snapshot, {
    role: "region",
    name: "Fixture scroll viewport",
  });
  const finalRow = getLayoutNode(snapshot, { text: "Scrollable row 12" });
  if (viewport.computed.overflowY !== "Scroll")
    throw new Error("ScrollArea did not retain overflow-y scrolling");
  if (layoutRectBottom(finalRow.rect) <= layoutRectBottom(viewport.contentRect))
    throw new Error("ScrollArea fixture did not produce a real scroll range");
};

const assertSelectLayout = (snapshot: LayoutSnapshot) => {
  const trigger = getLayoutNode(snapshot, {
    role: "combobox",
    name: "Fixture select",
  });
  const listbox = getLayoutNode(snapshot, {
    role: "listbox",
    name: "Fixture select",
  });
  assertLayoutRectContains(
    {
      x: 0,
      y: 0,
      width: snapshot.status.viewportWidth,
      height: snapshot.status.viewportHeight,
    },
    listbox.rect,
    { label: "select popup" },
  );
  if (listbox.rect.y < layoutRectBottom(trigger.rect))
    throw new Error("select popup overlaps its trigger");
  const selectedOption = queryLayoutNodes(snapshot, { role: "option" }).find(
    (node) =>
      node.attrs.some(
        ([name, value]) => name === "aria-selected" && value === "true",
      ),
  );
  if (!selectedOption)
    throw new Error("select popup did not expose its selected option");
};

const assertDialogLayout = (snapshot: LayoutSnapshot) => {
  const dialog = getLayoutNode(snapshot, {
    role: "dialog",
    name: "Fixture dialog",
  });
  const body = getLayoutNode(snapshot, { name: "Fixture dialog body" });
  const footer = getLayoutNode(snapshot, { name: "Fixture dialog footer" });
  assertLayoutRectContains(dialog.contentRect, body.rect, {
    label: "dialog scroll body",
  });
  assertLayoutRectContains(dialog.contentRect, footer.rect, {
    label: "dialog footer",
  });
  if (body.computed.overflowY !== "Scroll")
    throw new Error("dialog body did not establish an independent scroll area");
  if (layoutRectBottom(body.rect) > footer.rect.y)
    throw new Error("dialog body overlaps its fixed footer");
  assertClose(
    layoutRectBottom(footer.rect),
    layoutRectBottom(dialog.contentRect),
    "dialog footer bottom edge",
  );
};

const assertAdaptiveSplitPaneLayout = (snapshot: LayoutSnapshot) => {
  const boundary = getLayoutNode(snapshot, {
    name: "Adaptive split pane boundary",
  });
  const main = getLayoutNode(snapshot, { name: "Fixture split main" });
  const detail = getLayoutNode(snapshot, { name: "Fixture split detail" });
  assertLayoutRectContains(boundary.contentRect, main.rect, {
    label: "split pane main",
  });
  assertLayoutRectContains(boundary.contentRect, detail.rect, {
    label: "split pane detail",
  });
  assertClose(main.rect.y, detail.rect.y, "split pane top edge");
  assertClose(main.rect.height, detail.rect.height, "split pane height");
  assertClose(detail.rect.x - layoutRectRight(main.rect), 12, "split pane gap");
  assertClose(
    layoutRectRight(detail.rect),
    layoutRectRight(boundary.contentRect),
    "split pane right edge",
  );
};

const assertImageViewportLayout = (snapshot: LayoutSnapshot) => {
  const viewport = getLayoutNode(snapshot, { name: "Fixture image viewport" });
  const layer = getLayoutNode(snapshot, { name: "Fixture annotation layer" });
  const region = getLayoutNode(snapshot, { name: "Fixture speech region" });
  assertLayoutRectContains(viewport.contentRect, layer.rect, {
    label: "annotation layer",
  });
  assertLayoutRectContains(viewport.contentRect, region.rect, {
    label: "annotation region",
  });
  // The bordered 686×486 content box contains an 800×1200 image at 0.405 scale.
  assertClose(region.rect.width, 97.2, "projected annotation width");
  assertClose(region.rect.height, 72.9, "projected annotation height");
};

const assertImageListLayout = (snapshot: LayoutSnapshot) => {
  const list = getLayoutNode(snapshot, { name: "Fixture image list" });
  const first = getLayoutNode(snapshot, { name: "Fixture page 1" });
  assertLayoutRectContains(list.contentRect, first.rect, {
    label: "first image row",
  });
  assertClose(first.rect.height, 80, "virtual image row height");
};

const assertQrCodeLayout = (snapshot: LayoutSnapshot) => {
  const code = getLayoutNode(snapshot, { name: "Fixture QR code" });
  assertClose(code.rect.width, 196, "QR code width");
  assertClose(code.rect.height, 196, "QR code height");
  assertLayoutRectContains(
    {
      x: 0,
      y: 0,
      width: snapshot.status.viewportWidth,
      height: snapshot.status.viewportHeight,
    },
    code.rect,
    { label: "QR code" },
  );
};

const assertInputGroupLayout = (snapshot: LayoutSnapshot) => {
  const group = getLayoutNode(snapshot, { name: "Fixture input group" });
  const addon = getLayoutNode(snapshot, { name: "Fixture scheme addon" });
  const input = getLayoutNode(snapshot, { name: "Fixture hostname input" });
  const addonText = getLayoutNode(snapshot, { text: "https://" });
  assertClose(addon.rect.height, input.rect.height, "input group item height");
  assertLayoutRectContains(group.contentRect, addon.rect, {
    label: "input group addon",
  });
  assertLayoutRectContains(group.contentRect, input.rect, {
    label: "input group input",
  });
  if (!addonText.textMetrics || !input.textMetrics)
    throw new Error("InputGroup text metrics were not published");
  assertClose(
    addonText.textMetrics.baseline,
    input.textMetrics.baseline,
    "input group text baseline",
  );
};

const overrides: Readonly<Record<string, Omit<LayoutFixtureCase, "id">>> = {
  // Carousel tracks and message reactions deliberately extend past their
  // logical content box; their component-specific clipping is tested lower.
  "widgets/Carousel": { checks: ["sibling-collision"] as const },
  "widgets/Message": {
    checks: ["sibling-collision", "text-collision"] as const,
  },
  "component/Sidebar": { assert: assertSidebarLayout },
  "component/ScrollArea": { assert: assertScrollAreaLayout },
  "component/Select": { assert: assertSelectLayout },
  "component/Dialog": { assert: assertDialogLayout },
  "component/AdaptiveSplitPane": { assert: assertAdaptiveSplitPaneLayout },
  "component/ImageViewport": {
    width: 720,
    height: 520,
    assert: assertImageViewportLayout,
  },
  "component/ImageList": {
    width: 360,
    height: 360,
    assert: assertImageListLayout,
  },
  "component/QRCode": { assert: assertQrCodeLayout },
  "component/InputGroup": { assert: assertInputGroupLayout },
};
const fixtureCase = (id: string) => {
  const override = overrides[id];
  return { id, ...override, checks: override?.checks ?? checks };
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
if (effect)
  getLayoutNode(effect.snapshot, { name: "effect status", text: "ready" });

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
