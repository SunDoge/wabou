import { expect, test } from "bun:test";
import { mount, writer } from "@wabou/core/renderer";
import { createComponent, flush } from "solid-js";
// Exercise the Solid-transformed public artifact.
import { Switch } from "../../dist/index.mjs";

test("switch labels can shrink and wrap inside constrained forms", () => {
  const classes: string[] = [];
  const setClassName = writer.setClassName.bind(writer);
  writer.setClassName = (_id, value) => classes.push(value);

  let disposeMount: (() => void) | undefined;
  try {
    disposeMount = mount(() =>
      createComponent(Switch, {
        label: "Warn before quitting while downloads are running",
      }),
    );
    flush();
  } finally {
    disposeMount?.();
    writer.setClassName = setClassName;
  }

  expect(
    classes.some((value) =>
      value.includes("w-full min-w-0 flex items-center gap-3"),
    ),
  ).toBe(true);
  expect(
    classes.some((value) =>
      value.includes("min-w-0 flex-1 whitespace-normal text-sm"),
    ),
  ).toBe(true);
});
