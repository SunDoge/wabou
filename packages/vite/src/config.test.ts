import { expect, test } from "bun:test";
import { defineConfig } from "./config";

test("defines one project configuration for Vite and the CLI", async () => {
  const config = defineConfig({
    window: { title: "Example", width: 960, height: 640, decorations: false },
    ui: { entry: "ui/main.tsx" },
  });
  expect(typeof config).toBe("function");
  if (typeof config !== "function")
    throw new Error("expected Vite config factory");
  const resolved = await config({ command: "build", mode: "production" });
  expect(resolved.build?.lib).toMatchObject({ entry: "ui/main.tsx" });
});

test("accepts an asynchronous environment-dependent configuration factory", async () => {
  const factory = defineConfig(async () => ({
    window: { title: "Async" },
  }));
  if (typeof factory !== "function")
    throw new Error("expected Vite config factory");
  const resolved = await factory({ command: "serve", mode: "development" });
  expect(resolved.build?.minify).toBe(false);
});
