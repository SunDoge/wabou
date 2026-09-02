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
  /** Resolved text size in logical pixels, suitable for typography contracts. */
  readonly fontSize?: number | null;
  /** Resolved numeric text weight, after theme and font fallback resolution. */
  readonly fontWeight?: number | null;
  /** Resolved foreground and painted background used by visual contracts. */
  readonly textColor?: string | null;
  readonly background?: string | null;
  readonly opacity?: number | null;
}
interface LayoutSemanticProjection {
  readonly role: string;
  readonly label?: string | null;
}
interface LayoutTextMetrics {
  readonly source: "node" | "widget";
  readonly lineBox: LayoutRect;
  readonly baseline: number;
}
interface LayoutClip {
  readonly coordinateSpace: string;
  readonly rect: LayoutRect;
}
interface LayoutClipInfo {
  readonly chain: readonly LayoutClip[];
  readonly effective?: LayoutClip | null;
}
interface LayoutSnapshotNode {
  readonly id: LayoutNodeKey;
  readonly parentId?: LayoutNodeKey | null;
  readonly tag: string;
  readonly text?: string | null;
  readonly textMetrics?: LayoutTextMetrics | null;
  readonly classes: readonly string[];
  readonly attrs: readonly (readonly [string, string])[];
  readonly rect: LayoutRect;
  readonly contentRect: LayoutRect;
  readonly styleDiagnostics: readonly string[];
  readonly semantic?: LayoutSemanticProjection | null;
  /** Resolved native clipping published by DevTools. */
  readonly clip?: LayoutClipInfo;
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
  readonly code: "flow-sibling-overlap" | "interactive-target-too-small" | "low-text-contrast" | "style-diagnostic" | "text-overlap" | "visible-overflow";
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
interface LayoutVisualDiagnosticOptions extends LayoutDiagnosticOptions {
  /** WCAG-style contrast ratio used as a visual legibility floor. */
  readonly minimumTextContrast?: number;
  /** Minimum logical size of button-like controls. */
  readonly minimumInteractiveTarget?: number;
}
interface LayoutRectAssertionOptions {
  readonly tolerance?: number;
  readonly label?: string;
}
interface LayoutTextStyleAssertionOptions {
  /** Exact resolved logical font size expected by the component contract. */
  readonly fontSize?: number;
  /** Exact resolved numeric font weight expected by the component contract. */
  readonly fontWeight?: number;
  readonly tolerance?: number;
  readonly label?: string;
}
declare const layoutRectRight: (rect: LayoutRect) => number;
declare const layoutRectBottom: (rect: LayoutRect) => number;
/** Assert that a completed native layout rect stays inside another rect. */
declare function assertLayoutRectContains(outer: LayoutRect, inner: LayoutRect, options?: LayoutRectAssertionOptions): void;
/**
 * Assert typography after class resolution, Style IR application and native
 * layout. This deliberately checks the completed layout node instead of source
 * class names, so token and font-resolution regressions are visible to tests.
 */
declare function assertLayoutTextStyle(node: LayoutSnapshotNode, options: LayoutTextStyleAssertionOptions): void;
/** Validate the Rust snapshot boundary before layout assertions consume it. */
declare function parseLayoutSnapshot(value: unknown): LayoutSnapshot;
/** Return the visual contrast ratio for two opaque hexadecimal colors. */
declare function layoutColorContrast(foreground: string, background: string): number | undefined;
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
/**
 * Opt-in visual legibility checks over the resolved native scene contract.
 * These intentionally consume computed colors and geometry rather than source
 * class names, so theme changes and component composition are covered too.
 */
declare function visualQualityDiagnostics(snapshot: LayoutSnapshot, options?: LayoutVisualDiagnosticOptions): readonly LayoutDiagnostic[];
declare function assertNoLayoutDiagnostics(diagnostics: readonly LayoutDiagnostic[]): void;
//#endregion
export { LayoutClip, LayoutClipInfo, LayoutComputedStyle, LayoutDiagnostic, LayoutDiagnosticOptions, LayoutNodeKey, LayoutQuery, LayoutRect, LayoutRectAssertionOptions, LayoutSemanticProjection, LayoutSnapshot, LayoutSnapshotNode, LayoutTextMetrics, LayoutTextStyleAssertionOptions, LayoutVisualDiagnosticOptions, assertLayoutRectContains, assertLayoutTextStyle, assertNoLayoutDiagnostics, formatLayoutTree, getLayoutNode, layoutColorContrast, layoutName, layoutRectBottom, layoutRectRight, layoutRole, parseLayoutSnapshot, queryLayoutNodes, siblingCollisionDiagnostics, styleDiagnostics, textCollisionDiagnostics, visibleOverflowDiagnostics, visualQualityDiagnostics };
//# sourceMappingURL=layout.d.mts.map