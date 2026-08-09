import { generateRuntimeBundles } from "./bundles.ts";

await import("./protocol.ts");
await import("./style.ts");
await generateRuntimeBundles();
