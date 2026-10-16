//#region src/components/delayed-open.d.ts
export interface DelayedOpenController {
  scheduleOpen(): void;
  scheduleClose(): void;
  openNow(): void;
  closeNow(): void;
  cancel(): void;
  dispose(): void;
}
export interface DelayedOpenOptions {
  openDelay: () => number;
  closeDelay: () => number;
  setOpen(open: boolean): void;
}
/** Owns cancellable open/close timers independently from a rendered surface. */
export declare function createDelayedOpenController(options: DelayedOpenOptions): DelayedOpenController;
//#endregion
//# sourceMappingURL=delayed-open.d.mts.map