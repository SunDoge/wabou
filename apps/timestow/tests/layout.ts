import { resolve } from "node:path";
import { getLayoutNode } from "@wabou/test/layout";
import {
  type LayoutFixtureCase,
  renderLayoutFixtures,
} from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/release/wabou")];
const selected = process.argv.slice(2).filter(Boolean);
const selectCases = (...cases: LayoutFixtureCase[]): LayoutFixtureCase[] =>
  selected.length === 0
    ? cases
    : cases.filter(({ id }) => selected.includes(id));

function assertSetupWorkspace(
  fixture: Parameters<typeof getLayoutNode>[0],
  viewportWidth: number,
  viewportHeight: number,
): void {
  const shell = getLayoutNode(fixture, {
    role: "region",
    name: "Timestow setup workspace",
  });
  const navigation = getLayoutNode(fixture, {
    role: "group",
    name: "Primary navigation",
  });
  const form = getLayoutNode(fixture, {
    role: "group",
    name: "Backup repository setup",
  });
  const submit = getLayoutNode(fixture, {
    role: "button",
    name: "Create backup",
  });

  if (Math.abs(shell.rect.width - viewportWidth) > 0.5) {
    throw new Error(`setup shell width drifted: ${shell.rect.width}`);
  }
  if (Math.abs(navigation.rect.width - 224) > 0.5) {
    throw new Error(`profile sidebar width drifted: ${navigation.rect.width}`);
  }
  if (form.rect.x < navigation.rect.width - 0.5) {
    throw new Error(
      `setup form overlaps the profile sidebar: x=${form.rect.x}`,
    );
  }
  if (form.rect.x + form.rect.width > viewportWidth + 0.5) {
    throw new Error("setup form escapes the horizontal viewport");
  }
  if (form.rect.y + form.rect.height > viewportHeight + 0.5) {
    throw new Error("setup form escapes the declared minimum viewport");
  }
  if (submit.rect.x + submit.rect.width > form.rect.x + form.rect.width + 0.5) {
    throw new Error("primary setup action escapes its form surface");
  }
}

await renderLayoutFixtures({
  app: "apps/timestow",
  command,
  cases: selectCases(
    {
      id: "timestow/setup-wide",
      width: 1_200,
      height: 760,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => assertSetupWorkspace(fixture, 1_200, 760),
    },
    {
      id: "timestow/setup-minimum",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => assertSetupWorkspace(fixture, 900, 620),
    },
  ),
});
