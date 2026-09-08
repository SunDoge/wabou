import { describe, expect, test } from "vitest";
import { createLiquidOrbUniforms } from "./liquid-orb";

describe("liquid orb uniform presets", () => {
  test("pack distinct style programs into the shared shader contract", () => {
    const siri = createLiquidOrbUniforms("siri");
    const blueDrop = createLiquidOrbUniforms("blueDrop");
    const refractiveBlob = createLiquidOrbUniforms("refractiveBlob");

    expect(siri).toHaveLength(136);
    expect(blueDrop).toHaveLength(136);
    expect(refractiveBlob).toHaveLength(136);
    expect([siri[15], blueDrop[15], refractiveBlob[15]]).toEqual([9, 20, 23]);
    expect(siri.slice(40, 44)).not.toEqual(blueDrop.slice(40, 44));
    expect(blueDrop.slice(40, 44)).not.toEqual(refractiveBlob.slice(40, 44));
  });
});
