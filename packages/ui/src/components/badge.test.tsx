import { expect, test } from "bun:test";
import { mount, writer } from "@wabou/core/renderer";
import { createComponent, flush } from "solid-js";
// Exercise the Solid-transformed public artifact.
import { Fps } from "../../dist/index.mjs";

test("monospace FPS badges select one explicit font weight", () => {
  const classes: string[] = [];
  const setClassName = writer.setClassName.bind(writer);
  writer.setClassName = (_id, value) => classes.push(value);

  let disposeMount: (() => void) | undefined;
  try {
    disposeMount = mount(() => createComponent(Fps, { value: 0 }));
    flush();
  } finally {
    disposeMount?.();
    writer.setClassName = setClassName;
  }

  const fps = classes.find((value) => value.includes("font-mono"));
  expect(fps).toContain("font-normal");
  expect(fps).not.toContain("font-medium");
});
