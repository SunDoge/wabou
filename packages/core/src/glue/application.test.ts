import { expect, test } from "bun:test";
import { EFFECT_ABI_VERSION } from "../generated/effect-abi";

test("application lifecycle uses process-level native effects", async () => {
  const calls: unknown[][] = [];
  Object.assign(globalThis, {
    __wabou_effect_abi: EFFECT_ABI_VERSION,
    __wabou_effect_submit: (
      capability: number,
      method: number,
      json: string,
    ) => {
      calls.push([capability, method, JSON.parse(json)]);
      return 1;
    },
  });
  const { application } = await import("./application");

  application.exit();
  application.relaunch();

  expect(calls).toEqual([
    [7, 1, null],
    [7, 2, null],
  ]);
});
