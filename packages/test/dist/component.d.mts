import { BuiltinHost, Host } from "@wabou/core/renderer";
import { JSX } from "solid-js";
//#region src/component.d.ts
interface ComponentTypedStyleValue {
  readonly kind: number;
  readonly value: number;
}
type ComponentStyleValue = string | ComponentTypedStyleValue;
interface ComponentRoleListOptions {
  name?: string;
  disabled?: boolean;
  readOnly?: boolean;
  checked?: boolean | "mixed";
  selected?: boolean;
  expanded?: boolean;
  pressed?: boolean | "mixed";
  current?: boolean | string;
  orientation?: "horizontal" | "vertical";
  focused?: boolean;
}
interface ComponentRoleQueryOptions extends ComponentRoleListOptions {
  /** Select one occurrence in depth-first authored order. */
  index?: number;
}
interface ComponentQueries {
  getByRole(role: string, options?: ComponentRoleQueryOptions): ComponentLocator;
  queryByRole(role: string, options?: ComponentRoleQueryOptions): ComponentLocator | null;
  getAllByRole(role: string, options?: ComponentRoleListOptions): readonly ComponentLocator[];
  queryAllByRole(role: string, options?: ComponentRoleListOptions): readonly ComponentLocator[];
}
interface ComponentLocator extends ComponentQueries {
  readonly tag: string;
  readonly role: string;
  readonly name: string;
  readonly text: string;
  readonly className: string;
  /** Last authored string or typed value emitted for an inline style property. */
  style(name: string): ComponentStyleValue | null;
  /** Direct authored children for visual protocol assertions. Prefer role queries for behavior. */
  readonly children: readonly ComponentLocator[];
  /** Disabled state as authored through `disabled` or `aria-disabled`. */
  readonly disabled: boolean;
  /** Read-only state as authored through `readOnly` or `aria-readonly`. */
  readonly readOnly: boolean;
  /** Toggle state authored through `aria-checked`. */
  readonly checked: boolean | "mixed" | null;
  /** Selection state authored through `aria-selected`. */
  readonly selected: boolean | null;
  /** Disclosure state authored through `aria-expanded`. */
  readonly expanded: boolean | null;
  /** Toggle-button state authored through `aria-pressed`. */
  readonly pressed: boolean | "mixed" | null;
  /** Current item state authored through `aria-current`. */
  readonly current: boolean | string | null;
  /** Component axis authored through `aria-orientation`. */
  readonly orientation: "horizontal" | "vertical" | null;
  /** Authored textual value, including an input's controlled display value. */
  readonly value: string | null;
  /** Numeric range state authored through `aria-valuenow`. */
  readonly numericValue: number | null;
  /** Lower numeric range bound authored through `aria-valuemin`. */
  readonly minNumericValue: number | null;
  /** Upper numeric range bound authored through `aria-valuemax`. */
  readonly maxNumericValue: number | null;
  /** Human-readable numeric value authored through `aria-valuetext`. */
  readonly valueText: string | null;
  /** Last runtime affine transform emitted through the native protocol. */
  readonly transform: readonly [number, number, number, number, number, number] | null;
  /** Whether this locator owns the harness's native focus simulation. */
  readonly focused: boolean;
  /** Native tab order emitted through Wabou's interaction policy protocol. */
  readonly focusOrder: number | null;
  /** Whether native pointer and keyboard routing is blocked for this subtree. */
  readonly interactionBlocked: boolean;
  /** Whether native focus traversal is contained by this subtree. */
  readonly focusContained: boolean;
  attribute(name: string): string | null;
  pointerDown(position?: ComponentPointerPosition): void;
  /** Dispatch an uncaptured native pointer move with no pressed buttons. */
  movePointer(position?: ComponentPointerPosition): void;
  /** Dispatch a captured native pointer move while preserving button state. */
  pointerMove(position?: ComponentPointerPosition): void;
  pointerUp(position?: ComponentPointerPosition): void;
  click(): void;
  /** Dispatch a secondary-click context-menu event at a deterministic point. */
  contextMenu(position?: ComponentPointerPosition): void;
  press(key: string): void;
  input(value: string): void;
  /** Dispatch native focus/focusin, blurring the previously focused locator. */
  focus(): void;
  /** Dispatch native blur/focusout when this locator owns focus. */
  blur(): void;
  hover(): void;
  unhover(): void;
  /** Publish a deterministic native content-box observation. */
  resize(size: {
    width: number;
    height: number;
  }): void;
}
interface ComponentPointerPosition {
  clientX?: number;
  clientY?: number;
  offsetX?: number;
  offsetY?: number;
}
interface ComponentScreen extends ComponentQueries {
  /** Commit reactive work scheduled outside a locator action, such as a timer. */
  flush(): void;
  /** Advance a harness-owned fake clock and commit resulting reactive work. */
  advanceTime(milliseconds: number): Promise<void>;
  /** Retry an assertion while committing Promise-driven component updates. */
  waitFor<T>(assertion: () => T | Promise<T>, options?: ComponentWaitForOptions): Promise<T>;
  dispose(): void;
}
interface ComponentWaitForOptions {
  /** Total retry budget in milliseconds. Defaults to 1000. */
  timeout?: number;
  /** Retry interval in milliseconds. Defaults to 10. */
  interval?: number;
}
interface RenderComponentOptions {
  /** Host fixture injected into the component subtree. */
  host?: Host;
  /** Use a fake clock owned and restored by this component screen. */
  clock?: "real" | "fake";
}
interface TestHostCall {
  readonly path: string;
  readonly args: readonly unknown[];
}
interface TestHostFixture<H extends Host> {
  readonly host: H;
  readonly calls: readonly TestHostCall[];
  callsTo(path: string): readonly TestHostCall[];
  clearCalls(): void;
}
interface TestBuiltinHost {
  system?: Partial<BuiltinHost["system"]>;
  fonts?: Partial<BuiltinHost["fonts"]>;
  diagnostics?: Partial<BuiltinHost["diagnostics"]>;
  intl?: Partial<BuiltinHost["intl"]>;
  layout?: Partial<BuiltinHost["layout"]>;
}
/** Create a typed, deterministic Host with automatic call recording. */
declare function createTestHost<C extends object = Record<string, never>>(capabilities?: C, builtins?: TestBuiltinHost): TestHostFixture<Host & C>;
/** Dispose the active component tree. Vitest users get this automatically. */
declare function cleanupComponents(): void;
/**
 * Mount a component into Wabou's real Solid renderer while recording its
 * authored host tree. This is intentionally a fast component-contract test:
 * native layout, hit testing, and final semantic projection remain the job of
 * `wabou test` behavior scenarios.
 */
declare function renderComponent(render: () => JSX.Element, options?: RenderComponentOptions): ComponentScreen;
//#endregion
export { ComponentLocator, ComponentPointerPosition, ComponentQueries, ComponentRoleListOptions, ComponentRoleQueryOptions, ComponentScreen, ComponentStyleValue, ComponentTypedStyleValue, ComponentWaitForOptions, RenderComponentOptions, TestBuiltinHost, TestHostCall, TestHostFixture, cleanupComponents, createTestHost, renderComponent };
//# sourceMappingURL=component.d.mts.map