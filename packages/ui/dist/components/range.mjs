//#region src/components/range.ts
function finiteOr(value, fallback) {
	return value !== void 0 && Number.isFinite(value) ? value : fallback;
}
function normalizeRange(minValue, maxValue, stepValue) {
	const min = finiteOr(minValue, 0);
	const max = Math.max(min, finiteOr(maxValue, 100));
	const candidateStep = finiteOr(stepValue, 1);
	return {
		min,
		max,
		step: candidateStep > 0 ? candidateStep : 1
	};
}
function decimalPlaces(value) {
	const [coefficient, exponentText] = String(value).toLowerCase().split("e");
	const fractionLength = coefficient.split(".")[1]?.length ?? 0;
	const exponent = exponentText === void 0 ? 0 : Number(exponentText);
	return Math.max(0, Math.min(100, fractionLength - exponent));
}
//#endregion
export { decimalPlaces, finiteOr, normalizeRange };

//# sourceMappingURL=range.mjs.map