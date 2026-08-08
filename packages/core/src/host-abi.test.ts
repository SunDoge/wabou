import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import schema from "../host-abi.json";

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
