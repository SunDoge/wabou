import { expect, test } from "bun:test";
import { rgba } from "@wabou/style";
import { createElement, spread } from "./index";

test("renderer rejects invalid inline styles before they cross the bridge", () => {
  const node = createElement("view");
  expect(() =>
    spread(node, { style: { alignItems: "center" } }, false),
  ).toThrow("use align-items");
  expect(() =>
    spread(node, { style: { width: rgba(0xff0000ff) } }, false),
  ).toThrow("invalid for width");
});
