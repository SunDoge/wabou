import "./overview.behavior";
import { test } from "@wabou/test";
import {
  measurePerformance,
  measureAsyncPerformance,
  flushPerformance,
} from "@wabou/ui/performance";

test("application measurements preserve results in the native runtime", async () => {
  const sum = measurePerformance(
    "fixture.compute",
    () => {
      let result = 0;
      for (let i = 0; i < 10000; i++) result += i;
      return result;
    },
    81,
  );
  if (sum !== 49995000) throw new Error("measurement altered result");
  await measureAsyncPerformance(
    "fixture.wait",
    () => new Promise<void>((resolve) => setTimeout(resolve, 2)),
  );
  flushPerformance();
});
