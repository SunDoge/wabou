//#region src/components/pagination-state.ts
function integerAtLeast(value, minimum) {
	return Math.max(minimum, Math.floor(Number.isFinite(value) ? value : minimum));
}
function normalizePageCount(count) {
	return integerAtLeast(count, 1);
}
function clampPage(page, count) {
	return Math.min(normalizePageCount(count), integerAtLeast(page, 1));
}
function range(start, end) {
	return Array.from({ length: Math.max(0, end - start + 1) }, (_, index) => start + index);
}
/**
* Produces a stable, 1-indexed page range with explicit start/end ellipses.
* A single hidden page is shown directly instead of being replaced by an
* ellipsis, which keeps every item actionable and avoids misleading gaps.
*/
function createPaginationRange(options) {
	const count = normalizePageCount(options.count);
	const page = clampPage(options.page, count);
	const siblings = integerAtLeast(options.siblingCount ?? 1, 0);
	const boundaries = integerAtLeast(options.boundaryCount ?? 1, 0);
	if (count <= boundaries * 2 + siblings * 2 + 3) return range(1, count);
	const left = Math.max(page - siblings, boundaries + 2);
	const right = Math.min(page + siblings, count - boundaries - 1);
	const start = range(1, boundaries);
	const middle = range(left, right);
	const end = range(count - boundaries + 1, count);
	const before = left === boundaries + 2 ? [boundaries + 1] : ["ellipsis-start"];
	const after = right === count - boundaries - 1 ? [count - boundaries] : ["ellipsis-end"];
	return [
		...start,
		...before,
		...middle,
		...after,
		...end
	];
}
//#endregion
export { clampPage, createPaginationRange, normalizePageCount };

//# sourceMappingURL=pagination-state.mjs.map