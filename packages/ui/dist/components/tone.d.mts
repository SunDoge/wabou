//#region src/components/tone.d.ts
/** Semantic color shared by compact component surfaces and indicators. */
export type ComponentTone = "neutral" | "accent" | "success" | "danger";
export type ComponentTonePresentation = "text" | "surface" | "solid" | "indicator";
/**
 * Maps semantic intent to the built-in Wabou theme contract.
 *
 * Keeping this recipe in one place prevents badges, icon tiles and status
 * indicators from drifting into slightly different colors for the same state.
 */
export declare function componentToneClass(tone: ComponentTone, presentation: ComponentTonePresentation): string;
//#endregion
//# sourceMappingURL=tone.d.mts.map