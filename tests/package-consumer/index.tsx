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
