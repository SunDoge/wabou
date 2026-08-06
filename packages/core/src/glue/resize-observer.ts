interface WabouResizeObserverEntry {
  target: { id: number };
  contentRect: { width: number; height: number };
}

type ResizeCallback = (entries: WabouResizeObserverEntry[]) => void;
const observers = new Map<
  number,
  { target: { id: number }; callbacks: Set<ResizeCallback> }
>();

class WabouResizeObserver {
  private readonly targets = new Set<number>();

  constructor(private readonly callback: ResizeCallback) {}

  observe(target: { id: number }): void {
    const id = target.id;
    if (this.targets.has(id)) return;
    this.targets.add(id);
    let observed = observers.get(id);
    if (!observed) {
      observed = { target, callbacks: new Set() };
      observers.set(id, observed);
      __wabou_resize_observe(id);
    }
    observed.callbacks.add(this.callback);
  }

  unobserve(target: { id: number }): void {
    this.remove(target.id);
  }

  disconnect(): void {
    for (const id of this.targets) this.remove(id);
  }

  private remove(id: number): void {
    if (!this.targets.delete(id)) return;
    const observed = observers.get(id);
    observed?.callbacks.delete(this.callback);
    if (observed?.callbacks.size === 0) {
      observers.delete(id);
      __wabou_resize_unobserve(id);
    }
  }
}

export function dispatchResizeObservation(
  solidId: number,
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

export {};
