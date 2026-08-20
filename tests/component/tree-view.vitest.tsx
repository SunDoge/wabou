import { renderComponent } from "@wabou/test/component";
import { createTreeModel, type TreeNode, TreeView } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const items: readonly TreeNode[] = [
  {
    id: "src",
    label: "Source",
    children: [
      { id: "components", label: "Components" },
      { id: "runtime", label: "Runtime" },
    ],
  },
  { id: "tests", label: "Tests" },
  { id: "target", label: "Build output", disabled: true },
];

test("validates identity and flattens only expanded branches", () => {
  const model = createTreeModel(items);
  expect(model.visible([]).map(({ node }) => node.id)).toEqual([
    "src",
    "tests",
    "target",
  ]);
  expect(
    model.visible(["src"]).map(({ node, level }) => [node.id, level]),
  ).toEqual([
    ["src", 1],
    ["components", 2],
    ["runtime", 2],
    ["tests", 1],
    ["target", 1],
  ]);
  expect(model.parent("runtime")).toBe("src");
  expect(() =>
    createTreeModel([
      { id: "same", label: "One" },
      { id: "same", label: "Two" },
    ]),
  ).toThrow("tree node id must be unique: same");
});

test("expands, selects, and skips disabled items through native focus routing", () => {
  const screen = renderComponent(() => (
    <TreeView items={items} aria-label="Project files" />
  ));
  const source = screen.getByRole("treeitem", { name: "Source" });

  expect(source.attribute("aria-expanded")).toBe("false");
  source.focus();
  source.press("ArrowRight");
  expect(screen.queryByRole("treeitem", { name: "Components" })).not.toBeNull();
  expect(source.attribute("aria-expanded")).toBe("true");

  source.press("ArrowRight");
  const components = screen.getByRole("treeitem", { name: "Components" });
  expect(components.focused).toBe(true);
  components.press("ArrowDown");
  const runtime = screen.getByRole("treeitem", { name: "Runtime" });
  expect(runtime.focused).toBe(true);
  runtime.press("End");
  expect(screen.getByRole("treeitem", { name: "Tests" }).focused).toBe(true);

  runtime.click();
  expect(runtime.attribute("aria-selected")).toBe("true");
});

test("supports application-owned expansion and selection", () => {
  const Controlled = () => {
    const [expanded, setExpanded] = createSignal<readonly string[]>([]);
    const [selected, setSelected] = createSignal<string | null>(null);
    return (
      <TreeView
        items={items}
        aria-label="Controlled tree"
        expandedIds={expanded()}
        onExpandedChange={setExpanded}
        selectedId={selected()}
        onSelectedChange={setSelected}
      />
    );
  };
  const screen = renderComponent(Controlled);
  const source = screen.getByRole("treeitem", { name: "Source" });

  source.click();
  expect(source.attribute("aria-expanded")).toBe("true");
  expect(source.attribute("aria-selected")).toBe("true");
  expect(screen.getByRole("treeitem", { name: "Components" })).not.toBeNull();
});
