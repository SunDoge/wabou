//#region src/primitives/animation-frame.d.ts
export type AnimationFrameCallback = (timestamp: number) => unknown;
/**
 * Drive explicit paint state from the native host's animation clock.
 * Return `false` to stop scheduling frames before the owner is disposed.
 */
export declare function createAnimationFrame(callback: AnimationFrameCallback): () => void;
//#endregion
//# sourceMappingURL=animation-frame.d.mts.map