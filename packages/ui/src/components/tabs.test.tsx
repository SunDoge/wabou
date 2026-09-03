import { expect, test } from "bun:test";
import { mount, writer } from "@wabou/core/renderer";
import { createComponent, flush } from "solid-js";
// Exercise the Solid-transformed public artifact. Bun's direct TSX loader does
// not apply Wabou's Solid universal transform to component implementation files.
import { Tabs, TabsList, TabsTrigger } from "../../dist/index.mjs";

test("unstyled tabs retain behavior without injecting the default skin", () => {
  const classes: string[] = [];
  const attributes: Array<[string, string]> = [];
  let triggerSelected = false;
  const setClassName = writer.setClassName.bind(writer);
  const setAttribute = writer.setAttribute.bind(writer);
  writer.setClassName = (_id, value) => classes.push(value);
  writer.setAttribute = (_id, name, value) => {
    if (name === "role" || name.startsWith("aria-"))
      attributes.push([name, value]);
  };

  let disposeMount: (() => void) | undefined;
  try {
    disposeMount = mount(() =>
      createComponent(Tabs, {
        defaultValue: "general",
        get children() {
          return createComponent(TabsList, {
            unstyled: true,
            class: "custom-list",
            "aria-label": "Sections",
            get children() {
              return createComponent(TabsTrigger, {
                unstyled: true,
                value: "general",
                "aria-label": "General",
                class: (state) => {
                  triggerSelected = state.selected;
                  return "custom-trigger";
                },
                children: "General",
              });
            },
          });
        },
      }),
    );
    flush();
  } finally {
    disposeMount?.();
    writer.setClassName = setClassName;
    writer.setAttribute = setAttribute;
  }

  expect(classes).toContain("custom-list");
  expect(
    classes.some((value) => value.split(/\s+/).includes("custom-trigger")),
  ).toBe(true);
  expect(classes.some((value) => value.includes("custom-trigger h-7"))).toBe(
    false,
  );
  expect(triggerSelected).toBe(true);
  expect(attributes).toContainEqual(["role", "tablist"]);
  expect(attributes).toContainEqual(["aria-label", "Sections"]);
  expect(attributes).toContainEqual(["role", "tab"]);
  expect(attributes).toContainEqual(["aria-selected", "true"]);
});
