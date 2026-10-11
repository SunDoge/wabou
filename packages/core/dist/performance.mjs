//#region src/performance.ts
const pending = [];
let scheduled = false;
function enabled() {
	return typeof __wabou_profile_enabled === "function" && __wabou_profile_enabled();
}
/** Flush completed user measurements in one host call. */
function flushPerformance() {
	scheduled = false;
	if (!pending.length) return;
	const now = performance.now();
	const batch = pending.splice(0, 128).map(({ end, ...event }) => ({
		...event,
		ageUs: Math.max(0, Math.round((now - end) * 1e3))
	}));
	__wabou_profile_batch(JSON.stringify(batch));
}
function record(name, start, correlation, asynchronous) {
	const end = performance.now();
	pending.push({
		name: name.slice(0, 128).replace(/[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/g, "�"),
		durationUs: Math.max(0, Math.round((end - start) * 1e3)),
		end,
		correlation: Number.isSafeInteger(correlation) && correlation >= 0 ? correlation : 0,
		asynchronous
	});
	if (pending.length === 128) flushPerformance();
	else if (!scheduled) {
		scheduled = true;
		Promise.resolve().then(flushPerformance);
	}
}
/** Measure synchronous application work without changing its return/error behavior. */
function measurePerformance(name, operation, correlation = 0) {
	if (!enabled()) return operation();
	const start = performance.now();
	try {
		return operation();
	} finally {
		record(name, start, correlation, false);
	}
}
/** Record async elapsed time separately from CPU execution tracks. */
async function measureAsyncPerformance(name, operation, correlation = 0) {
	if (!enabled()) return operation();
	const start = performance.now();
	try {
		return await operation();
	} finally {
		record(name, start, correlation, true);
	}
}
//#endregion
export { flushPerformance, measureAsyncPerformance, measurePerformance };

//# sourceMappingURL=performance.mjs.map