import { createComponent, createContext, createEffect, createSignal, onCleanup, untrack, useContext } from "solid-js";
//#region src/primitives/overlay-layer.ts
const openLayers = [];
let nextOrder = 1;
const OverlayPlaneContext = createContext("floating");
/** Make nested portals inherit the current native stacking plane. */
function OverlayPlaneProvider(props) {
	return createComponent(OverlayPlaneContext, {
		get value() {
			return props.plane;
		},
		get children() {
			return props.children;
		}
	});
}
function useOverlayPlane() {
	return useContext(OverlayPlaneContext);
}
function planeRank(plane) {
	return plane === "modal" ? 1 : 0;
}
function removeLayer(token) {
	const index = openLayers.findIndex((layer) => layer.token === token);
	if (index >= 0) openLayers.splice(index, 1);
}
function pushLayer(token, plane) {
	removeLayer(token);
	const order = nextOrder++;
	openLayers.push({
		token,
		plane,
		order
	});
	return order;
}
function topmostLayer() {
	return openLayers.reduce((topmost, candidate) => {
		if (!topmost) return candidate;
		const rank = planeRank(candidate.plane) - planeRank(topmost.plane);
		return rank > 0 || rank === 0 && candidate.order > topmost.order ? candidate : topmost;
	}, void 0);
}
function createOverlayLayer(options) {
	const token = Symbol("wabou-overlay-layer");
	const plane = () => options.plane?.() ?? "floating";
	let wasOpen = untrack(options.open);
	let activePlane = untrack(plane);
	const [zIndex, setZIndex] = createSignal(wasOpen ? pushLayer(token, activePlane) : 0);
	const restoreFocus = (enabled = untrack(() => options.restoreFocus?.() ?? true), target = untrack(() => options.returnFocus?.())) => {
		if (enabled) untrack(() => target?.focus());
	};
	createEffect(() => ({
		open: options.open(),
		plane: plane(),
		restoreFocus: options.restoreFocus?.() ?? true,
		returnFocus: options.returnFocus?.()
	}), (snapshot) => {
		if (snapshot.open && !wasOpen) setZIndex(pushLayer(token, snapshot.plane));
		else if (!snapshot.open && wasOpen) {
			removeLayer(token);
			restoreFocus(snapshot.restoreFocus, snapshot.returnFocus);
		} else if (snapshot.open && activePlane !== snapshot.plane) setZIndex(pushLayer(token, snapshot.plane));
		wasOpen = snapshot.open;
		activePlane = snapshot.plane;
	});
	onCleanup(() => {
		removeLayer(token);
		if (wasOpen) restoreFocus();
		wasOpen = false;
	});
	const isTopmost = () => topmostLayer()?.token === token;
	return {
		plane,
		zIndex,
		isTopmost,
		onEscape(event) {
			if (!options.open() || !isTopmost() || event.key !== "Escape" || options.closeOnEscape?.() === false) return;
			event.preventDefault();
			event.stopPropagation();
			options.onDismiss("escape");
		},
		onOutside(event) {
			if (!options.open() || !isTopmost() || options.closeOnOutside?.() === false) return;
			event.preventDefault();
			event.stopPropagation();
			options.onDismiss("outside");
		}
	};
}
//#endregion
export { OverlayPlaneProvider, createOverlayLayer, useOverlayPlane };

//# sourceMappingURL=overlay-layer.mjs.map