import type { WindowKey } from "@wabou/core";
import { effectOps } from "@wabou/core/effects";
import {
  defaultHost,
  type WabouExposedSemanticRole,
} from "@wabou/core/renderer";
import { containmentDiagnostic } from "./locator-bounds";
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
  validateWindowKey,
  validateWindowPresence,
} from "./validation";

export interface NativeWindowState {
  presence: "visible" | "hidden" | "surface-released" | "closed";
  surfaceGeneration: number;
}

interface NativeTestCapability {
  writeTextFile(relativePath: string, contents: string): string;
  waitForIdle(lo: number, hi: number): Promise<boolean>;
  nativeClose(
    lo: number,
    hi: number,
    mutableVisibility: boolean,
  ): Promise<boolean>;
  showWindow(lo: number, hi: number): Promise<boolean>;
  resizeWindow(
    lo: number,
    hi: number,
    width: number,
    height: number,
  ): Promise<boolean>;
  windowState(lo: number, hi: number): string;
  windowViewport(lo: number, hi: number): string;
  clickByRole(
    lo: number,
    hi: number,
    role: string,
    label: string,
    index: number | null,
  ): Promise<boolean>;
  inputByRole(
    lo: number,
    hi: number,
    role: string,
    label: string,
    input: string,
    index: number | null,
  ): Promise<boolean>;
  queryByRole(
    lo: number,
    hi: number,
    role: string,
    label: string,
    index: number | null,
  ): Promise<string | null | undefined>;
  queueEffect(
    capability: number,
    method: number,
    result: string,
  ): string | null;
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
}

export type TestEffectOperation = keyof TestEffectResponseMap;

export interface TestEffects {
  /** Queue one deterministic response for the next matching native effect. */
  respond<K extends TestEffectOperation>(
    operation: K,
    result: TestEffectResponseMap[K],
  ): void;
}

export interface TestWindow {
  /** Window key of the runtime executing this behavior scenario. */
  readonly current: WindowKey;
  nativeClose(
    windowId: WindowKey,
    platform: "wayland" | "mutable-visibility",
  ): Promise<void>;
  show(windowId: WindowKey): Promise<void>;
  /** Resize a visible window in logical pixels through the native surface path. */
  resize(windowId: WindowKey, width: number, height: number): Promise<void>;
  state(windowId: WindowKey): NativeWindowState | null;
}

export interface TestPage {
  readonly effects: TestEffects;
  /** Bind subsequent locators and frame barriers to one logical window. */
  forWindow(windowId: WindowKey): TestPage;
  getByRole(
    role: SemanticRole,
    options: { name: string; index?: number },
  ): Locator;
  waitForIdle(): Promise<void>;
}

export type SemanticRole = WabouExposedSemanticRole;

export interface Locator {
  readonly windowId: WindowKey;
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
  current: LocatorCurrent | null;
  expanded: boolean | null;
  focused: boolean;
}

export type LocatorCurrent =
  | "true"
  | "page"
  | "step"
  | "location"
  | "date"
  | "time";

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
  | { type: "current"; expected: LocatorCurrent | null }
  | { type: "expanded"; expected: boolean }
  | { type: "pressed"; expected: boolean }
  | { type: "focused"; expected: boolean }
  | {
      type: "bounds";
      expected: Partial<LocatorBounds>;
      tolerance: number;
    }
  | {
      type: "withinBounds";
      expected: LocatorBounds;
      tolerance: number;
    }
  | { type: "viewport"; tolerance: number };

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
      action: "respondToEffect";
      operation: TestEffectOperation;
      result: TestEffectResponseMap[TestEffectOperation];
    }
  | {
      action: "nativeClose";
      windowId: WindowKey;
      platform: "wayland" | "mutable-visibility";
    }
  | { action: "showWindow"; windowId: WindowKey }
  | {
      action: "resizeWindow";
      windowId: WindowKey;
      width: number;
      height: number;
    }
  | {
      action: "clickByRole";
      windowId: WindowKey;
      role: SemanticRole;
      label: string;
      index?: number;
      wait?: ResolvedPollOptions;
    }
  | {
      action: "inputByRole";
      windowId: WindowKey;
      role: SemanticRole;
      label: string;
      index?: number;
      input: TestInput;
      wait?: ResolvedPollOptions;
    }
  | {
      action: "waitForByRole";
      windowId: WindowKey;
      role: SemanticRole;
      label: string;
      index?: number;
      wait: ResolvedPollOptions;
    }
  | {
      action: "assertByRole";
      windowId: WindowKey;
      role: SemanticRole;
      label: string;
      index?: number;
      assertion: LocatorAssertion;
      wait: ResolvedPollOptions;
    }
  | {
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
const tests: Array<{ name: string; body: TestBody; timeout: number }> = [];
const testNames = new Set<string>();
const registrationErrors: string[] = [];
const trace: TestAction[] = [];
const MAX_LOCATOR_INDEX = 0xffff_ffff;

function encodedEffectResult(
  operation: TestEffectOperation,
  result: TestEffectResponseMap[TestEffectOperation],
): unknown {
  if (operation === "clipboardRead")
    return { kind: "clipboardText", value: result };
  if (operation === "contextMenuShow")
    return { kind: "contextMenuSelection", value: result };
  if (
    operation === "dialogOpen" ||
    operation === "dialogSave" ||
    operation === "dialogPickDirectory"
  )
    return { kind: "dialogPaths", value: result };
  if (operation === "dialogMessage")
    return { kind: "dialogMessage", value: result };
  return { kind: "unit" };
}

const effects: TestEffects = {
  respond(operation, result) {
    const op = effectOps[operation];
    const error = capability().queueEffect(
      op.capability,
      op.method,
      JSON.stringify(encodedEffectResult(operation, result)),
    );
    if (error) throw new Error(error);
    trace.push({ action: "respondToEffect", operation, result });
  },
};

class LocatorNotFoundError extends Error {}

function capability(): NativeTestCapability {
  const value = defaultHost.test;
  if (!value) throw new Error("@wabou/test requires `wabou test`");
  return value;
}

function windowLabel(windowId: WindowKey): string {
  return `${windowId.lo}v${windowId.hi}`;
}

function decodeWindowViewport(windowId: WindowKey): LocatorBounds {
  const raw = capability().windowViewport(windowId.lo, windowId.hi);
  const value = JSON.parse(raw) as Partial<LocatorBounds> | null;
  if (!value) {
    throw new Error(
      `native window ${windowLabel(windowId)} has no visible viewport`,
    );
  }
  for (const key of ["x", "y", "width", "height"] as const) {
    if (!Number.isFinite(value[key])) {
      throw new Error(
        `native window ${windowLabel(windowId)} returned an invalid viewport`,
      );
    }
  }
  const viewport = value as LocatorBounds;
  if (viewport.width < 0 || viewport.height < 0) {
    throw new Error(
      `native window ${windowLabel(windowId)} returned a negative viewport`,
    );
  }
  return viewport;
}

function createPage(windowId: WindowKey): TestPage {
  validateWindowKey(windowId);
  return {
    effects,
    forWindow(nextWindowId) {
      return createPage(nextWindowId);
    },
    async waitForIdle() {
      // Cross two complete JS/native frame boundaries. One frame publishes
      // host inputs such as WindowMetrics into Solid; the next projects the
      // resulting responsive tree into layout and semantics. Waiting for all
      // animation to stop is intentionally not part of this contract because
      // applications may own infinite loops such as spinners and ripples.
      for (let frame = 0; frame < 2; frame++) {
        await Promise.resolve();
        await new Promise<void>((resolve) =>
          requestAnimationFrame(() => resolve()),
        );
        if (!(await capability().waitForIdle(windowId.lo, windowId.hi))) {
          throw new Error(
            `native window ${windowLabel(windowId)} did not complete frame ${frame + 1} of 2`,
          );
        }
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
      const description = `${locatorLabel} in window ${windowLabel(windowId)}`;
      const sendInput = async (value: TestInput): Promise<boolean> => {
        if (
          !(await capability().inputByRole(
            windowId.lo,
            windowId.hi,
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
          windowId.lo,
          windowId.hi,
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
      const waitUntilPresent = async (
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
          (state) => state !== null,
          wait,
          () => createPage(windowId).waitForIdle(),
        );
        if (!result.matched) {
          throw new Error(
            ambiguity === undefined
              ? `no ${locatorLabel} after ${wait.timeout}ms`
              : `${ambiguity} after ${wait.timeout}ms`,
          );
        }
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
              windowId.lo,
              windowId.hi,
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
          // Wheel input is positional and may scroll an enabled ancestor even
          // when the semantic node under the pointer is disabled.
          await waitUntilPresent(wait);
          if (!(await sendInput({ type: "wheel", deltaX, deltaY }))) {
            throw new Error(`cannot wheel ${locatorLabel}`);
          }
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
  page: createPage({
    lo: __wabou_window_id_lo,
    hi: __wabou_window_id_hi,
  } as WindowKey),
  window: {
    current: {
      lo: __wabou_window_id_lo,
      hi: __wabou_window_id_hi,
    } as WindowKey,
    async nativeClose(windowId, platform) {
      validateWindowKey(windowId);
      trace.push({ action: "nativeClose", windowId, platform });
      if (
        !(await capability().nativeClose(
          windowId.lo,
          windowId.hi,
          platform !== "wayland",
        ))
      ) {
        throw new Error(
          `failed to enqueue native close for window ${windowLabel(windowId)}`,
        );
      }
    },
    async show(windowId) {
      validateWindowKey(windowId);
      trace.push({ action: "showWindow", windowId });
      if (!(await capability().showWindow(windowId.lo, windowId.hi))) {
        throw new Error(
          `failed to enqueue show for window ${windowLabel(windowId)}`,
        );
      }
    },
    async resize(windowId, width, height) {
      validateWindowKey(windowId);
      for (const [name, value] of [
        ["width", width],
        ["height", height],
      ] as const) {
        if (!Number.isSafeInteger(value) || value <= 0 || value > 0xffff_ffff)
          throw new RangeError(
            `${name} must be an integer between 1 and 4294967295`,
          );
      }
      // A scenario is evaluated while the first native surface may still be
      // completing creation. Cross one full source/native frame boundary so
      // resize is valid even as the first authored test action.
      await createPage(windowId).waitForIdle();
      trace.push({ action: "resizeWindow", windowId, width, height });
      if (
        !(await capability().resizeWindow(
          windowId.lo,
          windowId.hi,
          width,
          height,
        ))
      ) {
        throw new Error(
          `failed to resize visible window ${windowLabel(windowId)}`,
        );
      }
      await createPage(windowId).waitForIdle();
    },
    state(windowId) {
      validateWindowKey(windowId);
      return JSON.parse(
        capability().windowState(windowId.lo, windowId.hi),
      ) as NativeWindowState | null;
    },
  },
  effects,
  files: {
    writeText(relativePath, contents) {
      const result = JSON.parse(
        capability().writeTextFile(relativePath, contents),
      ) as { path?: string; error?: string };
      if (result.error) throw new Error(result.error);
      if (!result.path) throw new Error("native test fixture omitted its path");
      return result.path;
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
  viewport?: LocatorBounds,
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
  if (assertion.type === "current")
    return state.current === assertion.expected
      ? null
      : `expected locator current state to be ${JSON.stringify(assertion.expected)}, received ${JSON.stringify(state.current)}`;
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
  if (assertion.type === "withinBounds") {
    return containmentDiagnostic(
      state.bounds,
      assertion.expected,
      assertion.tolerance,
      "within",
    );
  }
  if (assertion.type === "viewport") {
    if (!viewport) throw new Error("native window viewport is unavailable");
    return containmentDiagnostic(
      state.bounds,
      viewport,
      assertion.tolerance,
      "inside viewport",
    );
  }
  return state.focused === assertion.expected
    ? null
    : `expected locator to be ${assertion.expected ? "focused" : "blurred"}`;
}

async function locatorAbsenceDiagnostic(
  target: Locator,
): Promise<string | null> {
  const raw = await capability().queryByRole(
    target.windowId.lo,
    target.windowId.hi,
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
    target.windowId.lo,
    target.windowId.hi,
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
        const viewport =
          assertion.type === "viewport"
            ? decodeWindowViewport(target.windowId)
            : undefined;
        return locatorAssertionDiagnostic(
          assertion,
          await target.snapshot(),
          viewport,
        );
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
  windowId: WindowKey,
  expected: NativeWindowState,
  options: LocatorAssertionOptions = {},
): Promise<void> {
  validateWindowKey(windowId);
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
      windowId: WindowKey,
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
    toBeCurrent(
      expected: LocatorCurrent = "true",
      options?: LocatorAssertionOptions,
    ): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "current", expected },
        options,
      );
    },
    toNotBeCurrent(options?: LocatorAssertionOptions): Promise<void> {
      return assertLocatorEventually(
        locator(),
        { type: "current", expected: null },
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
    toBeWithinBounds(
      expected: LocatorBounds,
      options: BoundsAssertionOptions = {},
    ): Promise<void> {
      for (const key of ["x", "y", "width", "height"] as const) {
        if (!Number.isFinite(expected[key]))
          throw new RangeError("containing bounds must contain finite numbers");
      }
      if (expected.width < 0 || expected.height < 0)
        throw new RangeError(
          "containing bounds width and height cannot be negative",
        );
      const tolerance = options.tolerance ?? 0.5;
      validateTolerance("locator containing bounds", tolerance);
      return assertLocatorEventually(
        locator(),
        { type: "withinBounds", expected: { ...expected }, tolerance },
        options,
      );
    },
    toBeInViewport(options: BoundsAssertionOptions = {}): Promise<void> {
      const tolerance = options.tolerance ?? 0.5;
      validateTolerance("locator viewport", tolerance);
      return assertLocatorEventually(
        locator(),
        { type: "viewport", tolerance },
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
              const pendingEffects = capability().takePendingEffectFixtures();
              if (pendingEffects !== "") {
                throw new Error(
                  `native effect fixture was not consumed: ${pendingEffects}`,
                );
              }
              results.push({
                name: entry.name,
                passed: true,
                traceStart,
                traceEnd: trace.length,
                durationMs: performance.now() - startedAt,
              });
            } catch (error) {
              capability().takePendingEffectFixtures();
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
