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
export function createDelayedOpenController(
  options: DelayedOpenOptions,
): DelayedOpenController {
  let timer: ReturnType<typeof setTimeout> | undefined;

  const cancel = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
  };
  const schedule = (open: boolean, delay: number) => {
    cancel();
    if (delay <= 0) {
      options.setOpen(open);
      return;
    }
    timer = setTimeout(() => {
      timer = undefined;
      options.setOpen(open);
    }, delay);
  };

  return {
    scheduleOpen: () => schedule(true, options.openDelay()),
    scheduleClose: () => schedule(false, options.closeDelay()),
    openNow: () => {
      cancel();
      options.setOpen(true);
    },
    closeNow: () => {
      cancel();
      options.setOpen(false);
    },
    cancel,
    dispose: cancel,
  };
}
