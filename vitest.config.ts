import { defineWabouTestConfig } from "@wabou/vite/test";
import { defineConfig, mergeConfig } from "vitest/config";

export default mergeConfig(
  defineWabouTestConfig(),
  defineConfig({
    test: {
      include: ["tests/component/**/*.vitest.{ts,tsx}"],
    },
  }),
);
