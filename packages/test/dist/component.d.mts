import { BuiltinHost, Host } from "@wabou/core/renderer";
import { JSX } from "solid-js";
//#region src/component.d.ts
interface ComponentLocator {
  readonly tag: string;
  readonly role: string;
  readonly name: string;
  readonly text: string;
  readonly className: string;
  attribute(name: string): string | null;
  pointerDown(position?: ComponentPointerPosition): void;
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
interface ComponentScreen {
  getByRole(role: string, options?: {
    name?: string;
    index?: number;
  }): ComponentLocator;
  queryByRole(role: string, options?: {
    name?: string;
    index?: number;
  }): ComponentLocator | null;
  /** Commit reactive work scheduled outside a locator action, such as a timer. */
  flush(): void;
  dispose(): void;
}
interface RenderComponentOptions {
  /** Host fixture injected into the component subtree. */
  host?: Host;
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
export { ComponentLocator, ComponentPointerPosition, ComponentScreen, RenderComponentOptions, TestBuiltinHost, TestHostCall, TestHostFixture, cleanupComponents, createTestHost, renderComponent };
//# sourceMappingURL=component.d.mts.map