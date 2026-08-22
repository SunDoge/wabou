import { describe, expect, test } from "bun:test";
import { mount, writer } from "@wabou/core/renderer";
import { createComponent, flush } from "solid-js";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
  inputGroupAddonClass,
  inputGroupClass,
} from "../../dist/index.mjs";

describe("InputGroup", () => {
  test("owns one explicit surface and focus state", () => {
    expect(inputGroupClass("horizontal", false, false)).toContain(
      "h-8 flex-row",
    );
    expect(inputGroupClass("vertical", true, false)).toContain(
      "h-auto flex-col",
    );
    expect(inputGroupClass("vertical", true, false)).toContain("border-focus");
    expect(inputGroupClass("horizontal", true, true)).toContain(
      "border-danger",
    );
  });

  test("maps inline and block addons without selector inference", () => {
    expect(inputGroupAddonClass("inline-start")).toContain("h-full");
    expect(inputGroupAddonClass("block-end")).toContain("w-full");
  });

  test("lets the compound control own the native input height", () => {
    const classes: string[] = [];
    const setClassName = writer.setClassName.bind(writer);
    writer.setClassName = (_id, value) => classes.push(value);

    let disposeMount: (() => void) | undefined;
    try {
      disposeMount = mount(() =>
        createComponent(InputGroup, {
          get children() {
            return [
              createComponent(InputGroupAddon, {
                get children() {
                  return createComponent(InputGroupText, {
                    children: "https://",
                  });
                },
              }),
              createComponent(InputGroupInput, { placeholder: "example.com" }),
            ];
          },
        }),
      );
      flush();
    } finally {
      disposeMount?.();
      writer.setClassName = setClassName;
    }

    expect(classes.some((value) => value.includes("h-full flex-1"))).toBe(true);
  });
});
