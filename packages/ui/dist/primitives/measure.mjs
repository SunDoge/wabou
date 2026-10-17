import { createSignal, onCleanup } from "solid-js";
//#region src/primitives/measure.ts
function validateSizeQuery(query) {
	const entries = [
		["minWidth", query.minWidth],
		["maxWidth", query.maxWidth],
		["minHeight", query.minHeight],
		["maxHeight", query.maxHeight]
	];
	for (const [name, value] of entries) if (value !== void 0 && (!Number.isFinite(value) || value < 0)) throw new RangeError(`${name} must be a finite non-negative number`);
	if (query.minWidth !== void 0 && query.maxWidth !== void 0 && query.minWidth > query.maxWidth) throw new RangeError("minWidth cannot exceed maxWidth");
	if (query.minHeight !== void 0 && query.maxHeight !== void 0 && query.minHeight > query.maxHeight) throw new RangeError("minHeight cannot exceed maxHeight");
}
/** Observe the completed native content-box size of a host node. */
function createMeasuredSize(options = {}) {
	const [width, setWidth] = createSignal(0);
	const [height, setHeight] = createSignal(0);
	const [measured, setMeasured] = createSignal(false);
	let observer;
	const ref = (node) => {
		observer?.disconnect();
		observer = new ResizeObserver(([entry]) => {
			if (!entry) return;
			const size = {
				width: entry.contentRect.width,
				height: entry.contentRect.height
			};
			setWidth(size.width);
			setHeight(size.height);
			setMeasured(true);
			options.onChange?.(size);
		});
		observer.observe(node);
	};
	onCleanup(() => observer?.disconnect());
	return {
		ref,
		width,
		height,
		measured
	};
}
/**
* Match constraints against a component's completed native content-box size.
* The result remains false until the first measurement, avoiding a compact
* layout flash during boot.
*/
function createContainerMatch(query, options = {}) {
	validateSizeQuery(query);
	const size = createMeasuredSize(options);
	const matches = () => size.measured() && (query.minWidth === void 0 || size.width() >= query.minWidth) && (query.maxWidth === void 0 || size.width() <= query.maxWidth) && (query.minHeight === void 0 || size.height() >= query.minHeight) && (query.maxHeight === void 0 || size.height() <= query.maxHeight);
	return {
		...size,
		matches
	};
}
//#endregion
export { createContainerMatch, createMeasuredSize };

//# sourceMappingURL=measure.mjs.map