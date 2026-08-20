export interface TooltipDelayController {
  scheduleOpen(): void;
  scheduleClose(): void;
  openNow(): void;
  closeNow(): void;
  dispose(): void;
}

export interface TooltipDelayOptions {
  openDelay: () => number;
  closeDelay: () => number;
  setOpen(open: boolean): void;
}

/** Owns tooltip timers independently from rendering and positioning. */
export function createTooltipDelayController(
  options: TooltipDelayOptions,
): TooltipDelayController {
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
    dispose: cancel,
  };
}
