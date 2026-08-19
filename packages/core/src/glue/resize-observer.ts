import { type NodeKey, NodeKeyTable } from "../protocol";

interface WabouResizeObserverEntry {
  target: { id: NodeKey };
  contentRect: { width: number; height: number };
}

type ResizeCallback = (entries: WabouResizeObserverEntry[]) => void;
const observers = new NodeKeyTable<{
  target: { id: NodeKey };
  callbacks: Set<ResizeCallback>;
}>();

class WabouResizeObserver {
  private readonly targets = new Set<NodeKey>();

  constructor(private readonly callback: ResizeCallback) {}

  observe(target: { id: NodeKey }): void {
    const id = target.id;
    if (this.targets.has(id)) return;
    this.targets.add(id);
    let observed = observers.get(id);
    if (!observed) {
      observed = { target, callbacks: new Set() };
      observers.set(id, observed);
      __wabou_resize_observe(id.lo, id.hi);
    }
    observed.callbacks.add(this.callback);
  }

  unobserve(target: { id: NodeKey }): void {
    this.remove(target.id);
  }

  disconnect(): void {
    for (const id of this.targets) this.remove(id);
  }

  private remove(id: NodeKey): void {
    if (!this.targets.delete(id)) return;
    const observed = observers.get(id);
    observed?.callbacks.delete(this.callback);
    if (observed?.callbacks.size === 0) {
      observers.delete(id);
      __wabou_resize_unobserve(id.lo, id.hi);
    }
  }
}

export function dispatchResizeObservation(
  solidId: NodeKey,
  width: number,
  height: number,
): void {
  const observed = observers.get(solidId);
  if (!observed) return;
  const entry = { target: observed.target, contentRect: { width, height } };
  for (const callback of observed.callbacks) {
    try {
      callback([entry]);
    } catch (error: any) {
      __wabou_log("error", error?.stack ? String(error.stack) : String(error));
    }
  }
}

(globalThis as any).ResizeObserver = WabouResizeObserver;
