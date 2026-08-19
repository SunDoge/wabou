import { expect, test } from "bun:test";
import schema from "../effect-abi.json";
import { EFFECT_ABI_VERSION, effectOps } from "./generated/effect-abi";

test("generated Effect ABI constants exactly match the shared schema", () => {
  expect(EFFECT_ABI_VERSION).toBe(schema.abiVersion);
  expect(effectOps).toEqual(
    Object.fromEntries(
      schema.operations.map(({ name, capability, method }) => [
        name,
        { capability, method },
      ]),
    ),
  );
});
