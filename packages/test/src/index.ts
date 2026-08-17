import { defaultHost } from "@wabou/solid-renderer";
import { replayActions } from "./replay";

export interface NativeWindowState {
  presence: "visible" | "hidden" | "surface-released" | "closed";
  surfaceGeneration: number;
}

interface NativeTestCapability {
  waitForIdle(windowId: number): Promise<boolean>;
  nativeClose(windowId: number, mutableVisibility: boolean): Promise<boolean>;
  showWindow(windowId: number): Promise<boolean>;
  windowState(windowId: number): string;
  clickByRole(windowId: number, role: string, label: string): Promise<boolean>;
  inputByRole(
    windowId: number,
    role: string,
    label: string,
    input: string,
  ): Promise<boolean>;
  takeQueryResult(): string;
  finish(report: string): boolean;
}

declare module "@wabou/solid-renderer" {
  interface HostCapabilities {
    readonly test: NativeTestCapability;
  }
}

export interface TestContext {
  readonly page: TestPage;
  readonly window: {
    nativeClose(
      windowId: number,
      platform: "wayland" | "mutable-visibility",
    ): Promise<void>;
    show(windowId: number): Promise<void>;
    state(windowId: number): NativeWindowState | null;
  };
}

export interface TestPage {
  /** Bind subsequent locators and frame barriers to one logical window. */
  forWindow(windowId: number): TestPage;
  getByRole(role: SemanticRole, options: { name: string }): Locator;
  waitForIdle(): Promise<void>;
}

export type SemanticRole =
  | "button"
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
  | "table"
  | "row"
  | "cell"
  | "columnheader"
  | "rowheader"
  | "slider"
  | "label";

export interface Locator {
  readonly windowId: number;
  click(): Promise<void>;
  dragBy(deltaX: number, deltaY: number): Promise<void>;
  press(
    key: string,
    modifiers?: {
      shift?: boolean;
      control?: boolean;
      alt?: boolean;
      meta?: boolean;
    },
  ): Promise<void>;
  type(text: string): Promise<void>;
  paste(text: string): Promise<void>;
  ime(text: string): Promise<void>;
  wheel(deltaY: number, deltaX?: number): Promise<void>;
  waitFor(): Promise<void>;
  snapshot(): Promise<LocatorSnapshot>;
}

export interface LocatorSnapshot {
  name: string | null;
  value: string | null;
  disabled: boolean;
  checked: boolean | "mixed" | null;
  pressed: boolean | "mixed" | null;
  selected: boolean | null;
  expanded: boolean | null;
  focused: boolean;
}

export interface LocatorAssertionOptions {
  timeout?: number;
  interval?: number;
}

export type TestInput =
  | { type: "probe" }
  | { type: "drag"; deltaX: number; deltaY: number }
  | { type: "key"; key: string; modifiers: number }
  | { type: "text"; text: string }
  | { type: "paste"; text: string }
  | { type: "ime"; text: string }
  | { type: "wheel"; deltaX: number; deltaY: number };

export interface TestReport {
  passed: boolean;
  tests: Array<{ name: string; passed: boolean; error?: string }>;
  trace: TestAction[];
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
    }
  | {
      action: "inputByRole";
      windowId: number;
      role: SemanticRole;
      label: string;
      input: TestInput;
    };

type TestBody = (context: TestContext) => void | Promise<void>;
const tests: Array<{ name: string; body: TestBody }> = [];
const trace: TestAction[] = [];

function capability(): NativeTestCapability {
  const value = defaultHost.test;
  if (!value) throw new Error("@wabou/test requires `wabou test`");
  return value;
}

function createPage(windowId: number): TestPage {
  if (!Number.isSafeInteger(windowId) || windowId <= 0) {
    throw new RangeError(`invalid Wabou window id ${windowId}`);
  }
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
      const input = async (value: TestInput): Promise<void> => {
        trace.push({
          action: "inputByRole",
          windowId,
          role,
          label: options.name,
          input: value,
        });
        if (
          !(await capability().inputByRole(
            windowId,
            role,
            options.name,
            JSON.stringify(value),
          ))
        ) {
          throw new Error(
            `no enabled ${role} named ${JSON.stringify(options.name)}`,
          );
        }
      };
      const snapshot = async (): Promise<LocatorSnapshot> => {
        await input({ type: "probe" });
        const value = JSON.parse(
          capability().takeQueryResult(),
        ) as LocatorSnapshot | null;
        if (!value)
          throw new Error(
            `no semantic snapshot for ${role} named ${JSON.stringify(options.name)}`,
          );
        return value;
      };
      return {
        windowId,
        async click() {
          trace.push({
            action: "clickByRole",
            windowId,
            role,
            label: options.name,
          });
          if (!(await capability().clickByRole(windowId, role, options.name))) {
            throw new Error(
              `no enabled ${role} named ${JSON.stringify(options.name)}`,
            );
          }
        },
        dragBy(deltaX, deltaY) {
          return input({ type: "drag", deltaX, deltaY });
        },
        press(key, modifiers = {}) {
          const bits =
            (modifiers.shift ? 1 : 0) |
            (modifiers.control ? 2 : 0) |
            (modifiers.alt ? 4 : 0) |
            (modifiers.meta ? 8 : 0);
          return input({ type: "key", key, modifiers: bits });
        },
        type(text) {
          return input({ type: "text", text });
        },
        paste(text) {
          return input({ type: "paste", text });
        },
        ime(text) {
          return input({ type: "ime", text });
        },
        wheel(deltaY, deltaX = 0) {
          return input({ type: "wheel", deltaX, deltaY });
        },
        waitFor() {
          return input({ type: "probe" });
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
      trace.push({ action: "nativeClose", windowId, platform });
      if (!(await capability().nativeClose(windowId, platform !== "wayland"))) {
        throw new Error(
          `failed to enqueue native close for window ${windowId}`,
        );
      }
    },
    async show(windowId) {
      trace.push({ action: "showWindow", windowId });
      if (!(await capability().showWindow(windowId))) {
        throw new Error(`failed to enqueue show for window ${windowId}`);
      }
    },
    state(windowId) {
      return JSON.parse(
        capability().windowState(windowId),
      ) as NativeWindowState | null;
    },
  },
};

export function test(name: string, body: TestBody): void {
  tests.push({ name, body });
}

/** Register a previously recorded action trace as a behavior test. */
export function replay(actions: readonly TestAction[]): void {
  test("replay action trace", async ({ window }) => {
    await replayActions(actions, context.page, window);
  });
}

export function expect<T>(actual: T) {
  const locator = (): Locator => {
    if (!actual || typeof actual !== "object" || !("snapshot" in actual)) {
      throw new Error("this assertion requires a Wabou locator");
    }
    return actual as unknown as Locator;
  };
  const locatorSnapshot = async (): Promise<LocatorSnapshot> => {
    return locator().snapshot();
  };
  const eventually = async (
    assertion: (state: LocatorSnapshot) => string | null,
    options: LocatorAssertionOptions = {},
  ): Promise<void> => {
    const timeout = options.timeout ?? 1_000;
    const interval = options.interval ?? 16;
    const deadline = performance.now() + timeout;
    let diagnostic = "locator state did not match";
    do {
      await createPage(locator().windowId).waitForIdle();
      const state = await locatorSnapshot();
      const failure = assertion(state);
      if (failure === null) return;
      diagnostic = failure;
      if (performance.now() < deadline) {
        await new Promise<void>((resolve) => setTimeout(resolve, interval));
      }
    } while (performance.now() < deadline);
    throw new Error(`${diagnostic} after ${timeout}ms`);
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
    toHaveText(
      expected: string,
      options?: LocatorAssertionOptions,
    ): Promise<void> {
      return eventually((state) => {
        const value = state.value ?? state.name;
        return value === expected
          ? null
          : `expected locator text ${JSON.stringify(value)} to be ${JSON.stringify(expected)}`;
      }, options);
    },
    toHaveValue(
      expected: string,
      options?: LocatorAssertionOptions,
    ): Promise<void> {
      return eventually(
        (state) =>
          state.value === expected
            ? null
            : `expected locator value ${JSON.stringify(state.value)} to be ${JSON.stringify(expected)}`,
        options,
      );
    },
    toBeDisabled(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) => (state.disabled ? null : "expected locator to be disabled"),
        options,
      );
    },
    toBeEnabled(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) => (state.disabled ? "expected locator to be enabled" : null),
        options,
      );
    },
    toBeChecked(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.checked === true
            ? null
            : `expected locator to be checked, received ${JSON.stringify(state.checked)}`,
        options,
      );
    },
    toBeUnchecked(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.checked === false
            ? null
            : `expected locator to be unchecked, received ${JSON.stringify(state.checked)}`,
        options,
      );
    },
    toBeIndeterminate(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.checked === "mixed"
            ? null
            : `expected locator to be indeterminate, received ${JSON.stringify(state.checked)}`,
        options,
      );
    },
    toBeSelected(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.selected === true
            ? null
            : `expected locator to be selected, received ${JSON.stringify(state.selected)}`,
        options,
      );
    },
    toBeDeselected(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.selected === false
            ? null
            : `expected locator to be deselected, received ${JSON.stringify(state.selected)}`,
        options,
      );
    },
    toBeExpanded(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.expanded === true
            ? null
            : `expected locator to be expanded, received ${JSON.stringify(state.expanded)}`,
        options,
      );
    },
    toBeCollapsed(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.expanded === false
            ? null
            : `expected locator to be collapsed, received ${JSON.stringify(state.expanded)}`,
        options,
      );
    },
    toBePressed(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.pressed === true
            ? null
            : `expected locator to be pressed, received ${JSON.stringify(state.pressed)}`,
        options,
      );
    },
    toBeUnpressed(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) =>
          state.pressed === false
            ? null
            : `expected locator to be unpressed, received ${JSON.stringify(state.pressed)}`,
        options,
      );
    },
    toBeFocused(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) => (state.focused ? null : "expected locator to be focused"),
        options,
      );
    },
    toBeBlurred(options?: LocatorAssertionOptions): Promise<void> {
      return eventually(
        (state) => (state.focused ? "expected locator to be blurred" : null),
        options,
      );
    },
  };
}

expect.poll = function poll<T>(
  read: () => T,
  options: { timeout?: number; interval?: number } = {},
) {
  const timeout = options.timeout ?? 1_000;
  const interval = options.interval ?? 10;
  return {
    async toBe(expected: T): Promise<void> {
      const deadline = performance.now() + timeout;
      let actual = read();
      while (!Object.is(actual, expected) && performance.now() < deadline) {
        await new Promise<void>((resolve) => setTimeout(resolve, interval));
        actual = read();
      }
      if (!Object.is(actual, expected)) {
        throw new Error(
          `expected ${JSON.stringify(actual)} to become ${JSON.stringify(expected)}`,
        );
      }
    },
  };
};

async function run(): Promise<void> {
  const results: TestReport["tests"] = [];
  for (const entry of tests) {
    try {
      await entry.body(context);
      results.push({ name: entry.name, passed: true });
    } catch (error) {
      results.push({
        name: entry.name,
        passed: false,
        error:
          error instanceof Error
            ? `${error.message}${error.stack ? `\n${error.stack}` : ""}`
            : String(error),
      });
    }
  }
  const report: TestReport = {
    passed: results.every((result) => result.passed),
    tests: results,
    trace,
  };
  capability().finish(JSON.stringify(report));
}

queueMicrotask(() => {
  void run();
});
