import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import schema from "../host-abi.json";
import { BROWSER_HOST_BINDING_NAMES } from "./generated/browser-host-bindings";

test("generated TypeScript declarations contain exactly the Host ABI schema", async () => {
  const source = await readFile(
    new URL("./generated/host-abi.ts", import.meta.url),
    "utf8",
  );
  const declared = Array.from(
    source.matchAll(/(?:const|function) (__wabou_[a-z0-9_]+)/g),
    (match) => match[1],
  ).sort();
  const expected = schema.entries.map((entry) => entry.name).sort();

  expect(declared).toEqual(expected);
});

test("browser bindings are generated from the Host ABI schema", () => {
  const expected = schema.entries
    .filter((entry) => "browserMethod" in entry)
    .map((entry) => entry.name);
  const actual = Array.from(BROWSER_HOST_BINDING_NAMES) as string[];
  expect(actual).toEqual(expected);
});

test("QuickJS registrations use the generated HostBinding installer", async () => {
  for (const relative of [
    "../../../crates/wabou-runtime/src/jsrt.rs",
    "../../../crates/wabou-runtime/src/effect_bridge.rs",
  ]) {
    const source = await readFile(new URL(relative, import.meta.url), "utf8");
    const productionSource = source.split("\n#[cfg(test)]")[0] ?? source;
    expect(productionSource).not.toMatch(/\.(?:set|with_name)\(\s*"__wabou_/);
    expect(productionSource).not.toMatch(
      /\.set\(\s*(?:crate::host_abi::)?HostBinding::/,
    );
  }
});
