import { Button } from "@wabou/components";
import { mount } from "@wabou/core";
import {
  BaseRootRoute,
  createDataRouter,
  createMemoryHistory,
} from "@wabou/router";
import { defineWabouConfig } from "@wabou/vite";
import { createSignal } from "solid-js";

const [enabled, setEnabled] = createSignal(false);
const rootRoute = new BaseRootRoute();

// The public JSX registry is intentionally native and finite, rather than the
// complete browser element catalogue.
const structuralHostNode = <section class="flex" />;
const plainTextInput = <input type="text" />;
// @ts-expect-error links are components/capabilities, not implicit Web anchors
const unsupportedAnchor = <a href="https://example.com">Unsupported</a>;
// @ts-expect-error secrets require the dedicated PasswordInput + SecretStore
const insecureGenericInput = <input type="password" />;
// @ts-expect-error browser tooltip attributes have no native Wabou behavior
const unsupportedTitle = <div title="Browser tooltip" />;
// @ts-expect-error structural host nodes do not acquire browser navigation
const unsupportedHref = <div href="https://example.com" />;
// @ts-expect-error inline styles expose only properties implemented by Style IR
const unsupportedStyle = <div style={{ filter: "blur(4px)" }} />;
void [
  structuralHostNode,
  plainTextInput,
  unsupportedAnchor,
  insecureGenericInput,
  unsupportedTitle,
  unsupportedHref,
  unsupportedStyle,
];

createDataRouter({
  routeTree: rootRoute,
  history: createMemoryHistory(),
  context: {},
});

mount(() => (
  <Button onClick={() => setEnabled((value) => !value)}>
    {enabled() ? "Enabled" : "Disabled"}
  </Button>
));

export default defineWabouConfig({});
