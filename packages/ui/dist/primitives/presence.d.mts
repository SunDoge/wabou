import { Accessor } from "solid-js";
//#region src/primitives/presence.d.ts
export type PresencePhase = "unmounted" | "entering" | "present" | "exiting";
export interface Presence {
  phase: Accessor<PresencePhase>;
  mounted: Accessor<boolean>;
  finishEnter(): void;
  finishExit(): void;
}
/** Explicit mount lifecycle for content whose exit must finish before removal. */
export declare function createPresence(open: Accessor<boolean>): Presence;
//#endregion
//# sourceMappingURL=presence.d.mts.map