import "./host";
interface Measurement {
  name: string;
  durationUs: number;
  end: number;
  correlation: number;
  asynchronous: boolean;
}
const pending: Measurement[] = [];
let scheduled = false;
function enabled() {
  return (
    typeof __wabou_profile_enabled === "function" && __wabou_profile_enabled()
  );
}
/** Flush completed user measurements in one host call. */
export function flushPerformance(): void {
  scheduled = false;
  if (!pending.length) return;
  const now = performance.now();
  const batch = pending.splice(0, 128).map(({ end, ...event }) => ({
    ...event,
    ageUs: Math.max(0, Math.round((now - end) * 1000)),
  }));
  __wabou_profile_batch(JSON.stringify(batch));
}
function record(
  name: string,
  start: number,
  correlation: number,
  asynchronous: boolean,
) {
  const end = performance.now();
  pending.push({
    name: name
      .slice(0, 128)
      .replace(
        /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/g,
        "\ufffd",
      ),
    durationUs: Math.max(0, Math.round((end - start) * 1000)),
    end,
    correlation:
      Number.isSafeInteger(correlation) && correlation >= 0 ? correlation : 0,
    asynchronous,
  });
  if (pending.length === 128) flushPerformance();
  else if (!scheduled) {
    scheduled = true;
    Promise.resolve().then(flushPerformance);
  }
}
/** Measure synchronous application work without changing its return/error behavior. */
export function measurePerformance<T>(
  name: string,
  operation: () => T,
  correlation = 0,
): T {
  if (!enabled()) return operation();
  const start = performance.now();
  try {
    return operation();
  } finally {
    record(name, start, correlation, false);
  }
}
/** Record async elapsed time separately from CPU execution tracks. */
export async function measureAsyncPerformance<T>(
  name: string,
  operation: () => Promise<T>,
  correlation = 0,
): Promise<T> {
  if (!enabled()) return operation();
  const start = performance.now();
  try {
    return await operation();
  } finally {
    record(name, start, correlation, true);
  }
}
