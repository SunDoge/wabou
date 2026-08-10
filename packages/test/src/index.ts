import { defaultHost } from "@wabou/solid-renderer";

export interface NativeWindowState {
  presence: "visible" | "hidden" | "surface-released" | "closed";
  surfaceGeneration: number;
}

interface NativeTestCapability {
  nativeClose(windowId: number, mutableVisibility: boolean): Promise<boolean>;
  showWindow(windowId: number): Promise<boolean>;
  windowState(windowId: number): string;
  clickByRole(windowId: number, role: string, label: string): Promise<boolean>;
  finish(report: string): boolean;
}

declare module "@wabou/solid-renderer" {
  interface HostCapabilities {
    readonly test: NativeTestCapability;
  }
}

export interface TestContext {
  readonly page: {
    getByRole(role: SemanticRole, options: { name: string }): Locator;
  };
  readonly window: {
    nativeClose(
      windowId: number,
      platform: "wayland" | "mutable-visibility",
    ): Promise<void>;
    show(windowId: number): Promise<void>;
    state(windowId: number): NativeWindowState | null;
  };
}

export type SemanticRole = "button" | "textbox" | "link" | "dialog" | "label";

export interface Locator {
  click(): Promise<void>;
}

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
    };

type TestBody = (context: TestContext) => void | Promise<void>;
const tests: Array<{ name: string; body: TestBody }> = [];
const trace: TestAction[] = [];

function capability(): NativeTestCapability {
  const value = defaultHost.test;
  if (!value) throw new Error("@wabou/test requires `wabou test`");
  return value;
}

const context: TestContext = {
  page: {
    getByRole(role, options) {
      return {
        async click() {
          trace.push({
            action: "clickByRole",
            windowId: 1,
            role,
            label: options.name,
          });
          if (!(await capability().clickByRole(1, role, options.name))) {
            throw new Error(
              `no enabled ${role} named ${JSON.stringify(options.name)}`,
            );
          }
        },
      };
    },
  },
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
    for (const action of actions) {
      if (action.action === "nativeClose") {
        await window.nativeClose(action.windowId, action.platform);
      } else if (action.action === "showWindow") {
        await window.show(action.windowId);
      } else {
        await context.page
          .getByRole(action.role, { name: action.label })
          .click();
      }
    }
  });
}

export function expect<T>(actual: T) {
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
