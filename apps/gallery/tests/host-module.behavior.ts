import { expect, test } from "@wabou/test";
import { useHost } from "@wabou/ui";

test("palette host module calls Rust directly", () => {
  const response = useHost().bindingsDemo.describePalette({
    name: "ocean",
    swatchCount: 3,
  });
  expect(response.title).toBe("ocean palette");
  expect(response.swatches).toEqual(["ocean-1", "ocean-2", "ocean-3"]);
});
