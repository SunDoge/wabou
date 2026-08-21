import {
  createFormDraft,
  createStandardSchemaValidator,
  type StandardSchema,
} from "@wabou/ui";
import { createRoot, flush } from "solid-js";
import * as v from "valibot";
import { describe, expect, test } from "vitest";

interface Profile {
  name: string;
  port: number;
}

const profileSchema = v.object({
  name: v.pipe(v.string(), v.minLength(1, "Name is required.")),
  port: v.pipe(v.number(), v.minValue(1, "Port must be positive.")),
});

describe("Standard Schema integration", () => {
  test("maps standard issues into createFormDraft field errors", () => {
    let dispose = () => {};
    const draft = createRoot((rootDispose) => {
      dispose = rootDispose;
      return createFormDraft(
        { name: "Ready", port: 8080 },
        {
          validate: createStandardSchemaValidator(
            profileSchema as StandardSchema<Profile>,
          ),
        },
      );
    });

    expect(draft.valid()).toBe(true);
    draft.patch({ name: "", port: 0 });
    flush();
    expect(draft.valid()).toBe(false);
    expect(draft.fieldError("name")).toBe("Name is required.");
    expect(draft.fieldError("port")).toBe("Port must be positive.");
    dispose();
  });

  test("preserves root issues instead of attaching them to an arbitrary field", () => {
    const validator = createStandardSchemaValidator<Profile>({
      "~standard": {
        version: 1,
        vendor: "test",
        validate: () => ({ issues: [{ message: "Profile is unavailable." }] }),
      },
    });
    let dispose = () => {};
    const draft = createRoot((rootDispose) => {
      dispose = rootDispose;
      return createFormDraft(
        { name: "Ready", port: 8080 },
        { validate: validator },
      );
    });

    expect(draft.valid()).toBe(false);
    expect(draft.formError()).toBe("Profile is unavailable.");
    dispose();
  });

  test("rejects asynchronous schemas at the synchronous draft boundary", () => {
    const validator = createStandardSchemaValidator<Profile>({
      "~standard": {
        version: 1,
        vendor: "test",
        validate: async (value) => ({ value: value as Profile }),
      },
    });
    expect(() => validator({ name: "Ready", port: 8080 })).toThrow(
      "createFormDraft requires synchronous validation",
    );
  });
});
