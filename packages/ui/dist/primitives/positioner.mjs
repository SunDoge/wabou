//#region src/primitives/positioner.ts
/** Build the explicit native positioning contract for a retained trigger. */
function floatingFromNode(anchor, options = {}) {
	return {
		anchor: {
			kind: "node",
			id: anchor.id
		},
		placement: options.placement,
		offset: options.offset,
		margin: options.margin
	};
}
/** Build the explicit native positioning contract for a viewport point. */
function floatingFromPoint(point, options = {}) {
	if (![point.x, point.y].every(Number.isFinite)) throw new RangeError("floating point anchor must be finite");
	return {
		anchor: {
			kind: "point",
			x: point.x,
			y: point.y
		},
		placement: options.placement,
		offset: options.offset,
		margin: options.margin
	};
}
//#endregion
export { floatingFromNode, floatingFromPoint };

//# sourceMappingURL=positioner.mjs.map