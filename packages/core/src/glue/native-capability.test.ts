import { describe, expect, test } from "bun:test";
import { bindCapability, CapabilityError } from "./native-capability";

describe("native capability binding", () => {
  test("keeps direct structured methods intact", async () => {
    const capability = bindCapability(
      {
        __wabouCapabilityVersion: 2,
        async inspect(request: { path: string }) {
          return { path: request.path, changedFiles: 3 };
        },
      },
      { name: "workspace", version: 2 },
    );

    await expect(capability.inspect({ path: "/repo" })).resolves.toEqual({
      path: "/repo",
      changedFiles: 3,
    });
  });

  test("rejects an unavailable ABI before the first method call", () => {
    expect(() =>
      bindCapability(undefined, { name: "workspace", version: 2 }),
    ).toThrow(CapabilityError);
  });
});
