import { defaultHost } from "@wabou/solid-renderer";
import {
  decodeLocatorQuery,
  decodeNativeLocatorQuery,
  LocatorAmbiguousError,
  locatorQueryIsAbsent,
  locatorQueryMatchCount,
} from "./locator-query";
import {
  type PollOptions,
  pollUntil,
  type ResolvedPollOptions,
  resolvePollOptions,
} from "./poll";
import { replayActions } from "./replay";
import {
  replayTimeout,
  SUITE_TIMEOUT,
  SuiteTimeoutError,
  TestTimeoutError,
  testTimeout,
  withSuiteTimeout,
  withTestTimeout,
} from "./timeout";
import {
  validateInputDeltas,
  validateKey,
  validateLocatorCount,
  validateSurfaceGeneration,
  validateTolerance,
  validateWindowId,
  validateWindowPresence,
} from "./validation";

export interface NativeWindowState {
  presence: "visible" | "hidden" | "surface-released" | "closed";
  surfaceGeneration: number;
}

interface NativeTestCapability {
  waitForIdle(windowId: number): Promise<boolean>;
  nativeClose(windowId: number, mutableVisibility: boolean): Promise<boolean>;
  showWindow(windowId: number): Promise<boolean>;
  windowState(windowId: number): string;
  clickByRole(
    windowId: number,
    role: string,
    label: string,
    index: number | null,
  ): Promise<boolean>;
  inputByRole(
    windowId: number,
    role: string,
    label: string,
    input: string,
    index: number | null,
  ): Promise<boolean>;
  queryByRole(
    windowId: number,
    role: string,
    label: string,
    index: number | null,
  ): Promise<string | null | undefined>;
  finish(report: string): boolean;
}

declare module "@wabou/solid-renderer" {
  interface HostCapabilities {
    readonly test: NativeTestCapability;
  }
}

export interface TestContext {
  readonly page: TestPage;
  readonly window: TestWindow;
}

export interface TestWindow {
  nativeClose(
    windowId: number,
    platform: "wayland" | "mutable-visibility",
  ): Promise<void>;
  show(windowId: number): Promise<void>;
  state(windowId: number): NativeWindowState | null;
}

export interface TestPage {
  /** Bind subsequent locators and frame barriers to one logical window. */
  forWindow(windowId: number): TestPage;
  getByRole(
    role: SemanticRole,
    options: { name: string; index?: number },
  ): Locator;
  waitForIdle(): Promise<void>;
}

export type SemanticRole =
  | "button"
  | "group"
  | "textbox"
  | "link"
  | "dialog"
  | "alert"
  | "status"
  | "checkbox"
  | "radio"
  | "switch"
  | "combobox"
  | "listbox"
  | "option"
  | "menu"
  | "menuitem"
  | "tree"
  | "treeitem"
  | "table"
  | "row"
  | "cell"
  | "columnheader"
  | "rowheader"
  | "slider"
  | "progressbar"
  | "heading"
  | "label"
  | "img"
  | "radiogroup"
  | "tablist"
  | "tab"
  | "tabpanel"
  | "grid"
  | "gridcell";

export interface Locator {
  readonly windowId: number;
  readonly role: SemanticRole;
  readonly name: string;
  readonly index?: number;
  click(options?: LocatorWaitOptions): Promise<void>;
  dragBy(
    deltaX: number,
    deltaY: number,
    options?: LocatorWaitOptions,
  ): Promise<void>;
  press(
    key: string,
    modifiers?: {
      shift?: boolean;
      control?: boolean;
      alt?: boolean;
      meta?: boolean;
    },
    options?: LocatorWaitOptions,
  ): Promise<void>;
  type(text: string, options?: LocatorWaitOptions): Promise<void>;
  paste(text: string, options?: LocatorWaitOptions): Promise<void>;
  ime(text: string, options?: LocatorWaitOptions): Promise<void>;
  wheel(
    deltaY: number,
    deltaX?: number,
    options?: LocatorWaitOptions,
  ): Promise<void>;
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
  expanded: boolean | null;
  focused: boolean;
}

export interface LocatorBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LocatorNumericRange {
  value: number;
  min: number;
  max: number;
}

export type LocatorWaitOptions = PollOptions;
export type LocatorAssertionOptions = LocatorWaitOptions;

export type TestInput =
  | { type: "probe" }
  | { type: "drag"; deltaX: number; deltaY: number }
  | { type: "key"; key: string; modifiers: number }
  | { type: "text"; text: string }
  | { type: "paste"; text: string }
  | { type: "ime"; text: string }
  | { type: "wheel"; deltaX: number; deltaY: number };

export type LocatorAssertion =
  | { type: "absent" }
  | { type: "count"; expected: number }
  | { type: "text"; expected: string }
  | { type: "value"; expected: string }
  | {
      type: "numericRange";
      expected: Partial<LocatorNumericRange>;
      tolerance: number;
    }
  | { type: "disabled"; expected: boolean }
  | { type: "checked"; expected: boolean | "mixed" }
  | { type: "selected"; expected: boolean }
  | { type: "expanded"; expected: boolean }
  | { type: "pressed"; expected: boolean }
  | { type: "focused"; expected: boolean }
  | {
      type: "bounds";
      expected: Partial<LocatorBounds>;
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

export const TEST_ARTIFACT_VERSION = 1 as const;

export interface TestEnvironment {
  backend: "deterministic" | "native";
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

export type TestAction =
  | {
      action: "nativeClose";
      windowId: number;
      platform: "wayland" | "mutable-visibility";
    }
  | { action: "showWindow"; windowId: number }
  | {
      action: "clickByRole";
      windowId: number;
      role: SemanticRole;
      label: string;
      index?: number;
      wait?: ResolvedPollOptions;
    }
  | {
      action: "inputByRole";
      windowId: number;
      role: SemanticRole;
      label: string;
      index?: number;
      input: TestInput;
      wait?: ResolvedPollOptions;
    }
  | {
      action: "waitForByRole";
      windowId: number;
      role: SemanticRole;
      label: string;
      index?: number;
      wait: ResolvedPollOptions;
    }
  | {
      action: "assertByRole";
      windowId: number;
      role: SemanticRole;
      label: string;
      index?: number;
      assertion: LocatorAssertion;
      wait: ResolvedPollOptions;
    }
  | {
      action: "assertWindowState";
      windowId: number;
      expected: NativeWindowState;
      wait: ResolvedPollOptions;
    };

type TestBody = (context: TestContext) => void | Promise<void>;
export interface TestOptions {
  /** Per-test timeout in milliseconds. Defaults to 5 seconds. */
  timeout?: number;
}
const tests: Array<{ name: string; body: TestBody; timeout: number }> = [];
const testNames = new Set<string>();
const registrationErrors: string[] = [];
const trace: TestAction[] = [];
const MAX_LOCATOR_INDEX = 0xffff_ffff;

class LocatorNotFoundError extends Error {}

function capability(): NativeTestCapability {
  const value = defaultHost.test;
  if (!value) throw new Error("@wabou/test requires `wabou test`");
  return value;
}

function createPage(windowId: number): TestPage {
  validateWindowId(windowId);
  return {
    forWindow(nextWindowId) {
      return createPage(nextWindowId);
    },
    async waitForIdle() {
      // Let queued Solid work reach the renderer, then wait for the native test
      // driver to observe a complete frame. A pair of JS animation frames alone
      // cannot prove that layout and semantics have caught up.
      await Promise.resolve();
      await new Promise<void>((resolve) =>
        requestAnimationFrame(() => resolve()),
      );
      if (!(await capability().waitForIdle(windowId))) {
        throw new Error(`native window ${windowId} did not become idle`);
      }
    },
    getByRole(role, options) {
      const index = options.index;
      if (
        index !== undefined &&
        (!Number.isSafeInteger(index) || index < 0 || index > MAX_LOCATOR_INDEX)
      ) {
        throw new RangeError(
          `locator index must be an integer between 0 and ${MAX_LOCATOR_INDEX}`,
        );
      }
      const locatorLabel = `${role} named ${JSON.stringify(options.name)}${index === undefined ? "" : ` at index ${index}`}`;
      const description = `${locatorLabel} in window ${windowId}`;
      const sendInput = async (value: TestInput): Promise<boolean> => {
        if (
          !(await capability().inputByRole(
            windowId,
            role,
            options.name,
            JSON.stringify(value),
            index ?? null,
          ))
        ) {
          return false;
        }
        return true;
      };
      const input = async (value: TestInput): Promise<void> => {
        if (!(await sendInput(value))) {
          throw new Error(`no enabled ${locatorLabel}`);
        }
      };
      const probe = async (): Promise<LocatorSnapshot | null> => {
        const result = await capability().queryByRole(
          windowId,
          role,
          options.name,
          index ?? null,
        );
        return decodeLocatorQuery<LocatorSnapshot>(result, description, index);
      };
      const snapshot = async (): Promise<LocatorSnapshot> => {
        const value = await probe();
        if (!value) {
          throw new LocatorNotFoundError(`no ${locatorLabel}`);
        }
        return value;
      };
      const waitUntilActionable = async (
        assertionOptions: LocatorWaitOptions = {},
      ): Promise<void> => {
        const wait = resolvePollOptions(assertionOptions);
        let ambiguity: string | undefined;
        const result = await pollUntil(
          async () => {
            try {
              const value = await probe();
              ambiguity = undefined;
              return value;
            } catch (error) {
              if (!(error instanceof LocatorAmbiguousError)) throw error;
              ambiguity = error.message;
              return null;
            }
          },
          (state) => state !== null && !state.disabled,
          wait,
          () => createPage(windowId).waitForIdle(),
        );
        if (result.matched) return;
        throw new Error(
          (ambiguity === undefined
            ? undefined
            : `${ambiguity} after ${wait.timeout}ms`) ??
            `no enabled ${locatorLabel} after ${wait.timeout}ms`,
        );
      };
      return {
        windowId,
        role,
        name: options.name,
        index,
        async click(assertionOptions) {
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "clickByRole",
            windowId,
            role,
            label: options.name,
            index,
            wait,
          });
          await waitUntilActionable(wait);
          if (
            !(await capability().clickByRole(
              windowId,
              role,
              options.name,
              index ?? null,
            ))
          ) {
            throw new Error(`no enabled ${locatorLabel}`);
          }
        },
        async dragBy(deltaX, deltaY, assertionOptions) {
          validateInputDeltas("drag", deltaX, deltaY);
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "inputByRole",
            windowId,
            role,
            label: options.name,
            index,
            input: { type: "drag", deltaX, deltaY },
            wait,
          });
          await waitUntilActionable(wait);
          await input({ type: "drag", deltaX, deltaY });
        },
        async press(key, modifiers = {}, assertionOptions) {
          validateKey(key);
          const bits =
            (modifiers.shift ? 1 : 0) |
            (modifiers.control ? 2 : 0) |
            (modifiers.alt ? 4 : 0) |
            (modifiers.meta ? 8 : 0);
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "inputByRole",
            windowId,
            role,
            label: options.name,
            index,
            input: { type: "key", key, modifiers: bits },
            wait,
          });
          await waitUntilActionable(wait);
          await input({ type: "key", key, modifiers: bits });
        },
        async type(text, assertionOptions) {
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "inputByRole",
            windowId,
            role,
            label: options.name,
            index,
            input: { type: "text", text },
            wait,
          });
          await waitUntilActionable(wait);
          await input({ type: "text", text });
        },
        async paste(text, assertionOptions) {
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "inputByRole",
            windowId,
            role,
            label: options.name,
            index,
            input: { type: "paste", text },
            wait,
          });
          await waitUntilActionable(wait);
          await input({ type: "paste", text });
        },
        async ime(text, assertionOptions) {
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "inputByRole",
            windowId,
            role,
            label: options.name,
            index,
            input: { type: "ime", text },
            wait,
          });
          await waitUntilActionable(wait);
          await input({ type: "ime", text });
        },
        async wheel(deltaY, deltaX = 0, assertionOptions) {
          validateInputDeltas("wheel", deltaX, deltaY);
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "inputByRole",
            windowId,
            role,
            label: options.name,
            index,
            input: { type: "wheel", deltaX, deltaY },
            wait,
          });
          await waitUntilActionable(wait);
          await input({ type: "wheel", deltaX, deltaY });
        },
        async waitFor(assertionOptions = {}) {
          const wait = resolvePollOptions(assertionOptions);
          trace.push({
            action: "waitForByRole",
            windowId,
            role,
            label: options.name,
            index,
            wait,
          });
          let ambiguity: string | undefined;
          const result = await pollUntil(
            async () => {
              try {
                const value = await probe();
                ambiguity = undefined;
                return value;
              } catch (error) {
                if (!(error instanceof LocatorAmbiguousError)) throw error;
                ambiguity = error.message;
                return null;
              }
            },
            (state) => state !== null,
            wait,
            () => createPage(windowId).waitForIdle(),
          );
          if (result.matched) return;
          throw new Error(
            (ambiguity === undefined
              ? undefined
              : `${ambiguity} after ${wait.timeout}ms`) ??
              `no ${locatorLabel} after ${wait.timeout}ms`,
          );
        },
        snapshot,
      };
    },
  };
}

const context: TestContext = {
  page: createPage(1),
  window: {
    async nativeClose(windowId, platform) {
      validateWindowId(windowId);
      trace.push({ action: "nativeClose", windowId, platform });
      if (!(await capability().nativeClose(windowId, platform !== "wayland"))) {
        throw new Error(
          `failed to enqueue native close for window ${windowId}`,
        );
      }
    },
    async show(windowId) {
      validateWindowId(windowId);
      trace.push({ action: "showWindow", windowId });
      if (!(await capability().showWindow(windowId))) {
        throw new Error(`failed to enqueue show for window ${windowId}`);
      }
    },
    state(windowId) {
      validateWindowId(windowId);
      return JSON.parse(
        capability().windowState(windowId),
      ) as NativeWindowState | null;
    },
  },
};

export function test(
  name: string,
  body: TestBody,
  options: TestOptions = {},
): void {
  if (name.trim() === "") {
    registrationErrors.push("test name cannot be empty");
    return;
  }
  if (testNames.has(name)) {
    registrationErrors.push(`duplicate test name ${JSON.stringify(name)}`);
    return;
  }
  let timeout: number;
  try {
    timeout = testTimeout(options.timeout);
  } catch (error) {
    registrationErrors.push(
      `invalid options for test ${JSON.stringify(name)}: ${error instanceof Error ? error.message : String(error)}`,
    );
    return;
  }
  testNames.add(name);
  tests.push({ name, body, timeout });
}

function locatorAssertionDiagnostic(
  assertion: LocatorAssertion,
  state: LocatorSnapshot,
): string | null {
  if (assertion.type === "absent" || assertion.type === "count") {
    throw new Error(
      `${assertion.type} assertions do not accept a locator snapshot`,
    );
  }
  if (assertion.type === "text") {
    const value = state.value ?? state.name;
    return value === assertion.expected
      ? null
      : `expected locator text ${JSON.stringify(value)} to be ${JSON.stringify(assertion.expected)}`;
  }
  if (assertion.type === "value")
    return state.value === assertion.expected
      ? null
      : `expected locator value ${JSON.stringify(state.value)} to be ${JSON.stringify(assertion.expected)}`;
  if (assertion.type === "numericRange") {
    const actual: LocatorNumericRange = {
      value: state.numericValue as number,
      min: state.minNumericValue as number,
      max: state.maxNumericValue as number,
    };
    for (const key of ["value", "min", "max"] as const) {
      const expected = assertion.expected[key];
      if (
        expected !== undefined &&
        (actual[key] === null ||
          Math.abs(actual[key] - expected) > assertion.tolerance)
      ) {
        return `expected locator numeric range.${key} ${JSON.stringify(actual[key])} to be within ${assertion.tolerance} of ${expected}`;
      }
    }
    return null;
  }
  if (assertion.type === "disabled")
    return state.disabled === assertion.expected
      ? null
      : `expected locator to be ${assertion.expected ? "disabled" : "enabled"}`;
  if (assertion.type === "checked")
    return state.checked === assertion.expected
      ? null
      : `expected locator to be ${assertion.expected === "mixed" ? "indeterminate" : assertion.expected ? "checked" : "unchecked"}, received ${JSON.stringify(state.checked)}`;
  if (assertion.type === "selected")
    return state.selected === assertion.expected
      ? null
      : `expected locator to be ${assertion.expected ? "selected" : "deselected"}, received ${JSON.stringify(state.selected)}`;
  if (assertion.type === "expanded")
    return state.expanded === assertion.expected
      ? null
      : `expected locator to be ${assertion.expected ? "expanded" : "collapsed"}, received ${JSON.stringify(state.expanded)}`;
  if (assertion.type === "pressed")
    return state.pressed === assertion.expected
      ? null
      : `expected locator to be ${assertion.expected ? "pressed" : "unpressed"}, received ${JSON.stringify(state.pressed)}`;
  if (assertion.type === "bounds") {
    for (const key of ["x", "y", "width", "height"] as const) {
      const expected = assertion.expected[key];
      if (
        expected !== undefined &&
        Math.abs(state.bounds[key] - expected) > assertion.tolerance
      ) {
        return `expected locator bounds.${key} ${state.bounds[key]} to be within ${assertion.tolerance}px of ${expected}`;
      }
    }
    return null;
  }
  return state.focused === assertion.expected
    ? null
    : `expected locator to be ${assertion.expected ? "focused" : "blurred"}`;
}

async function locatorAbsenceDiagnostic(
  target: Locator,
): Promise<string | null> {
  const raw = await capability().queryByRole(
    target.windowId,
    target.role,
    target.name,
    target.index ?? null,
  );
  if (locatorQueryIsAbsent(raw, target.index)) {
    return null;
  }
  const query = decodeNativeLocatorQuery<LocatorSnapshot>(raw);
  if (query === null) throw new Error("unreachable absent locator query");
  const occurrence =
    target.index === undefined ? "" : ` at index ${target.index}`;
  return `expected ${target.role} named ${JSON.stringify(target.name)}${occurrence} to be absent, found ${query.matchCount} matching semantic ${query.matchCount === 1 ? "node" : "nodes"}`;
}

async function locatorCountDiagnostic(
  target: Locator,
  expected: number,
): Promise<string | null> {
  if (target.index !== undefined) {
    throw new Error("toHaveCount requires an unindexed locator");
  }
  const raw = await capability().queryByRole(
    target.windowId,
    target.role,
    target.name,
    null,
  );
  const actual = locatorQueryMatchCount(raw);
  return actual === expected
    ? null
    : `expected ${target.role} named ${JSON.stringify(target.name)} to have ${expected} ${expected === 1 ? "match" : "matches"}, found ${actual}`;
}

async function assertLocatorEventually(
  target: Locator,
  assertion: LocatorAssertion,
  options: LocatorAssertionOptions = {},
): Promise<void> {
  const wait = resolvePollOptions(options);
  trace.push({
    action: "assertByRole",
    windowId: target.windowId,
    role: target.role,
    label: target.name,
    index: target.index,
    assertion,
    wait,
  });
  const result = await pollUntil(
    async () => {
      if (assertion.type === "absent") {
        return locatorAbsenceDiagnostic(target);
      }
      if (assertion.type === "count") {
        return locatorCountDiagnostic(target, assertion.expected);
      }
      try {
        return locatorAssertionDiagnostic(assertion, await target.snapshot());
      } catch (error) {
        if (
          !(error instanceof LocatorNotFoundError) &&
          !(error instanceof LocatorAmbiguousError)
        )
          throw error;
        return error.message;
      }
    },
    (diagnostic) => diagnostic === null,
    wait,
    () => createPage(target.windowId).waitForIdle(),
  );
  if (!result.matched)
    throw new Error(`${result.value} after ${wait.timeout}ms`);
}

/** Register a previously recorded action trace as a behavior test. */
export function replay(actions: readonly TestAction[]): void {
  test(
    "replay action trace",
    async ({ window }) => {
      await replayActions(
        actions,
        context.page,
        window,
        replayLocatorAssertion,
        replayWindowAssertion,
      );
    },
    { timeout: replayTimeout(actions) },
  );
}

async function replayLocatorAssertion(
  locator: Locator,
  action: Extract<TestAction, { action: "assertByRole" }>,
): Promise<void> {
  await assertLocatorEventually(locator, action.assertion, action.wait);
}

async function replayWindowAssertion(
  window: TestWindow,
  action: Extract<TestAction, { action: "assertWindowState" }>,
): Promise<void> {
  await assertWindowStateEventually(
    window,
    action.windowId,
    action.expected,
    action.wait,
  );
}

async function assertWindowStateEventually(
  target: TestWindow,
  windowId: number,
  expected: NativeWindowState,
  options: LocatorAssertionOptions = {},
): Promise<void> {
  validateWindowId(windowId);
  validateWindowPresence(expected.presence);
  validateSurfaceGeneration(expected.surfaceGeneration);
  const wait = resolvePollOptions(options);
  // Capture the authored expectation so later object mutation cannot rewrite a
  // completed assertion in report.json.
  const recorded: NativeWindowState = {
    presence: expected.presence,
    surfaceGeneration: expected.surfaceGeneration,
  };
  trace.push({
    action: "assertWindowState",
    windowId,
    expected: recorded,
    wait,
  });
  const result = await pollUntil(
    () => target.state(windowId),
    (actual) =>
      actual?.presence === recorded.presence &&
      actual.surfaceGeneration === recorded.surfaceGeneration,
    wait,
  );
  if (!result.matched) {
    throw new Error(
      `expected window ${windowId} state ${JSON.stringify(result.value)} to be ${JSON.stringify(recorded)} after ${wait.timeout}ms`,
    );
  }
}

export function expect<T>(actual: T) {
  const locator = (): Locator => {
    if (!actual || typeof actual !== "object" || !("snapshot" in actual)) {
      throw new Error("this assertion requires a Wabou locator");
    }
    return actual as unknown as Locator;
  };
  return {
    toBe(expected: T): void {
      if (!Object.is(actual, expected)) {
        throw new Error(
          `expected ${JSON.stringify(actual)} to be ${JSON.stringify(expected)}`,
        );
      }
    },
    toEqual(expected: T): void {
      const left = JSON.stringify(actual);
      const right = JSON.stringify(expected);
      if (left !== right) throw new Error(`expected ${left} to equal ${right}`);
    },
    toHaveState(
      windowId: number,
      expected: NativeWindowState,
      options?: LocatorAssertionOptions,
    ): Promise<void> {
      if (actual !== context.window) {
        throw new Error(
          "toHaveState requires the Wabou test window capability",
        );
      }
      return assertWindowStateEventually(
        context.window,
        windowId,
        expected,
        options,
      );
    },
    toHaveText(
      expected: string,
      options?: LocatorAssertionOptions,
    ): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "text", expected },
        options,
      );
    },
    toBeAbsent(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(locator(), { type: "absent" }, options);
    },
    toHaveCount(
      expected: number,
      options?: LocatorAssertionOptions,
    ): Promise<void> {
      validateLocatorCount(expected);
      const target = locator();
      if (target.index !== undefined) {
        throw new Error("toHaveCount requires an unindexed locator");
      }
      return assertLocatorEventually(
        target,
        { type: "count", expected },
        options,
      );
    },
    toHaveValue(
      expected: string,
      options?: LocatorAssertionOptions,
    ): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "value", expected },
        options,
      );
    },
    toHaveRange(
      expected: Partial<LocatorNumericRange>,
      options: NumericRangeAssertionOptions = {},
    ): Promise<void> {
      const entries = Object.entries(expected);
      if (entries.length === 0) {
        throw new RangeError("expected locator numeric range cannot be empty");
      }
      const supported = new Set(["value", "min", "max"]);
      if (entries.some(([key]) => !supported.has(key))) {
        throw new RangeError(
          "expected locator numeric range may only contain value, min, and max",
        );
      }
      if (entries.some(([, value]) => !Number.isFinite(value))) {
        throw new RangeError(
          "expected locator numeric range must contain finite numbers",
        );
      }
      const tolerance = options.tolerance ?? 1e-9;
      validateTolerance("numeric range assertion", tolerance);
      return assertLocatorEventually(
        locator(),
        { type: "numericRange", expected: { ...expected }, tolerance },
        options,
      );
    },
    toBeDisabled(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "disabled", expected: true },
        options,
      );
    },
    toBeEnabled(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "disabled", expected: false },
        options,
      );
    },
    toBeChecked(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "checked", expected: true },
        options,
      );
    },
    toBeUnchecked(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "checked", expected: false },
        options,
      );
    },
    toBeIndeterminate(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "checked", expected: "mixed" },
        options,
      );
    },
    toBeSelected(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "selected", expected: true },
        options,
      );
    },
    toBeDeselected(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "selected", expected: false },
        options,
      );
    },
    toBeExpanded(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "expanded", expected: true },
        options,
      );
    },
    toBeCollapsed(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "expanded", expected: false },
        options,
      );
    },
    toBePressed(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "pressed", expected: true },
        options,
      );
    },
    toBeUnpressed(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "pressed", expected: false },
        options,
      );
    },
    toBeFocused(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "focused", expected: true },
        options,
      );
    },
    toBeBlurred(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "focused", expected: false },
        options,
      );
    },
    toHaveBounds(
      expected: Partial<LocatorBounds>,
      options: BoundsAssertionOptions = {},
    ): Promise<void> {
      const entries = Object.entries(expected);
      if (entries.length === 0) {
        throw new RangeError("expected locator bounds cannot be empty");
      }
      const supported = new Set(["x", "y", "width", "height"]);
      if (entries.some(([key]) => !supported.has(key))) {
        throw new RangeError(
          "expected locator bounds may only contain x, y, width, and height",
        );
      }
      if (entries.some(([, value]) => !Number.isFinite(value))) {
        throw new RangeError("expected locator bounds must be finite numbers");
      }
      const tolerance = options.tolerance ?? 0.5;
      validateTolerance("locator bounds", tolerance);
      return assertLocatorEventually(
        locator(),
        { type: "bounds", expected: { ...expected }, tolerance },
        options,
      );
    },
  };
}

expect.poll = function poll<T>(
  read: () => T | Promise<T>,
  options: { timeout?: number; interval?: number } = {},
) {
  return {
    async toBe(expected: T): Promise<void> {
      const result = await pollUntil(
        read,
        (actual) => Object.is(actual, expected),
        { timeout: options.timeout, interval: options.interval ?? 10 },
      );
      if (!result.matched) {
        throw new Error(
          `expected ${JSON.stringify(result.value)} to become ${JSON.stringify(expected)}`,
        );
      }
    },
  };
};

async function run(): Promise<void> {
  const results: TestReport["tests"] = [];
  let activeTest:
    | { name: string; traceStart: number; startedAt: number }
    | undefined;
  if (registrationErrors.length > 0) {
    results.push({
      name: "test suite",
      passed: false,
      error: registrationErrors.join("\n"),
      traceStart: 0,
      traceEnd: 0,
      durationMs: 0,
    });
  } else if (tests.length === 0) {
    results.push({
      name: "test suite",
      passed: false,
      error: "no tests registered",
      traceStart: 0,
      traceEnd: 0,
      durationMs: 0,
    });
  }
  if (registrationErrors.length === 0 && tests.length > 0) {
    try {
      await withSuiteTimeout(
        SUITE_TIMEOUT,
        async () => {
          for (const entry of tests) {
            const traceStart = trace.length;
            const startedAt = performance.now();
            activeTest = { name: entry.name, traceStart, startedAt };
            try {
              await withTestTimeout(entry.name, entry.timeout, () =>
                entry.body(context),
              );
              results.push({
                name: entry.name,
                passed: true,
                traceStart,
                traceEnd: trace.length,
                durationMs: performance.now() - startedAt,
              });
            } catch (error) {
              results.push({
                name: entry.name,
                passed: false,
                error:
                  error instanceof Error
                    ? `${error.message}${error.stack ? `\n${error.stack}` : ""}`
                    : String(error),
                traceStart,
                traceEnd: trace.length,
                durationMs: performance.now() - startedAt,
              });
              if (error instanceof TestTimeoutError) break;
            } finally {
              activeTest = undefined;
            }
          }
        },
        () => activeTest?.name,
      );
    } catch (error) {
      if (!(error instanceof SuiteTimeoutError)) throw error;
      results.push({
        name: activeTest?.name ?? "test suite",
        passed: false,
        error: `${error.message}${error.stack ? `\n${error.stack}` : ""}`,
        traceStart: activeTest?.traceStart ?? trace.length,
        traceEnd: trace.length,
        durationMs:
          activeTest === undefined
            ? SUITE_TIMEOUT
            : performance.now() - activeTest.startedAt,
      });
    }
  }
  const report: TestReport = {
    version: TEST_ARTIFACT_VERSION,
    passed: results.every((result) => result.passed),
    tests: results,
    trace,
  };
  capability().finish(JSON.stringify(report));
}

queueMicrotask(() => {
  void run().catch((error) => {
    const report: TestReport = {
      version: TEST_ARTIFACT_VERSION,
      passed: false,
      tests: [
        {
          name: "test runner",
          passed: false,
          error:
            error instanceof Error
              ? `${error.message}${error.stack ? `\n${error.stack}` : ""}`
              : String(error),
          traceStart: 0,
          traceEnd: 0,
          durationMs: 0,
        },
      ],
      trace: [],
    };
    capability().finish(JSON.stringify(report));
  });
});
