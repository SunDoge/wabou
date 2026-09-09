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

test("represents a lazy branch before its children are loaded", () => {
  const model = createTreeModel([
    { id: "remote", label: "Remote folder", hasChildren: true },
  ]);
  expect(model.isBranch("remote")).toBe(true);
  expect(model.visible(["remote"]).map(({ node }) => node.id)).toEqual([
    "remote",
  ]);
});

test("expands, selects, and skips disabled items through native focus routing", () => {
  const screen = renderComponent(() => (
    <TreeView items={items} aria-label="Project files" />
  ));
  const source = screen.getByRole("treeitem", { name: "Source" });

  expect(source.expanded).toBe(false);
  source.focus();
  source.press("ArrowRight");
  expect(screen.queryByRole("treeitem", { name: "Components" })).not.toBeNull();
  expect(source.expanded).toBe(true);

  source.press("ArrowRight");
  const components = screen.getByRole("treeitem", { name: "Components" });
  expect(components.focused).toBe(true);
  components.press("ArrowDown");
  const runtime = screen.getByRole("treeitem", { name: "Runtime" });
  expect(runtime.focused).toBe(true);
  runtime.press("End");
  expect(screen.getByRole("treeitem", { name: "Tests" }).focused).toBe(true);

  runtime.click();
  expect(runtime.selected).toBe(true);
  const tests = screen.getByRole("treeitem", { name: "Tests" });
  tests.click();
  expect(runtime.selected).toBe(false);
  expect(tests.selected).toBe(true);
  expect(screen.getAllByRole("treeitem", { selected: true })).toHaveLength(1);
});

test("focuses the next matching visible node through typeahead", () => {
  const screen = renderComponent(() => (
    <TreeView items={items} aria-label="Searchable project files" />
  ));
  const source = screen.getByRole("treeitem", { name: "Source" });

  source.focus();
  source.press("t");

  expect(screen.getByRole("treeitem", { name: "Tests" }).focused).toBe(true);
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
  expect(source.expanded).toBe(true);
  expect(source.selected).toBe(true);
  expect(screen.getByRole("treeitem", { name: "Components" })).not.toBeNull();
});

test("windows large trees and keeps End-key focus navigation intact", () => {
  const largeItems = Array.from({ length: 200 }, (_, index) => ({
    id: `entry-${index}`,
    label: `Entry ${index}`,
  }));
  const screen = renderComponent(() => (
    <TreeView
      items={largeItems}
      aria-label="Large tree"
      virtual={{ itemHeight: 34, viewportHeight: 102, overscan: 1 }}
    />
  ));

  expect(screen.getAllByRole("treeitem")).toHaveLength(4);
  const first = screen.getByRole("treeitem", { name: "Entry 0" });
  first.focus();
  first.press("End");
  screen.flush();

  expect(screen.getByRole("treeitem", { name: "Entry 199" }).focused).toBe(
    true,
  );
  expect(screen.queryByRole("treeitem", { name: "Entry 0" })).toBeNull();
  expect(screen.getAllByRole("treeitem").length).toBeLessThanOrEqual(5);
});

test("does not carry virtual row interaction state to a different item", () => {
  const largeItems = Array.from({ length: 200 }, (_, index) => ({
    id: `entry-${index}`,
    label: `Entry ${index}`,
  }));
  const screen = renderComponent(() => (
    <TreeView
      items={largeItems}
      aria-label="Virtual interaction tree"
      virtual={{ itemHeight: 34, viewportHeight: 102, overscan: 1 }}
    />
  ));

  const first = screen.getByRole("treeitem", { name: "Entry 0" });
  first.hover();
  first.focus();
  first.press("End");
  screen.flush();

  const last = screen.getByRole("treeitem", { name: "Entry 199" });
  expect(last.className).not.toContain("bg-control-hover");
  expect(screen.queryAllByRole("treeitem", { selected: true })).toHaveLength(0);
});

test("typeahead scrolls a virtual tree before focusing an offscreen match", () => {
  const largeItems = Array.from({ length: 200 }, (_, index) => ({
    id: `entry-${index}`,
    label: index === 173 ? "Quarterly reports" : `Entry ${index}`,
  }));
  const screen = renderComponent(() => (
    <TreeView
      items={largeItems}
      aria-label="Virtual searchable tree"
      virtual={{ itemHeight: 34, viewportHeight: 102, overscan: 1 }}
    />
  ));

  const first = screen.getByRole("treeitem", { name: "Entry 0" });
  first.focus();
  first.press("q");
  screen.flush();

  expect(
    screen.getByRole("treeitem", { name: "Quarterly reports" }).focused,
  ).toBe(true);
  expect(screen.queryByRole("treeitem", { name: "Entry 0" })).toBeNull();
});
