import { decimalPlaces, finiteOr } from "./range.mjs";
//#region src/components/number-field-state.ts
function normalizeNumberFieldRange(minValue, maxValue, stepValue, largeStepValue) {
	const min = finiteOr(minValue, Number.NEGATIVE_INFINITY);
	const max = Math.max(min, finiteOr(maxValue, Number.POSITIVE_INFINITY));
	const candidateStep = finiteOr(stepValue, 1);
	const step = candidateStep > 0 ? candidateStep : 1;
	const candidateLargeStep = finiteOr(largeStepValue, step * 10);
	return {
		min,
		max,
		step,
		largeStep: candidateLargeStep > 0 ? candidateLargeStep : step * 10
	};
}
function clampNumberFieldValue(value, range) {
	return Math.max(range.min, Math.min(range.max, value));
}
function addNumberFieldStep(value, amount, range) {
	const precision = Math.max(decimalPlaces(value), decimalPlaces(amount));
	return clampNumberFieldValue(Number((value + amount).toFixed(precision)), range);
}
function numberFieldValueFromEmpty(direction, range) {
	if (direction > 0 && Number.isFinite(range.min)) return range.min;
	if (direction < 0 && Number.isFinite(range.max)) return range.max;
	return clampNumberFieldValue(0, range);
}
//#endregion
export { addNumberFieldStep, clampNumberFieldValue, normalizeNumberFieldRange, numberFieldValueFromEmpty };

//# sourceMappingURL=number-field-state.mjs.map