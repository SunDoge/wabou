import { expect, test } from "bun:test";
import schema from "../effect-abi.json";
import { EFFECT_ABI_VERSION, effectOps } from "./generated/effect-abi";

test("generated Effect ABI constants exactly match the shared schema", () => {
  expect(Number(EFFECT_ABI_VERSION)).toBe(Number(schema.abiVersion));
  expect(effectOps as unknown).toEqual(
    Object.fromEntries(
      schema.operations.map(({ name, capability, method }) => [
        name,
        { capability, method },
      ]),
    ),
  );
});
