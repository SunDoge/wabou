import { expect, test } from "bun:test";
import { createRoot, flush } from "solid-js";
import { createFormDraft } from "./form-draft";

test("form drafts update fields and reset to an explicit baseline", () => {
  createRoot((dispose) => {
    const draft = createFormDraft({ name: "first", count: 1, selected: [1] });
    const [name, setName] = draft.control("name");
    expect(draft.dirty()).toBe(false);
    draft.set("count", (count) => count + 1);
    setName("second");
    draft.patch({ selected: [1, 2] });
    flush();
    expect(draft.value()).toEqual({
      name: "second",
      count: 2,
      selected: [1, 2],
    });
    expect(draft.dirty()).toBe(true);

    draft.reset();
    flush();
    expect(draft.field("name")).toBe("first");
    expect(name()).toBe("first");
    expect(draft.field("count")).toBe(1);
    expect(draft.dirty()).toBe(false);

    draft.resetTo({ name: "remote", count: 4, selected: [] });
    draft.set("name", "edited");
    flush();
    expect(draft.dirty()).toBe(true);
    draft.commit();
    flush();
    expect(draft.dirty()).toBe(false);
    draft.set("name", "remote");
    flush();
    expect(draft.dirty()).toBe(true);
    dispose();
  });
});

test("form drafts derive validation errors from every mutation path", () => {
  createRoot((dispose) => {
    const draft = createFormDraft(
      { name: "ready", count: 1 },
      {
        validate: (value) => ({
          ...(value.name.trim() ? {} : { name: "Name is required." }),
          ...(value.count > 0 ? {} : { count: "Count must be positive." }),
        }),
      },
    );

    expect(draft.valid()).toBe(true);
    expect(draft.errors()).toEqual({});
    draft.patch({ name: "", count: 0 });
    flush();
    expect(draft.valid()).toBe(false);
    expect(draft.fieldError("name")).toBe("Name is required.");
    expect(draft.fieldError("count")).toBe("Count must be positive.");

    draft.reset();
    flush();
    expect(draft.valid()).toBe(true);
    draft.resetTo({ name: "", count: 2 });
    flush();
    expect(draft.valid()).toBe(false);
    expect(draft.fieldError("name")).toBe("Name is required.");
    dispose();
  });
});
