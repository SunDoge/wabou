import { expect, test } from "bun:test";
import { createSignal, DEV, flush, type JSX, onCleanup } from "solid-js";
import { $$component, $$refresh, $$registry } from "solid-js/refresh";
import { ROOT_NODE_KEY } from "../protocol";
import {
  createComponent,
  createElement,
  type Handle,
  registerRoot,
  render,
  runSweep,
  writer,
} from "./index";

const refresh = $$refresh as (...args: unknown[]) => void;
const refreshComponent = $$component as unknown as <Props>(
  registry: ReturnType<typeof $$registry>,
  id: string,
  component: (props: Props) => Handle,
) => (props: Props) => Handle;
const renderComponent = createComponent as unknown as <Props>(
  component: (props: Props) => Handle,
  props: Props,
) => Handle;

type AcceptCallback = (module: unknown) => void;
type DisposeCallback = (data: Record<string, unknown>) => void;

class ViteHotContext {
  readonly data: Record<string, unknown>;
  accepted: AcceptCallback[] = [];
  disposed: DisposeCallback[] = [];
  invalidated = false;

  constructor(data: Record<string, unknown> = {}) {
    this.data = data;
  }

  accept(callback?: AcceptCallback): void {
    if (callback) this.accepted.push(callback);
  }

  dispose(callback: DisposeCallback): void {
    this.disposed.push(callback);
  }

  invalidate(): void {
    this.invalidated = true;
  }

  decline(): void {
    this.invalidated = true;
  }
}

function rootHandle(): Handle {
  return {
    id: ROOT_NODE_KEY,
    tag: "#root",
    parent: null,
    firstChild: null,
    lastChild: null,
    prev: null,
    next: null,
    focus() {},
    blur() {},
    scrollTo() {},
    scrollBy() {},
    setTextSelection() {},
    selectAll() {},
    undo() {},
    redo() {},
  };
}

test.skipIf(!DEV)(
  "Solid refresh replaces a component without remounting its parent",
  () => {
    const root = rootHandle();
    registerRoot(root);
    let parentRuns = 0;
    let readParentState: (() => number) | undefined;
    let writeParentState: ((value: number) => number) | undefined;

    const oldRegistry = $$registry();
    const OldChild = refreshComponent<Record<string, never>>(
      oldRegistry,
      "Child",
      () => createElement("old-child"),
    );
    const oldHot = new ViteHotContext();
    refresh("vite", oldHot, oldRegistry);

    const dispose = render(() => {
      parentRuns++;
      [readParentState, writeParentState] = createSignal(0);
      return renderComponent(OldChild, {}) as unknown as JSX.Element;
    }, root);
    writer.flush();
    writeParentState?.(7);
    flush();

    expect(root.firstChild?.tag).toBe("old-child");
    expect(parentRuns).toBe(1);

    const newRegistry = $$registry();
    refreshComponent<Record<string, never>>(newRegistry, "Child", () =>
      createElement("new-child"),
    );
    const newHot = new ViteHotContext(oldHot.data);
    refresh("vite", newHot, newRegistry);

    for (const accept of oldHot.accepted) accept({});
    flush();
    runSweep();
    writer.flush();

    expect(oldHot.invalidated).toBe(false);
    expect(root.firstChild?.tag).toBe("new-child");
    expect(parentRuns).toBe(1);
    expect(readParentState?.()).toBe(7);

    dispose();
    runSweep();
    writer.flush();
  },
);

test("cache-busted renderer instances share one application mount", async () => {
  const revision = Date.now();
  const first = await import(`./index.ts?renderer=${revision}-first`);
  let firstCleanups = 0;
  const disposeFirst = first.mount(() => {
    onCleanup(() => firstCleanups++);
    return first.createElement("first-app") as unknown as JSX.Element;
  });
  const firstRoot = first.getMountRoot();
  first.writer.flush();
  expect(firstRoot.firstChild?.tag).toBe("first-app");

  const second = await import(`./index.ts?renderer=${revision}-second`);
  const disposeSecond = second.mount(
    () => second.createElement("second-app") as unknown as JSX.Element,
  );
  const secondRoot = second.getMountRoot();
  second.writer.flush();

  expect(secondRoot).not.toBe(firstRoot);
  expect(firstCleanups).toBe(1);
  expect(secondRoot.firstChild?.tag).toBe("second-app");

  // A stale disposer from the first graph must not tear down the active graph.
  disposeFirst();
  expect(second.getMountRoot()).toBe(secondRoot);
  expect(secondRoot.firstChild?.tag).toBe("second-app");
  disposeSecond();
});
