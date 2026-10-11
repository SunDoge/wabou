import { expect, test } from "bun:test";
import {
  flushPerformance,
  measureAsyncPerformance,
  measurePerformance,
} from "./performance";

test("measurements batch transport and preserve results and thrown values", async () => {
  const runtime = globalThis as unknown as Record<string, any>;
  const enabled = runtime.__wabou_profile_enabled;
  const submit = runtime.__wabou_profile_batch;
  const batches: any[][] = [];
  try {
    runtime.__wabou_profile_enabled = () => true;
    runtime.__wabou_profile_batch = (raw: string) =>
      batches.push(JSON.parse(raw));
    expect(measurePerformance("compute", () => 42, 7)).toBe(42);
    const error = new Error("failure");
    expect(() =>
      measurePerformance("fail", () => {
        throw error;
      }),
    ).toThrow(error);
    flushPerformance();
    expect(batches[0].map((event) => event.name)).toEqual(["compute", "fail"]);
    expect(batches[0][0].correlation).toBe(7);
    expect(batches[0][0].durationUs).toBeGreaterThanOrEqual(0);
    expect(await measureAsyncPerformance("load", async () => 9)).toBe(9);
    flushPerformance();
    expect(
      batches.flat().find((event) => event.name === "load").asynchronous,
    ).toBe(true);
    runtime.__wabou_profile_enabled = () => false;
    expect(measurePerformance("off", () => 3)).toBe(3);
    flushPerformance();
    expect(batches.flat().some((event) => event.name === "off")).toBe(false);
  } finally {
    if (enabled) runtime.__wabou_profile_enabled = enabled;
    else Reflect.deleteProperty(runtime, "__wabou_profile_enabled");
    if (submit) runtime.__wabou_profile_batch = submit;
    else Reflect.deleteProperty(runtime, "__wabou_profile_batch");
  }
});
