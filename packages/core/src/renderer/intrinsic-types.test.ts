import { expect, test } from "bun:test";
import type { WabouBuiltinIntrinsicElements } from "./index";

type PublicIntrinsic = keyof WabouBuiltinIntrinsicElements;

const structuralTag: PublicIntrinsic = "view";
const vectorTag: PublicIntrinsic = "vector-path";

// These host tags remain renderer implementation details. Public application
// code must use the typed Button, TextInput, Image, and Svg components.
// @ts-expect-error button is not a public JSX intrinsic.
const webButton: PublicIntrinsic = "button";
// @ts-expect-error input is not a public JSX intrinsic.
const webInput: PublicIntrinsic = "input";
// @ts-expect-error img is not a public JSX intrinsic.
const webImage: PublicIntrinsic = "img";
// @ts-expect-error svg is not a public JSX intrinsic.
const webSvg: PublicIntrinsic = "svg";

test("only Wabou structural primitives are public intrinsic tags", () => {
  expect([structuralTag, vectorTag]).toEqual(["view", "vector-path"]);
  void webButton;
  void webInput;
  void webImage;
  void webSvg;
});
