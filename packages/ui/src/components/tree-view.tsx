import type { Handle } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import {
  createMemo,
  createSignal,
  For as ForValue,
  type JSX,
  untrack,
} from "solid-js";
import { Button as HeadlessButton, Icon, Text, View } from "../primitives";
import { mergeClasses } from "@wabou/core/style";
import { createControllableState } from "./state";

export interface TreeNode {
  id: string;
  label: string;
  disabled?: boolean;
  /** The node is expandable even when its children have not been loaded yet. */
  hasChildren?: boolean;
  children?: readonly TreeNode[];
}

export interface VisibleTreeNode {
  node: TreeNode;
  parentId: string | null;
  level: number;
  position: number;
  setSize: number;
}

export interface TreeModel {
  get(id: string): TreeNode | undefined;
  parent(id: string): string | null | undefined;
  firstChild(id: string): string | undefined;
  isBranch(id: string): boolean;
  visible(expandedIds: readonly string[]): readonly VisibleTreeNode[];
}

/** Validates a nested tree once and provides deterministic visible traversal. */
export function createTreeModel(nodes: readonly TreeNode[]): TreeModel {
  const byId = new Map<string, TreeNode>();
  const parents = new Map<string, string | null>();
  const entries = new Map<string, VisibleTreeNode>();

  const visit = (items: readonly TreeNode[], parentId: string | null) => {
    items.forEach((node, index) => {
      if (!node.id) throw new Error("tree node id must not be empty");
      if (byId.has(node.id)) {
        throw new Error(`tree node id must be unique: ${node.id}`);
      }
      if (!node.label)
        throw new Error(`tree node label must not be empty: ${node.id}`);
      byId.set(node.id, node);
      parents.set(node.id, parentId);
      entries.set(node.id, {
        node,
        parentId,
        level: parentId === null ? 1 : (entries.get(parentId)?.level ?? 0) + 1,
        position: index + 1,
        setSize: items.length,
      });
      if (node.children?.length) visit(node.children, node.id);
    });
  };
  visit(nodes, null);

  const isBranch = (id: string) => {
    const node = byId.get(id);
    return Boolean(node?.hasChildren || node?.children?.length);
  };
  return {
    get: (id) => byId.get(id),
    parent: (id) => parents.get(id),
    firstChild: (id) => byId.get(id)?.children?.[0]?.id,
    isBranch,
    visible(expandedIds) {
      const expanded = new Set(expandedIds);
      const result: VisibleTreeNode[] = [];
      const flatten = (items: readonly TreeNode[]) => {
        items.forEach((node) => {
          const entry = entries.get(node.id);
          if (!entry) throw new Error(`missing tree model entry: ${node.id}`);
          result.push(entry);
          if (node.children?.length && expanded.has(node.id)) {
            flatten(node.children);
          }
        });
      };
      flatten(nodes);
      return result;
    },
  };
}

export interface TreeItemRenderState {
  expanded: boolean;
  selected: boolean;
  level: number;
}

export interface TreeViewProps {
  items: readonly TreeNode[];
  "aria-label": string;
  expandedIds?: readonly string[];
  defaultExpandedIds?: readonly string[];
  onExpandedChange?(ids: readonly string[]): void;
  /** `undefined` selects uncontrolled mode; `null` is a controlled empty selection. */
  selectedId?: string | null;
  defaultSelectedId?: string | null;
  onSelectedChange?(id: string | null): void;
  renderItem?(node: TreeNode, state: TreeItemRenderState): JSX.Element;
  class?: string;
  itemClass?: string;
}

function validateExpandedIds(
  model: TreeModel,
  ids: readonly string[],
): readonly string[] {
  const unique = new Set<string>();
  for (const id of ids) {
    if (!model.get(id)) throw new Error(`unknown expanded tree node: ${id}`);
    if (!model.isBranch(id))
      throw new Error(`tree leaf cannot be expanded: ${id}`);
    unique.add(id);
  }
  return [...unique];
}

/** A single-select tree with explicit data, expansion, and native focus routing. */
export function TreeView(props: TreeViewProps): JSX.Element {
  const initialModel = createTreeModel(untrack(() => props.items));
  const model = createMemo(() => createTreeModel(props.items));
  const defaultExpanded = validateExpandedIds(
    initialModel,
    untrack(() => props.defaultExpandedIds) ?? [],
  );
  const expandedState = createControllableState<readonly string[]>({
    value: () =>
      props.expandedIds === undefined
        ? undefined
        : validateExpandedIds(model(), props.expandedIds),
    defaultValue: defaultExpanded,
    onChange: props.onExpandedChange,
  });
  const selectedState = createControllableState<string | null>({
    value: () => props.selectedId,
    defaultValue: untrack(() => props.defaultSelectedId) ?? null,
    onChange: props.onSelectedChange,
  });
  const [activeId, setActiveId] = createSignal<string | undefined>(undefined, {
    ownedWrite: true,
  });
  const handles = new Map<string, Handle>();
  const expanded = () => expandedState.value();
  const visible = createMemo(() => model().visible(expanded()));
  const enabledVisible = () => visible().filter(({ node }) => !node.disabled);
  const isExpanded = (id: string) => expanded().includes(id);
  const isSelected = (id: string) => selectedState.value() === id;
  const tabStop = () => {
    const candidates = enabledVisible();
    const active = activeId();
    if (active && candidates.some(({ node }) => node.id === active))
      return active;
    const selected = selectedState.value();
    if (selected && candidates.some(({ node }) => node.id === selected)) {
      return selected;
    }
    return candidates[0]?.node.id;
  };
  const focus = (id: string | undefined) => {
    if (!id || model().get(id)?.disabled) return false;
    setActiveId(id);
    handles.get(id)?.focus();
    return true;
  };
  const setExpanded = (id: string, next: boolean) => {
    if (!model().isBranch(id)) return false;
    const current = expanded();
    const has = current.includes(id);
    if (has === next) return false;
    return expandedState.set(
      next ? [...current, id] : current.filter((candidate) => candidate !== id),
    );
  };
  const select = (node: TreeNode) => {
    if (node.disabled) return;
    selectedState.set(node.id);
  };
  const activate = (node: TreeNode) => {
    select(node);
    if (model().isBranch(node.id)) setExpanded(node.id, !isExpanded(node.id));
  };
  const moveLinear = (id: string, key: string) => {
    const candidates = enabledVisible();
    const index = candidates.findIndex(({ node }) => node.id === id);
    const target =
      key === "Home"
        ? candidates[0]
        : key === "End"
          ? candidates.at(-1)
          : key === "ArrowDown"
            ? candidates[index + 1]
            : key === "ArrowUp"
              ? candidates[index - 1]
              : undefined;
    return focus(target?.node.id);
  };
  const handleKey = (
    item: VisibleTreeNode,
    event: { key: string; preventDefault(): void },
  ) => {
    const { id } = item.node;
    let handled = false;
    if (["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      handled = moveLinear(id, event.key);
    } else if (event.key === "ArrowRight" && model().isBranch(id)) {
      handled = isExpanded(id)
        ? focus(model().firstChild(id))
        : setExpanded(id, true);
    } else if (event.key === "ArrowLeft") {
      handled = isExpanded(id)
        ? setExpanded(id, false)
        : focus(item.parentId ?? undefined);
    }
    if (handled) event.preventDefault();
  };

  return (
    <View
      role="tree"
      aria-label={props["aria-label"]}
      class={mergeClasses("min-w-0 flex flex-col gap-0.5", props.class)}
    >
      <ForValue each={visible()}>
        {(item) => {
          const branch = () => model().isBranch(item.node.id);
          return (
            <HeadlessButton
              unstyled
              ref={(node) => handles.set(item.node.id, node)}
              role="treeitem"
              aria-label={item.node.label}
              aria-expanded={branch() ? isExpanded(item.node.id) : undefined}
              aria-selected={isSelected(item.node.id)}
              selected={isSelected(item.node.id)}
              disabled={item.node.disabled}
              focusOrder={tabStop() === item.node.id ? 0 : -1}
              class={(state) =>
                mergeClasses(
                  "w-full h-8 min-w-0 pr-2 items-center gap-2 rounded-md text-sm",
                  state.selected
                    ? "bg-selected text-primary"
                    : state.hovered
                      ? "bg-control-hover text-primary"
                      : "bg-transparent text-secondary",
                  props.itemClass,
                )
              }
              style={{
                "padding-left": `${8 + (item.level - 1) * 20}px`,
              }}
              onFocus={() => setActiveId(item.node.id)}
              onClick={() => activate(item.node)}
              onKeyDown={(event) => handleKey(item, event)}
            >
              {branch() ? (
                <Icon
                  aria-hidden="true"
                  source={isExpanded(item.node.id) ? chevronDown : chevronRight}
                  size={14}
                  class="flex-none text-muted"
                />
              ) : (
                <View aria-hidden="true" class="w-3.5 h-3.5 flex-none" />
              )}
              {props.renderItem ? (
                props.renderItem(item.node, {
                  expanded: isExpanded(item.node.id),
                  selected: isSelected(item.node.id),
                  level: item.level,
                })
              ) : (
                <Text maxLines={1} class="min-w-0 flex-1 text-sm">
                  {item.node.label}
                </Text>
              )}
            </HeadlessButton>
          );
        }}
      </ForValue>
    </View>
  );
}
