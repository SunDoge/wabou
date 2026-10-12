import { expect, test } from "bun:test";
import { defineConfig, rustCapability, rustWidget, rustPlugin } from "./config";

test("Rust contracts supply capability identity, version and methods", () => {
  const contract = {
    name: "demo",
    version: 2,
    methods: [{ name: "tick", hasRequest: false, json: true }],
  } as const;
  const capability = rustCapability({
    contract,
    manifest: "./functions/Cargo.toml",
  });
  expect(capability).toEqual({
    ...contract,
    implementation: {
      kind: "project",
      manifest: "./functions/Cargo.toml",
      watch: [],
      features: [],
    },
  });
  expect(
    defineConfig({ capabilities: [capability] }).capabilities[0].version,
  ).toBe(2);
});

test("async configuration factories remain ordinary TS functions", async () => {
  const config = defineConfig(async () => ({ window: { title: "Async" } }));
  expect((await config()).window.title).toBe("Async");
});

test("Rust widgets separate initial state from replaceable implementation", () => {
  const widget = rustWidget({
    tag: "reload-counter",
    version: 1,
    initialState: 40,
    manifest: "./widget/Cargo.toml",
    features: ["demo"],
  });
  expect(widget).toEqual({
    tag: "reload-counter",
    version: 1,
    initialState: 40,
    implementation: {
      kind: "project",
      manifest: "./widget/Cargo.toml",
      watch: [],
      features: ["demo"],
    },
  });
  expect(defineConfig({ widgets: [widget] }).widgets[0].initialState).toBe(40);
});

test("one Rust plugin registers capabilities and widgets with one build target", () => {
  const contract = {
    name: "counter",
    version: 1,
    methods: [{ name: "increment", hasRequest: false, json: false }],
  };
  const plugin = rustPlugin({
    name: "app",
    manifest: "./native/Cargo.toml",
    capabilities: [contract],
    widgets: [{ tag: "counter", version: 2, initialState: 40 }],
    targetDirectory: "./target/native",
  });
  expect(plugin.capabilities[0]).toBe(contract);
  expect(plugin.widgets[0].initialState).toBe(40);
  expect(plugin.implementation).toEqual({
    kind: "project",
    manifest: "./native/Cargo.toml",
    watch: [],
    features: [],
    targetDirectory: "./target/native",
  });
  expect(defineConfig({ plugins: [plugin] }).plugins).toHaveLength(1);
});
