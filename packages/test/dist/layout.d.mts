//#region src/layout.d.ts
interface LayoutNodeKey {
  readonly lo: number;
  readonly hi: number;
}
interface LayoutRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}
interface LayoutComputedStyle {
  readonly position?: string | null;
  readonly overflowX?: string | null;
  readonly overflowY?: string | null;
  readonly overlayPlane?: string;
}
interface LayoutSemanticProjection {
  readonly role: string;
  readonly label?: string | null;
}
interface LayoutSnapshotNode {
  readonly id: LayoutNodeKey;
  readonly parentId?: LayoutNodeKey | null;
  readonly tag: string;
  readonly text?: string | null;
  readonly classes: readonly string[];
  readonly attrs: readonly (readonly [string, string])[];
  readonly rect: LayoutRect;
  readonly contentRect: LayoutRect;
  readonly styleDiagnostics: readonly string[];
  readonly semantic?: LayoutSemanticProjection | null;
  readonly computed: LayoutComputedStyle;
}
interface LayoutSnapshot {
  readonly status: {
    readonly viewportWidth: number;
    readonly viewportHeight: number;
    readonly deviceScale: number;
    readonly nodeCount: number;
  };
  readonly nodes: readonly LayoutSnapshotNode[];
}
interface LayoutQuery {
  readonly tag?: string;
  readonly role?: string;
  readonly name?: string;
  readonly text?: string;
  readonly className?: string;
}
interface LayoutDiagnostic {
  readonly code: "flow-sibling-overlap" | "style-diagnostic" | "text-overlap" | "visible-overflow";
  readonly message: string;
  readonly node: LayoutSnapshotNode;
  readonly related?: LayoutSnapshotNode;
  readonly amount?: number;
}
interface LayoutDiagnosticOptions {
  readonly tolerance?: number;
  /** Restrict checks to descendants of this node, including itself. */
  readonly within?: LayoutSnapshotNode;
}
interface LayoutRectAssertionOptions {
  readonly tolerance?: number;
  readonly label?: string;
}
declare const layoutRectRight: (rect: LayoutRect) => number;
declare const layoutRectBottom: (rect: LayoutRect) => number;
/** Assert that a completed native layout rect stays inside another rect. */
declare function assertLayoutRectContains(outer: LayoutRect, inner: LayoutRect, options?: LayoutRectAssertionOptions): void;
/** Validate the Rust snapshot boundary before layout assertions consume it. */
declare function parseLayoutSnapshot(value: unknown): LayoutSnapshot;
declare function layoutRole(node: LayoutSnapshotNode): string;
declare function layoutName(node: LayoutSnapshotNode): string;
declare function queryLayoutNodes(snapshot: LayoutSnapshot, query: LayoutQuery): readonly LayoutSnapshotNode[];
declare function getLayoutNode(snapshot: LayoutSnapshot, query: LayoutQuery): LayoutSnapshotNode;
/** Stable text projection of the exact Style IR + Taffy result. */
declare function formatLayoutTree(snapshot: LayoutSnapshot): string;
declare function visibleOverflowDiagnostics(snapshot: LayoutSnapshot, options?: LayoutDiagnosticOptions): readonly LayoutDiagnostic[];
/** Opt-in collision check for normal-flow siblings. */
declare function siblingCollisionDiagnostics(snapshot: LayoutSnapshot, options?: LayoutDiagnosticOptions): readonly LayoutDiagnostic[];
/**
 * Opt-in collision check for visible text leaves across component subtrees.
 * This catches content collisions that a direct-sibling layout check cannot
 * see, such as a transformed reaction inside a bubble covering its footer.
 */
declare function textCollisionDiagnostics(snapshot: LayoutSnapshot, options?: LayoutDiagnosticOptions): readonly LayoutDiagnostic[];
declare function styleDiagnostics(snapshot: LayoutSnapshot, options?: LayoutDiagnosticOptions): readonly LayoutDiagnostic[];
declare function assertNoLayoutDiagnostics(diagnostics: readonly LayoutDiagnostic[]): void;
//#endregion
export { LayoutComputedStyle, LayoutDiagnostic, LayoutDiagnosticOptions, LayoutNodeKey, LayoutQuery, LayoutRect, LayoutRectAssertionOptions, LayoutSemanticProjection, LayoutSnapshot, LayoutSnapshotNode, assertLayoutRectContains, assertNoLayoutDiagnostics, formatLayoutTree, getLayoutNode, layoutName, layoutRectBottom, layoutRectRight, layoutRole, parseLayoutSnapshot, queryLayoutNodes, siblingCollisionDiagnostics, styleDiagnostics, textCollisionDiagnostics, visibleOverflowDiagnostics };
//# sourceMappingURL=layout.d.mts.map