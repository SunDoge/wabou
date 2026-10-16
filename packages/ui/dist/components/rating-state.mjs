//#region src/components/rating-state.ts
function normalizeRatingMax(max) {
	if (max === void 0 || !Number.isFinite(max)) return 5;
	return Math.max(1, Math.min(20, Math.floor(max)));
}
function clampRatingValue(value, max) {
	if (value === void 0 || !Number.isFinite(value)) return 0;
	return Math.max(0, Math.min(normalizeRatingMax(max), Math.round(value)));
}
function ratingLabel(value) {
	return `${value} ${value === 1 ? "star" : "stars"}`;
}
//#endregion
export { clampRatingValue, normalizeRatingMax, ratingLabel };

//# sourceMappingURL=rating-state.mjs.map