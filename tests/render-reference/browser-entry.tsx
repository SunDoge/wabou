import { render } from "@solidjs/web";
import "virtual:uno.css";
import { defaultWabouColorThemes } from "../../packages/vite/src/theme-contract";
import { CrossRenderFixture } from "./cross-render-fixture";
import "./browser.css";

declare const __WABOU_COMPARE_WIDTH__: number;
declare const __WABOU_COMPARE_HEIGHT__: number;

const theme = defaultWabouColorThemes.themes.light.colors;
for (const [name, value] of Object.entries(theme)) {
  document.documentElement.style.setProperty(`--wabou-${name}`, value);
}

const root = document.getElementById("root");
if (!root) throw new Error("cross-render browser root is missing");
for (const element of [document.documentElement, document.body, root]) {
  element.style.width = `${__WABOU_COMPARE_WIDTH__}px`;
  element.style.height = `${__WABOU_COMPARE_HEIGHT__}px`;
}
render(() => <CrossRenderFixture />, root);

const snapshot = [
  ...document.querySelectorAll<HTMLElement>("[aria-label]"),
].map((element) => {
  const rect = element.getBoundingClientRect();
  const style = getComputedStyle(element);
  return {
    name: element.getAttribute("aria-label"),
    rect: {
      x: rect.x,
      y: rect.y,
      width: rect.width,
      height: rect.height,
    },
    style: {
      background: style.backgroundColor,
      color: style.color,
    },
  };
});
document.documentElement.dataset.wabouSnapshot = btoa(JSON.stringify(snapshot));
