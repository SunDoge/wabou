// Timer identity and cancellation stay in JavaScript. Rust only provides an
// async sleep primitive, so rquickjs owns Promise scheduling and Tokio owns the
// actual deadline.

let nextTimerId = 1;
const active = new Set<number>();

function schedule(
  callback: (...args: any[]) => void,
  delay: number,
  repeat: boolean,
  args: any[],
): number {
  const id = nextTimerId++;
  active.add(id);
  const run = async (): Promise<void> => {
    await __wabou_sleep(delay);
    if (!active.has(id)) return;
    try {
      callback(...args);
    } catch (error: any) {
      __wabou_log("error", error?.stack ? String(error.stack) : String(error));
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
