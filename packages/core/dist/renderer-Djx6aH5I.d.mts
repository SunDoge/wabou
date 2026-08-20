import "./index-DZzljb8I.mjs";
//#region src/renderer/use-fps.d.ts
/**
 * Track frames-per-second. A self-perpetuating rAF loop counts frames; a
 * 1s interval samples the count and resets it. The rAF loop keeps the host
 * redrawing (it drives `has_anim`), so this measures the active vsync rate
 * while mounted — ~60 on a 60Hz display, ~120 on 120Hz. When nothing animates,
 * the host stops redrawing and the count drops.
 *
 * ```tsx
 * const fps = createFps();
 * <Text>{`${fps()} fps`}</Text>
 * ```
 */
declare function createFps(): () => number;
//#endregion
export { createFps as t };
//# sourceMappingURL=renderer-Djx6aH5I.d.mts.map