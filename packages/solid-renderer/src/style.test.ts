import { expect, test } from "bun:test";
import { rgba } from "@wabou/style";
import { createElement, setProp } from "./index";

test("renderer rejects invalid inline styles before they cross the bridge", () => {
  const node = createElement("view");
  expect(() =>
    setProp(node, "style", { alignItems: "center" }, undefined),
  ).toThrow("use align-items");
  expect(() =>
    setProp(node, "style", { width: rgba(0xff0000ff) }, undefined),
  ).toThrow("invalid for width");
});
