import { expect, test } from "bun:test";
import { Button } from "@wabou/primitives";
import { writer } from "@wabou/solid-renderer";
import { createComponent, createRoot } from "solid-js";

test("published Button forwards native tab and accessibility state", () => {
  const attributes: Array<[string, string]> = [];
  const setAttribute = writer.setAttribute.bind(writer);
  writer.setAttribute = (_id, name, value) => {
    if (name === "tabIndex" || name === "aria-current") {
      attributes.push([name, value]);
    }
  };
  try {
    createRoot((dispose) => {
      createComponent(Button, {
        tabIndex: -1,
        "aria-current": "date",
      });
      dispose();
    });
  } finally {
    writer.setAttribute = setAttribute;
  }
  expect(attributes).toEqual([
    ["tabIndex", "-1"],
    ["aria-current", "date"],
  ]);
});
