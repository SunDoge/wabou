import { expect, test } from "bun:test";
import { defineConfig } from "./config";

test("defines frontend-only window and UI configuration", () => {
  const config = defineConfig({
    window: { title: "Example", width: 960, height: 640, decorations: false },
    ui: { entry: "ui/main.tsx" },
  });
  expect(config.window?.width).toBe(960);
  expect(config.ui?.entry).toBe("ui/main.tsx");
});

test("accepts an asynchronous configuration factory", async () => {
  const factory = defineConfig(async () => ({
    window: { title: "Async" },
  }));
  expect((await factory()).window?.title).toBe("Async");
});
