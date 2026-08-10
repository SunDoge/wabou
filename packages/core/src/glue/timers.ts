// Timer identity and cancellation stay in JavaScript. Rust only provides an
// async sleep primitive, so rquickjs owns Promise scheduling and Tokio owns the
// actual deadline.

let nextTimerId = 1;
const active = new Set<number>();
const nativeSetTimeout = globalThis.setTimeout?.bind(globalThis);

function sleep(delay: number): Promise<void> {
  if (typeof __wabou_sleep === "function") {
    return __wabou_sleep(delay);
  }
  if (nativeSetTimeout) {
    return new Promise((resolve) => nativeSetTimeout(resolve, delay));
  }
  return Promise.reject(new Error("Wabou timer host is unavailable"));
}

function reportTimerError(error: unknown): void {
  const message =
    error instanceof Error && error.stack ? error.stack : String(error);
  if (typeof __wabou_log === "function") {
    __wabou_log("error", message);
  } else {
    console.error(message);
  }
}

function schedule(
  callback: (...args: any[]) => void,
  delay: number,
  repeat: boolean,
  args: any[],
): number {
  const id = nextTimerId++;
  active.add(id);
  const run = async (): Promise<void> => {
    await sleep(delay);
    if (!active.has(id)) return;
    try {
      callback(...args);
    } catch (error: unknown) {
      reportTimerError(error);
    }
    if (repeat && active.has(id)) {
      void run();
    } else {
      active.delete(id);
    }
  };
  void run();
  return id;
}

(globalThis as any).setTimeout = (
  callback: (...args: any[]) => void,
  delay = 0,
  ...args: any[]
): number => schedule(callback, Number(delay) || 0, false, args);

(globalThis as any).setInterval = (
  callback: (...args: any[]) => void,
  delay = 0,
  ...args: any[]
): number => schedule(callback, Number(delay) || 0, true, args);

function clearTimer(id: number): void {
  active.delete(id);
}

(globalThis as any).clearTimeout = clearTimer;
(globalThis as any).clearInterval = clearTimer;

export {};
