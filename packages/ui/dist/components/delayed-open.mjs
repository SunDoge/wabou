//#region src/components/delayed-open.ts
/** Owns cancellable open/close timers independently from a rendered surface. */
function createDelayedOpenController(options) {
	let timer;
	const cancel = () => {
		if (timer !== void 0) clearTimeout(timer);
		timer = void 0;
	};
	const schedule = (open, delay) => {
		cancel();
		if (delay <= 0) {
			options.setOpen(open);
			return;
		}
		timer = setTimeout(() => {
			timer = void 0;
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
		dispose: cancel
	};
}
//#endregion
export { createDelayedOpenController };

//# sourceMappingURL=delayed-open.mjs.map