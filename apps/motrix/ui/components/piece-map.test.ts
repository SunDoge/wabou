import { expect, test } from "bun:test";
import { decodePieceStates } from "./piece-map-model";

test("piece bitfields use the highest bit as piece zero", () => {
  expect(decodePieceStates("a4", 8)).toEqual([
    true,
    false,
    true,
    false,
    false,
    true,
    false,
    false,
  ]);
});

test("large piece maps are bounded without losing completed majorities", () => {
  const states = decodePieceStates("f".repeat(1_000), 4_000);
  expect(states.length).toBeLessThanOrEqual(1_200);
  expect(states.every(Boolean)).toBe(true);
});
