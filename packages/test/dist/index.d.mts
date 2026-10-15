import { DebugOverlayPaintStats, FrameStats, WabouExposedSemanticRole } from "@wabou/core/renderer";
import { WindowKey } from "@wabou/core";
//#region src/poll.d.ts
interface PollOptions {
  timeout?: number;
  interval?: number;
  /** The predicate must remain true continuously for this many milliseconds. */
  stableFor?: number;
}
interface ResolvedPollOptions {
  timeout: number;
  interval: number;
  stableFor: number;
}
//#endregion
//#region src/index.d.ts
export interface NativeWindowState {
  presence: "visible" | "hidden" | "surface-released" | "closed";
  surfaceGeneration: number;
}
interface NativeTestCapability {
  writeTextFile(relativePath: string, contents: string): string;
  waitForIdle(lo: number, hi: number): Promise<boolean>;
  nativeClose(lo: number, hi: number, mutableVisibility: boolean): Promise<boolean>;
  showWindow(lo: number, hi: number): Promise<boolean>;
  resizeWindow(lo: number, hi: number, width: number, height: number): Promise<boolean>;
  fileDrop(lo: number, hi: number, phase: TestFileDropPhase, paths: string): Promise<boolean>;
  windowState(lo: number, hi: number): string;
  windowViewport(lo: number, hi: number): string;
  clickByRole(lo: number, hi: number, role: string, label: string, index: number | null, scope: string): Promise<boolean>;
  inputByRole(lo: number, hi: number, role: string, label: string, input: string, index: number | null, scope: string): Promise<boolean>;
  queryByRole(lo: number, hi: number, role: string, label: string, index: number | null, scope: string): Promise<string | null | undefined>;
  queueEffect(capability: number, method: number, result: string): string | null;
  takePendingEffectFixtures(): string;
  finish(report: string): boolean;
}
declare module "@wabou/core/registry" {
  interface HostCapabilities {
    readonly test: NativeTestCapability;
  }
}
export interface TestContext {
  readonly page: TestPage;
  readonly window: TestWindow;
  readonly effects: TestEffects;
  readonly files: TestFiles;
  readonly diagnostics: TestDiagnostics;
}
export interface TestDiagnostics {
  frameStats(): FrameStats | null;
  overlayPaintStats(): DebugOverlayPaintStats | null;
}
export interface TestFiles {
  /** Write exact UTF-8 contents beneath this run's isolated temporary root. */
  writeText(relativePath: string, contents: string): string;
}
export interface TestEffectResponseMap {
  clipboardRead: string | null;
  clipboardWrite: null;
  contextMenuShow: string | null;
  dialogOpen: string[] | null;
  dialogSave: string[] | null;
  dialogPickDirectory: string[] | null;
  dialogMessage: "ok" | "cancel" | "yes" | "no" | "custom";
  notificationShow: null;
  windowClose: null;
}
export type TestEffectOperation = keyof TestEffectResponseMap;
export interface TestEffects {
  /** Queue one deterministic response for the next matching native effect. */
  respond<K extends TestEffectOperation>(operation: K, result: TestEffectResponseMap[K]): void;
}
export interface TestWindow {
  /** Window key of the runtime executing this behavior scenario. */
  readonly current: WindowKey;
  nativeClose(windowId: WindowKey, platform: "wayland" | "mutable-visibility"): Promise<void>;
  show(windowId: WindowKey): Promise<void>;
  /** Resize a visible window in logical pixels through the native surface path. */
  resize(windowId: WindowKey, width: number, height: number): Promise<void>;
  /** Dispatch one native file-drag lifecycle event to a window. */
  fileDrop(windowId: WindowKey, phase: TestFileDropPhase, paths?: readonly string[]): Promise<void>;
  state(windowId: WindowKey): NativeWindowState | null;
}
export type TestFileDropPhase = "entered" | "moved" | "left" | "dropped";
export interface TestPage {
  readonly effects: TestEffects;
  /** Bind subsequent locators and frame barriers to one logical window. */
  forWindow(windowId: WindowKey): TestPage;
  getByRole(role: SemanticRole, options: {
    name: string;
    index?: number;
  }): Locator;
  waitForIdle(): Promise<void>;
}
export type SemanticRole = WabouExposedSemanticRole;
export interface Locator {
  readonly windowId: WindowKey;
  readonly role: SemanticRole;
  readonly name: string;
  readonly index?: number;
  /** Ancestor selector chain used to resolve this locator's semantic subtree. */
  readonly scope: readonly LocatorSelector[];
  getByRole(role: SemanticRole, options: {
    name: string;
    index?: number;
  }): Locator;
  click(options?: LocatorWaitOptions): Promise<void>;
  dragBy(deltaX: number, deltaY: number, options?: LocatorWaitOptions): Promise<void>;
  press(key: string, modifiers?: {
    shift?: boolean;
    control?: boolean;
    alt?: boolean;
    meta?: boolean;
  }, options?: LocatorWaitOptions): Promise<void>;
  type(text: string, options?: LocatorWaitOptions): Promise<void>;
  paste(text: string, options?: LocatorWaitOptions): Promise<void>;
  ime(text: string, options?: LocatorWaitOptions): Promise<void>;
  wheel(deltaY: number, deltaX?: number, options?: LocatorWaitOptions): Promise<void>;
  waitFor(options?: LocatorWaitOptions): Promise<void>;
  snapshot(): Promise<LocatorSnapshot>;
}
export interface LocatorSnapshot {
  name: string | null;
  value: string | null;
  numericValue: number | null;
  minNumericValue: number | null;
  maxNumericValue: number | null;
  bounds: LocatorBounds;
  disabled: boolean;
  checked: boolean | "mixed" | null;
  pressed: boolean | "mixed" | null;
  selected: boolean | null;
  current: LocatorCurrent | null;
  expanded: boolean | null;
  focused: boolean;
}
export type LocatorCurrent = "true" | "page" | "step" | "location" | "date" | "time";
export interface LocatorBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}
export type LocatorBoundsField = keyof LocatorBounds;
export interface LocatorNumericRange {
  value: number;
  min: number;
  max: number;
}
export type LocatorWaitOptions = PollOptions;
export type LocatorAssertionOptions = LocatorWaitOptions;
export type TestInput = {
  type: "probe";
} | {
  type: "drag";
  deltaX: number;
  deltaY: number;
} | {
  type: "key";
  key: string;
  modifiers: number;
} | {
  type: "text";
  text: string;
} | {
  type: "paste";
  text: string;
} | {
  type: "ime";
  text: string;
} | {
  type: "wheel";
  deltaX: number;
  deltaY: number;
};
export type LocatorAssertion = {
  type: "absent";
} | {
  type: "count";
  expected: number;
} | {
  type: "text";
  expected: string;
} | {
  type: "value";
  expected: string;
} | {
  type: "numericRange";
  expected: Partial<LocatorNumericRange>;
  tolerance: number;
} | {
  type: "disabled";
  expected: boolean;
} | {
  type: "checked";
  expected: boolean | "mixed";
} | {
  type: "selected";
  expected: boolean;
} | {
  type: "current";
  expected: LocatorCurrent | null;
} | {
  type: "expanded";
  expected: boolean;
} | {
  type: "pressed";
  expected: boolean;
} | {
  type: "focused";
  expected: boolean;
} | {
  type: "bounds";
  expected: Partial<LocatorBounds>;
  tolerance: number;
} | {
  type: "withinBounds";
  expected: LocatorBounds;
  tolerance: number;
} | {
  type: "notOverlap";
  other: LocatorReference;
  tolerance: number;
} | {
  type: "sameBounds";
  other: LocatorReference;
  fields: LocatorBoundsField[];
  tolerance: number;
} | {
  type: "viewport";
  tolerance: number;
};
export interface BoundsAssertionOptions extends LocatorAssertionOptions {
  /** Maximum absolute difference for supplied coordinates, in logical pixels. */
  tolerance?: number;
}
export interface NumericRangeAssertionOptions extends LocatorAssertionOptions {
  /** Maximum absolute difference; defaults to 1e-9. */
  tolerance?: number;
}
export declare const TEST_ARTIFACT_VERSION: 1;
export interface LocatorSelector {
  role: SemanticRole;
  name: string;
  index?: number;
}
export interface LocatorReference extends LocatorSelector {
  scope?: LocatorSelector[];
}
export interface TestEnvironment {
  backend: "deterministic" | "native";
  renderer: "vello-gpu" | "unknown";
  os: string;
  arch: string;
  wabouVersion: string;
}
export interface TestReport {
  version: typeof TEST_ARTIFACT_VERSION;
  /** Added by the Rust host before report.json is written. */
  environment?: TestEnvironment;
  passed: boolean;
  tests: TestResult[];
  trace: TestAction[];
}
export interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
  /** Half-open range into the report's action trace. */
  traceStart: number;
  traceEnd: number;
  durationMs: number;
}
export type TestAction = {
  action: "writeTextFile";
  relativePath: string;
  contents: string;
  path: string;
} | {
  action: "respondToEffect";
  operation: TestEffectOperation;
  result: TestEffectResponseMap[TestEffectOperation];
} | {
  action: "nativeClose";
  windowId: WindowKey;
  platform: "wayland" | "mutable-visibility";
} | {
  action: "showWindow";
  windowId: WindowKey;
} | {
  action: "resizeWindow";
  windowId: WindowKey;
  width: number;
  height: number;
} | {
  action: "fileDrop";
  windowId: WindowKey;
  phase: TestFileDropPhase;
  paths: string[];
} | {
  action: "clickByRole";
  windowId: WindowKey;
  role: SemanticRole;
  label: string;
  index?: number;
  scope?: LocatorSelector[];
  wait?: ResolvedPollOptions;
} | {
  action: "inputByRole";
  windowId: WindowKey;
  role: SemanticRole;
  label: string;
  index?: number;
  scope?: LocatorSelector[];
  input: TestInput;
  wait?: ResolvedPollOptions;
} | {
  action: "waitForByRole";
  windowId: WindowKey;
  role: SemanticRole;
  label: string;
  index?: number;
  scope?: LocatorSelector[];
  wait: ResolvedPollOptions;
} | {
  action: "assertByRole";
  windowId: WindowKey;
  role: SemanticRole;
  label: string;
  index?: number;
  scope?: LocatorSelector[];
  assertion: LocatorAssertion;
  wait: ResolvedPollOptions;
} | {
  action: "assertWindowState";
  windowId: WindowKey;
  expected: NativeWindowState;
  wait: ResolvedPollOptions;
};
type TestBody = (context: TestContext) => void | Promise<void>;
export interface TestOptions {
  /** Per-test timeout in milliseconds. Defaults to 5 seconds. */
  timeout?: number;
}
export declare function test(name: string, body: TestBody, options?: TestOptions): void;
/** Register a previously recorded action trace as a behavior test. */
export declare function replay(actions: readonly TestAction[]): void;
export declare function expect<T>(actual: T): {
  toBe(expected: T): void;
  toEqual(expected: T): void;
  toHaveState(windowId: WindowKey, expected: NativeWindowState, options?: LocatorAssertionOptions): Promise<void>;
  toHaveText(expected: string, options?: LocatorAssertionOptions): Promise<void>;
  toBeAbsent(options?: LocatorAssertionOptions): Promise<void>;
  toHaveCount(expected: number, options?: LocatorAssertionOptions): Promise<void>;
  toHaveValue(expected: string, options?: LocatorAssertionOptions): Promise<void>;
  toHaveRange(expected: Partial<LocatorNumericRange>, options?: NumericRangeAssertionOptions): Promise<void>;
  toBeDisabled(options?: LocatorAssertionOptions): Promise<void>;
  toBeEnabled(options?: LocatorAssertionOptions): Promise<void>;
  toBeChecked(options?: LocatorAssertionOptions): Promise<void>;
  toBeUnchecked(options?: LocatorAssertionOptions): Promise<void>;
  toBeIndeterminate(options?: LocatorAssertionOptions): Promise<void>;
  toBeSelected(options?: LocatorAssertionOptions): Promise<void>;
  toBeDeselected(options?: LocatorAssertionOptions): Promise<void>;
  toBeCurrent(expected?: LocatorCurrent, options?: LocatorAssertionOptions): Promise<void>;
  toNotBeCurrent(options?: LocatorAssertionOptions): Promise<void>;
  toBeExpanded(options?: LocatorAssertionOptions): Promise<void>;
  toBeCollapsed(options?: LocatorAssertionOptions): Promise<void>;
  toBePressed(options?: LocatorAssertionOptions): Promise<void>;
  toBeUnpressed(options?: LocatorAssertionOptions): Promise<void>;
  toBeFocused(options?: LocatorAssertionOptions): Promise<void>;
  toBeBlurred(options?: LocatorAssertionOptions): Promise<void>;
  toHaveBounds(expected: Partial<LocatorBounds>, options?: BoundsAssertionOptions): Promise<void>;
  toBeWithinBounds(expected: LocatorBounds, options?: BoundsAssertionOptions): Promise<void>;
  toNotOverlap(other: Locator, options?: BoundsAssertionOptions): Promise<void>;
  toHaveSameBoundsAs(other: Locator, fields?: readonly LocatorBoundsField[], options?: BoundsAssertionOptions): Promise<void>;
  toBeInViewport(options?: BoundsAssertionOptions): Promise<void>;
};
export declare namespace expect {
  var poll: <T>(read: () => T | Promise<T>, options?: {
    timeout?: number;
    interval?: number;
  }) => {
    toBe(expected: T): Promise<void>;
  };
}
//#endregion
export {};
//# sourceMappingURL=index.d.mts.map