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
const unsupportedAnchor = <a href="https://example.com" />;
// @ts-expect-error secrets require the dedicated PasswordInput + SecretStore
const insecureGenericInput = <input type="password" />;
void [
  structuralHostNode,
  plainTextInput,
  unsupportedAnchor,
  insecureGenericInput,
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
