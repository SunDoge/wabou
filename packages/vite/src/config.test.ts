import { expect, test } from "bun:test";
import { defineConfig, rustCapability } from "./config";

test("Rust contracts supply capability identity, version and methods", () => {
  const contract = { name: "demo", version: 2, methods: [{ name: "tick", hasRequest: false, json: true }] } as const;
  const capability = rustCapability({ contract, manifest: "./functions/Cargo.toml" });
  expect(capability).toEqual({ ...contract, implementation: { kind: "project", manifest: "./functions/Cargo.toml", watch: [], features: [] } });
  expect(defineConfig({ capabilities: [capability] }).capabilities[0].version).toBe(2);
});

test("async configuration factories remain ordinary TS functions", async () => {
  const config = defineConfig(async () => ({ window: { title: "Async" } }));
  expect((await config()).window.title).toBe("Async");
});
