import { WabouExposedSemanticRole } from "@wabou/solid-renderer";
//#region src/poll.d.ts
interface PollOptions {
  timeout?: number;
  interval?: number;
}
interface ResolvedPollOptions {
  timeout: number;
  interval: number;
}
//#endregion
//#region src/index.d.ts
interface NativeWindowState {
  presence: "visible" | "hidden" | "surface-released" | "closed";
  surfaceGeneration: number;
}
interface NativeTestCapability {
  waitForIdle(windowId: number): Promise<boolean>;
  nativeClose(windowId: number, mutableVisibility: boolean): Promise<boolean>;
  showWindow(windowId: number): Promise<boolean>;
  windowState(windowId: number): string;
  clickByRole(windowId: number, role: string, label: string, index: number | null): Promise<boolean>;
  inputByRole(windowId: number, role: string, label: string, input: string, index: number | null): Promise<boolean>;
  queryByRole(windowId: number, role: string, label: string, index: number | null): Promise<string | null | undefined>;
  finish(report: string): boolean;
}
declare module "@wabou/solid-renderer" {
  interface HostCapabilities {
    readonly test: NativeTestCapability;
  }
}
interface TestContext {
  readonly page: TestPage;
  readonly window: TestWindow;
}
interface TestWindow {
  nativeClose(windowId: number, platform: "wayland" | "mutable-visibility"): Promise<void>;
  show(windowId: number): Promise<void>;
  state(windowId: number): NativeWindowState | null;
}
interface TestPage {
  /** Bind subsequent locators and frame barriers to one logical window. */
  forWindow(windowId: number): TestPage;
  getByRole(role: SemanticRole, options: {
    name: string;
    index?: number;
  }): Locator;
  waitForIdle(): Promise<void>;
}
type SemanticRole = WabouExposedSemanticRole;
interface Locator {
  readonly windowId: number;
  readonly role: SemanticRole;
  readonly name: string;
  readonly index?: number;
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
interface LocatorSnapshot {
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
  expanded: boolean | null;
  focused: boolean;
}
interface LocatorBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}
interface LocatorNumericRange {
  value: number;
  min: number;
  max: number;
}
type LocatorWaitOptions = PollOptions;
type LocatorAssertionOptions = LocatorWaitOptions;
type TestInput = {
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
type LocatorAssertion = {
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
};
interface BoundsAssertionOptions extends LocatorAssertionOptions {
  /** Maximum absolute difference for supplied coordinates, in logical pixels. */
  tolerance?: number;
}
interface NumericRangeAssertionOptions extends LocatorAssertionOptions {
  /** Maximum absolute difference; defaults to 1e-9. */
  tolerance?: number;
}
declare const TEST_ARTIFACT_VERSION: 1;
interface TestEnvironment {
  backend: "deterministic" | "native";
  os: string;
  arch: string;
  wabouVersion: string;
}
interface TestReport {
  version: typeof TEST_ARTIFACT_VERSION;
  /** Added by the Rust host before report.json is written. */
  environment?: TestEnvironment;
  passed: boolean;
  tests: TestResult[];
  trace: TestAction[];
}
interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
  /** Half-open range into the report's action trace. */
  traceStart: number;
  traceEnd: number;
  durationMs: number;
}
type TestAction = {
  action: "nativeClose";
  windowId: number;
  platform: "wayland" | "mutable-visibility";
} | {
  action: "showWindow";
  windowId: number;
} | {
  action: "clickByRole";
  windowId: number;
  role: SemanticRole;
  label: string;
  index?: number;
  wait?: ResolvedPollOptions;
} | {
  action: "inputByRole";
  windowId: number;
  role: SemanticRole;
  label: string;
  index?: number;
  input: TestInput;
  wait?: ResolvedPollOptions;
} | {
  action: "waitForByRole";
  windowId: number;
  role: SemanticRole;
  label: string;
  index?: number;
  wait: ResolvedPollOptions;
} | {
  action: "assertByRole";
  windowId: number;
  role: SemanticRole;
  label: string;
  index?: number;
  assertion: LocatorAssertion;
  wait: ResolvedPollOptions;
} | {
  action: "assertWindowState";
  windowId: number;
  expected: NativeWindowState;
  wait: ResolvedPollOptions;
};
type TestBody = (context: TestContext) => void | Promise<void>;
interface TestOptions {
  /** Per-test timeout in milliseconds. Defaults to 5 seconds. */
  timeout?: number;
}
declare function test(name: string, body: TestBody, options?: TestOptions): void;
/** Register a previously recorded action trace as a behavior test. */
declare function replay(actions: readonly TestAction[]): void;
declare function expect<T>(actual: T): {
  toBe(expected: T): void;
  toEqual(expected: T): void;
  toHaveState(windowId: number, expected: NativeWindowState, options?: LocatorAssertionOptions): Promise<void>;
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
  toBeExpanded(options?: LocatorAssertionOptions): Promise<void>;
  toBeCollapsed(options?: LocatorAssertionOptions): Promise<void>;
  toBePressed(options?: LocatorAssertionOptions): Promise<void>;
  toBeUnpressed(options?: LocatorAssertionOptions): Promise<void>;
  toBeFocused(options?: LocatorAssertionOptions): Promise<void>;
  toBeBlurred(options?: LocatorAssertionOptions): Promise<void>;
  toHaveBounds(expected: Partial<LocatorBounds>, options?: BoundsAssertionOptions): Promise<void>;
};
declare namespace expect {
  var poll: <T>(read: () => T | Promise<T>, options?: {
    timeout?: number;
    interval?: number;
  }) => {
    toBe(expected: T): Promise<void>;
  };
}
//#endregion
export { BoundsAssertionOptions, Locator, LocatorAssertion, LocatorAssertionOptions, LocatorBounds, LocatorNumericRange, LocatorSnapshot, LocatorWaitOptions, NativeWindowState, NumericRangeAssertionOptions, SemanticRole, TEST_ARTIFACT_VERSION, TestAction, TestContext, TestEnvironment, TestInput, TestOptions, TestPage, TestReport, TestResult, TestWindow, expect, replay, test };
//# sourceMappingURL=index.d.mts.map