import { expect, test } from "@wabou/test";

test("palette host module calls Rust directly", ({ host }) => {
  const response = host.bindingsDemo.describePalette({
    name: "ocean",
    swatchCount: 3,
  });
  expect(response.title).toBe("ocean palette");
  expect(response.swatches).toEqual(["ocean-1", "ocean-2", "ocean-3"]);
});
