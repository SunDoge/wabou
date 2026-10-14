import "./host.mjs";
//#region src/performance.d.ts
/** Flush completed user measurements in one host call. */
export declare function flushPerformance(): void;
/** Measure synchronous application work without changing its return/error behavior. */
export declare function measurePerformance<T>(name: string, operation: () => T, correlation?: number): T;
/** Record async elapsed time separately from CPU execution tracks. */
export declare function measureAsyncPerformance<T>(name: string, operation: () => Promise<T>, correlation?: number): Promise<T>;
//#endregion
//# sourceMappingURL=performance.d.mts.map