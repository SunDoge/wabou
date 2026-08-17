import { createMemo, createRoot, createSignal, flush } from "solid-js";
import "../../packages/core/src/glue/timers";

const [location, setLocation] = createSignal("/projects/alpha");
const pathname = createRoot(() => createMemo(location));
flush(() => setLocation("/settings"));

globalThis.__wabouRouterBaseline = pathname();

declare global {
  var __wabouRouterBaseline: string | undefined;
}
