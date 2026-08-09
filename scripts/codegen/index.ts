import { generateRuntimeBundles } from "./bundles.ts";

// Keep dependency order explicit: style conformance consumes the utility
// manifest, while the remaining domains are independent.
await import("../gen-host-abi.ts");
await import("../gen-utility-manifest.ts");
await import("../gen-style-properties.ts");
await import("../gen-style-conformance.ts");
await import("../gen-rust-op.ts");
await generateRuntimeBundles();
