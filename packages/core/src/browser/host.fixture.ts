// Isolated lifecycle contract: no real page or ambient browser globals are modified.
import { strict as assert } from "node:assert";

const frames = new Map<number, FrameRequestCallback>();
let nextFrame = 0;
class Observer {
  disconnected = false;
  observe() {}
  disconnect() {
    this.disconnected = true;
  }
}
const observers: Observer[] = [];
class Canvas extends EventTarget {
  clientWidth = 240;
  clientHeight = 160;
  clientLeft = 0;
  clientTop = 0;
  getBoundingClientRect() {
    return { left: 0, top: 0 };
  }
}
const canvas = new Canvas() as unknown as HTMLCanvasElement;
const previousIntern = () => 77;
Object.assign(globalThis, {
  document: { nodeType: 9 },
  devicePixelRatio: 1,
  requestAnimationFrame: (callback: FrameRequestCallback) => {
    const id = ++nextFrame;
    frames.set(id, callback);
    return id;
  },
  cancelAnimationFrame: (id: number) => frames.delete(id),
  ResizeObserver: class extends Observer {
    constructor() {
      super();
      observers.push(this);
    }
  },
  __wabou_intern: previousIntern,
});
const { startBrowserApp } = await import("./host");
const { mount, createElement, writer } = await import("../renderer");
class Runtime {
  atoms: string[] = [];
  frees = 0;
  renders = 0;
  target = "";
  scrolls: number[][] = [];
  acceptsScroll = true;
  intern(value: string) {
    this.atoms.push(value);
    return this.atoms.length;
  }
  set_stylesheet(_bytes: Uint8Array) {}
  load_font() {}
  apply() {}
  tick_motion() {
    return '{"active":false,"completed":[]}';
  }
  motion_create() {
    return 1;
  }
  motion_sample(_id: number, _elapsed: number, output: Float64Array) {
    output.set([1, 0, 1]);
    return true;
  }
  motion_drop() {
    return true;
  }
  render() {
    this.renders++;
  }
  hit_test() {
    return this.target;
  }
  scroll(x: number, y: number, delta: number) {
    this.scrolls.push([x, y, delta]);
    return this.acceptsScroll;
  }
  snapshot() {
    return "[]";
  }
  free() {
    this.frees++;
  }
}
const options = (runtime: Runtime) => ({
  canvas,
  createRuntime: () => runtime,
  entry: async () => {
    mount(() => createElement("view"));
  },
});
const tick = () => {
  const next = frames.entries().next().value;
  if (!next) throw new Error("Expected a scheduled host frame");
  const [id, callback] = next;
  frames.delete(id);
  callback(16);
};
const first = new Runtime();
const dispose = await startBrowserApp(options(first));
await assert.rejects(
  startBrowserApp(options(new Runtime())),
  /Only one Wabou browser host/,
);
tick();
assert.equal(first.renders, 1);
assert.deepEqual(first.atoms, ["view"]);
dispose();
dispose();
assert.equal(first.frees, 1);
assert.equal(frames.size, 0);
assert.equal(observers[0].disconnected, true);
assert.equal(
  (globalThis as unknown as Record<string, unknown>).__wabou_intern,
  previousIntern,
);
assert.equal(writer.flush(), null);
const second = new Runtime();
const disposeSecond = await startBrowserApp(options(second));
tick();
assert.deepEqual(
  second.atoms,
  ["view"],
  "new host must receive atoms from a previously used tag",
);
disposeSecond();
const failed = new Runtime();
await assert.rejects(
  startBrowserApp({
    ...options(failed),
    entry: async () => {
      mount(() => createElement("view"));
      throw new Error("entry failed");
    },
  }),
  /entry failed/,
);
assert.equal(failed.frees, 1);
assert.equal(frames.size, 0);
assert.equal(writer.flush(), null);
const fontFailure = new Runtime();
globalThis.fetch = Object.assign(
  async () => new Response("missing", { status: 404 }),
  {
    preconnect: (..._args: Parameters<typeof fetch.preconnect>) => {},
  },
);
await assert.rejects(
  startBrowserApp({ ...options(fontFailure), fonts: ["missing.ttf"] }),
  /Font failed/,
);
assert.equal(fontFailure.frees, 1);
assert.equal(
  (globalThis as unknown as Record<string, unknown>).__wabou_intern,
  previousIntern,
);
const recovered = new Runtime();
const disposeRecovered = await startBrowserApp(options(recovered));
tick();
disposeRecovered();
assert.equal(recovered.frees, 1);
const pointerRuntime = new Runtime();
let enters = 0;
let leaves = 0;
const disposePointer = await startBrowserApp({
  ...options(pointerRuntime),
  entry: async () => {
    mount(() => {
      const node = createElement("view", {
        onPointerEnter: () => enters++,
        onPointerLeave: () => leaves++,
      });
      pointerRuntime.target = `${node.id.lo},${node.id.hi}`;
      return node;
    });
  },
});
const moveEvent = () => {
  const event = new Event("pointermove");
  Object.assign(event, {
    clientX: 10,
    clientY: 10,
    button: 0,
    buttons: 0,
    pointerType: "mouse",
  });
  canvas.dispatchEvent(event);
};
moveEvent();
moveEvent();
assert.equal(enters, 1, "remaining over one target must not re-enter it");
pointerRuntime.target = "";
moveEvent();
assert.equal(leaves, 1, "moving into empty canvas must leave the target");
disposePointer();
moveEvent();
assert.equal(enters, 1, "disposed hosts must remove pointer listeners");
const scrollRuntime = new Runtime();
const disposeScroll = await startBrowserApp(options(scrollRuntime));
const wheelEvent = () => {
  const event = new Event("wheel", { cancelable: true });
  Object.assign(event, { clientX: 12, clientY: 34, deltaY: 3, deltaMode: 1 });
  canvas.dispatchEvent(event);
  return event;
};
assert.equal(wheelEvent().defaultPrevented, true);
assert.deepEqual(
  scrollRuntime.scrolls,
  [[12, 34, 48]],
  "line wheel deltas must be normalized to logical pixels",
);
scrollRuntime.acceptsScroll = false;
assert.equal(
  wheelEvent().defaultPrevented,
  false,
  "the page may scroll when the native viewport reaches its boundary",
);
disposeScroll();
wheelEvent();
assert.equal(
  scrollRuntime.scrolls.length,
  2,
  "disposed hosts must remove wheel listeners",
);
const hidden = new Runtime();
const size = canvas as unknown as Canvas;
size.clientWidth = 0;
const disposeHidden = await startBrowserApp(options(hidden));
tick();
assert.equal(
  hidden.renders,
  0,
  "hidden canvas must not dispose its application",
);
size.clientWidth = 240;
tick();
assert.equal(hidden.renders, 1);
disposeHidden();
const readyDisposal = new Runtime();
const disposeOnReady = await startBrowserApp(options(readyDisposal));
canvas.addEventListener("wabou:ready", disposeOnReady, { once: true });
tick();
assert.equal(readyDisposal.frees, 1);
assert.equal(
  frames.size,
  0,
  "ready listener disposal must not enqueue another frame",
);
console.log(
  "browser host lifecycle: exclusive ownership, idempotent disposal, restart, entry/font rollback passed",
);
