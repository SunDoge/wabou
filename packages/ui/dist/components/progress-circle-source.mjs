import { finiteOr } from "./range.mjs";
//#region src/components/progress-circle-source.ts
const circlePoint = (angle) => ({
	x: 12 + 9 * Math.cos(angle),
	y: 12 + 9 * Math.sin(angle)
});
/** Build a theme-colored SVG arc without relying on browser SVG layout. */
function progressCircleSource(percent) {
	const normalized = Math.max(0, Math.min(100, finiteOr(percent, 0)));
	const track = "<circle cx=\"12\" cy=\"12\" r=\"9\" stroke=\"currentColor\" stroke-opacity=\"0.2\" stroke-width=\"3\"/>";
	if (normalized <= 0) return `<svg viewBox="0 0 24 24" fill="none">${track}</svg>`;
	if (normalized >= 100) return `<svg viewBox="0 0 24 24" fill="none">${track}<circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="3"/></svg>`;
	const start = circlePoint(-Math.PI / 2);
	const end = circlePoint(-Math.PI / 2 + normalized / 100 * Math.PI * 2);
	const largeArc = normalized > 50 ? 1 : 0;
	const value = (coordinate) => coordinate.toFixed(4);
	return `<svg viewBox="0 0 24 24" fill="none">${track}<path d="${`M ${value(start.x)} ${value(start.y)} A 9 9 0 ${largeArc} 1 ${value(end.x)} ${value(end.y)}`}" stroke="currentColor" stroke-width="3" stroke-linecap="round"/></svg>`;
}
//#endregion
export { progressCircleSource };

//# sourceMappingURL=progress-circle-source.mjs.map