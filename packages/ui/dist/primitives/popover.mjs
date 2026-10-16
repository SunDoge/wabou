import { useReducedMotion } from "../animation/config.mjs";
import "../animation.mjs";
import { View } from "./view.mjs";
import { createOverlayLayer, useOverlayPlane } from "./overlay-layer.mjs";
import { createTransitionPresence } from "./transition-presence.mjs";
import { floatingFromNode, floatingFromPoint } from "./positioner.mjs";
import { scale2d } from "@wabou/core/style";
import { Show, createSignal, onCleanup, untrack } from "solid-js";
import { Portal, createComponent as createComponent$1, memo, observeGlobalPointerEvent } from "@wabou/core/renderer";
//#region src/primitives/popover.tsx
/** A root-layer floating panel positioned from native layout snapshots. */
function Popover(props) {
	const inheritedPlane = useOverlayPlane();
	const reducedMotion = useReducedMotion();
	const plane = () => props.plane ?? inheritedPlane;
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(untrack(() => props.defaultOpen ?? false));
	const open = () => props.open ?? uncontrolledOpen();
	const motion = untrack(() => props.motion);
	const motionEnabled = motion !== void 0 && motion !== false;
	const presence = createTransitionPresence(open, {
		initialProgress: untrack(open) ? 1 : 0,
		duration: motionEnabled ? motion.duration ?? .14 : 0,
		ease: motionEnabled ? motion.ease ?? "easeOut" : "linear",
		reducedMotion
	});
	let anchor;
	let content;
	let suppressPointerClick = false;
	const motionFromScale = () => motionEnabled ? motion.fromScale ?? .98 : 1;
	const contains = (root, target) => {
		if (!root || !target) return false;
		let current = target;
		while (current) {
			if (current === root) return true;
			current = current.parent;
		}
		return false;
	};
	const setOpen = (next, reason) => {
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next, reason);
	};
	const layer = createOverlayLayer({
		open,
		plane,
		onDismiss: (reason) => setOpen(false, reason),
		closeOnEscape: () => props.closeOnEscape ?? true,
		returnFocus: () => anchor,
		restoreFocus: () => props.restoreFocus ?? true
	});
	const stopObservingPointer = observeGlobalPointerEvent("click", (target) => {
		if (props.outsidePointerStrategy !== "passthrough" || !open() || contains(anchor, target) || contains(content, target)) return;
		layer.onOutside({
			preventDefault() {},
			stopPropagation() {}
		});
	});
	const floatingPosition = () => {
		const options = {
			placement: props.placement ?? "bottom-start",
			offset: props.offset ?? 6,
			margin: 8
		};
		const point = props.anchorPoint?.();
		if (point) return floatingFromPoint(point, options);
		return anchor ? floatingFromNode(anchor, options) : void 0;
	};
	onCleanup(() => {
		stopObservingPointer();
	});
	const handleEscape = (event) => layer.onEscape(event);
	const popup = () => {
		if (props.contentRole !== "presentation") return "dialog";
		return props.popupRole === "tooltip" ? void 0 : props.popupRole;
	};
	const triggerProps = {
		ref: (node) => {
			anchor = node;
		},
		onPointerDown: (event) => {
			if (!props.openOnPointerDown || open() || event.button !== void 0 && event.button !== 0) return;
			event.stopPropagation();
			suppressPointerClick = true;
			setOpen(true, "trigger");
		},
		onPointerCancel: () => {
			suppressPointerClick = false;
		},
		onClick: (event) => {
			event.stopPropagation();
			if (suppressPointerClick) {
				suppressPointerClick = false;
				return;
			}
			setOpen(!open(), "trigger");
		},
		onKeyDown: handleEscape,
		get "aria-haspopup"() {
			return popup();
		},
		get "aria-expanded"() {
			return open();
		}
	};
	return [untrack(() => props.trigger(triggerProps)), createComponent$1(Show, {
		get when() {
			return presence.mounted();
		},
		get children() {
			return createComponent$1(Portal, {
				get plane() {
					return plane();
				},
				role: "presentation",
				get style() {
					return {
						position: "absolute",
						left: 0,
						top: 0,
						width: "100%",
						height: "100%",
						"z-index": layer.zIndex(),
						"pointer-events": !open() || props.outsidePointerStrategy === "passthrough" ? "none" : "auto"
					};
				},
				get onClick() {
					return memo(() => {
						return props.outsidePointerStrategy === "passthrough";
					})() ? void 0 : layer.onOutside;
				},
				onKeyDown: handleEscape,
				get children() {
					return createComponent$1(View, {
						ref: (node) => {
							content = node;
						},
						get role() {
							return props.contentRole ?? "dialog";
						},
						get "aria-label"() {
							return props["aria-label"];
						},
						get "class"() {
							return props.contentClass;
						},
						get shadows() {
							return props.contentShadows;
						},
						get transform() {
							return scale2d(motionFromScale() + (1 - motionFromScale()) * presence.progress());
						},
						get floatingPosition() {
							return floatingPosition();
						},
						get interactionBlocked() {
							return !open() || props.contentInteractionBlocked;
						},
						get "aria-hidden"() {
							return open() ? void 0 : "true";
						},
						get style() {
							return {
								position: "absolute",
								...props.contentStyle,
								opacity: presence.progress()
							};
						},
						onClick: (event) => event.stopPropagation(),
						get onPointerEnter() {
							return props.onContentPointerEnter;
						},
						get onPointerLeave() {
							return props.onContentPointerLeave;
						},
						get onFocusIn() {
							return props.onContentFocusIn;
						},
						get onFocusOut() {
							return props.onContentFocusOut;
						},
						onKeyDown: handleEscape,
						get children() {
							return props.children;
						}
					});
				}
			});
		}
	})];
}
//#endregion
export { Popover };

//# sourceMappingURL=popover.mjs.map