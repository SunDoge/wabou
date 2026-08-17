//#region src/index.d.ts
interface NativeWindowState {
  presence: "visible" | "hidden" | "surface-released" | "closed";
  surfaceGeneration: number;
}
interface NativeTestCapability {
  nativeClose(windowId: number, mutableVisibility: boolean): Promise<boolean>;
  showWindow(windowId: number): Promise<boolean>;
  windowState(windowId: number): string;
  clickByRole(windowId: number, role: string, label: string): Promise<boolean>;
  inputByRole(windowId: number, role: string, label: string, input: string): Promise<boolean>;
  finish(report: string): boolean;
}
declare module "@wabou/solid-renderer" {
  interface HostCapabilities {
    readonly test: NativeTestCapability;
  }
}
interface TestContext {
  readonly page: {
    getByRole(role: SemanticRole, options: {
      name: string;
    }): Locator;
  };
  readonly window: {
    nativeClose(windowId: number, platform: "wayland" | "mutable-visibility"): Promise<void>;
    show(windowId: number): Promise<void>;
    state(windowId: number): NativeWindowState | null;
  };
}
type SemanticRole = "button" | "textbox" | "link" | "dialog" | "alert" | "status" | "checkbox" | "radio" | "switch" | "combobox" | "listbox" | "option" | "label";
interface Locator {
  click(): Promise<void>;
  dragBy(deltaX: number, deltaY: number): Promise<void>;
  press(key: string, modifiers?: {
    shift?: boolean;
    control?: boolean;
    alt?: boolean;
    meta?: boolean;
  }): Promise<void>;
  type(text: string): Promise<void>;
  paste(text: string): Promise<void>;
  ime(text: string): Promise<void>;
  wheel(deltaY: number, deltaX?: number): Promise<void>;
  waitFor(): Promise<void>;
}
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
interface TestReport {
  passed: boolean;
  tests: Array<{
    name: string;
    passed: boolean;
    error?: string;
  }>;
  trace: TestAction[];
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
} | {
  action: "inputByRole";
  windowId: number;
  role: SemanticRole;
  label: string;
  input: TestInput;
};
type TestBody = (context: TestContext) => void | Promise<void>;
declare function test(name: string, body: TestBody): void;
/** Register a previously recorded action trace as a behavior test. */
declare function replay(actions: readonly TestAction[]): void;
declare function expect<T>(actual: T): {
  toBe(expected: T): void;
  toEqual(expected: T): void;
};
declare namespace expect {
  var poll: <T>(read: () => T, options?: {
    timeout?: number;
    interval?: number;
  }) => {
    toBe(expected: T): Promise<void>;
  };
}
//#endregion
export { Locator, NativeWindowState, SemanticRole, TestAction, TestContext, TestInput, TestReport, expect, replay, test };
//# sourceMappingURL=index.d.mts.map