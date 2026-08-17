import { Button } from "@wabou/components";
import { createSignal } from "solid-js";
import { mount } from "@wabou/core";
import { defineWabouConfig } from "@wabou/vite";

const [enabled, setEnabled] = createSignal(false);

mount(() => (
  <Button onClick={() => setEnabled((value) => !value)}>
    {enabled() ? "Enabled" : "Disabled"}
  </Button>
));

export default defineWabouConfig({});
