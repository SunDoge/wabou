import { defineWabouTestConfig } from "@wabou/vite/test";
import { defineConfig, mergeConfig } from "vitest/config";

export default mergeConfig(
  defineWabouTestConfig(),
  defineConfig({
    test: {
      include: ["tests/component/**/*.vitest.{ts,tsx}"],
      // The component harness installs one process-global Wabou host and
      // renderer. Keep component files sequential so isolated workers cannot
      // race those globals during a cold dependency transform.
      maxWorkers: 1,
    },
  }),
);
