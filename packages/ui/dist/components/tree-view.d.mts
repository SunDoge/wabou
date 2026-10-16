import { JSX } from "solid-js";
//#region src/components/tree-view.d.ts
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
export declare function createTreeModel(nodes: readonly TreeNode[]): TreeModel;
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
  /** Window a large fixed-height tree while retaining native tree semantics. */
  virtual?: {
    itemHeight: number;
    overscan?: number;
    viewportHeight?: number;
  };
  class?: string;
  itemClass?: string;
}
/** A single-select tree with explicit data, expansion, and native focus routing. */
export declare function TreeView(props: TreeViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=tree-view.d.mts.map