import manifest from "./preset/manifest.json" with { type: "json" };
import type { WabouUtilityManifest } from "./preset";

/** Build-time manifest used by tooling that presents Wabou's utility surface. */
export default manifest as WabouUtilityManifest;
