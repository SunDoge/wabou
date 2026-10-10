import { expect, test } from "@wabou/test";
import { useBindingsDemoClient } from "../ui/generated/host-bindings";

test("palette capability uses the generated client in the real host", async () => {
  const client = useBindingsDemoClient();
  const response = await client.describePalette({
    name: "ocean",
    swatchCount: 3,
  });
  expect(response.status).toBe("palette");
  if (response.status !== "palette") throw new Error(response.message);
  expect(response.title).toBe("ocean palette");
  expect(response.swatches).toEqual(["ocean-1", "ocean-2", "ocean-3"]);
});
