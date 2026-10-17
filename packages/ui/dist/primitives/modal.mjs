import { useReducedMotion } from "../animation/config.mjs";
import "../animation.mjs";
import { View } from "./view.mjs";
import { OverlayPlaneProvider, createOverlayLayer } from "./overlay-layer.mjs";
import { createTransitionPresence } from "./transition-presence.mjs";
import { mergeClasses, rgba } from "@wabou/core/style";
import { Show, createComponent, createEffect, createSignal, onCleanup, untrack } from "solid-js";
import { Portal } from "@wabou/core/renderer";
//#region src/primitives/modal.tsx
/**
* Derive every modal-plane policy from the committed controlled state.
*
* Presence only controls how long the subtree remains mounted. It must never
* prolong focus containment, hit testing, blur, or an opaque scrim after the
* owner has committed `open=false`.
*/
function modalVisualState(open, backdropFade) {
	return {
		active: open,
		retainBackdropVisuals: open || backdropFade !== false,
		transparentBackdrop: !open && backdropFade === false
	};
}
function modalMotionTransform(options, progress) {
	const scale = (options?.fromScale ?? 1) + progress * (1 - (options?.fromScale ?? 1));
	const remaining = 1 - progress;
	const offset = (value) => {
		const result = value * remaining;
		return result === 0 ? 0 : result;
	};
	return [
		scale,
		0,
		0,
		scale,
		offset(options?.fromX ?? 0),
		offset(options?.fromY ?? 0)
	];
}
/**
* A native modal plane with host-enforced focus, hit-test, and accessibility
* isolation. Visual styling remains explicit so applications can own it.
*/
function Modal(props) {
	const reducedMotion = useReducedMotion();
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(untrack(() => props.defaultOpen ?? false));
	const open = () => props.open ?? uncontrolledOpen();
	const motion = untrack(() => props.motion);
	const motionOptions = motion === false ? void 0 : motion;
	const motionEnabled = motionOptions !== void 0;
	const transitionDuration = (entering) => (entering ? motionOptions?.enterDuration : motionOptions?.exitDuration) ?? motionOptions?.duration ?? (motionEnabled ? .16 : 0);
	const presence = createTransitionPresence(open, {
		duration: () => transitionDuration(open()),
		ease: motionOptions?.ease ?? motionOptions?.enterEase ?? motionOptions?.exitEase ?? "easeOut",
		reducedMotion
	});
	const visualState = () => modalVisualState(open(), props.backdropFade);
	let trigger;
	let focusFrame = 0;
	let wasOpenForInitialFocus = false;
	const setOpen = (next, reason) => {
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next, reason);
	};
	const close = (reason = "programmatic") => setOpen(false, reason);
	const controls = { close: () => close("programmatic") };
	const layer = createOverlayLayer({
		open,
		plane: () => "modal",
		onDismiss: (reason) => close(reason === "outside" ? "backdrop" : "escape"),
		closeOnEscape: () => props.closeOnEscape ?? true,
		closeOnOutside: () => props.closeOnBackdrop ?? true,
		restoreFocus: () => props.restoreFocus ?? true,
		returnFocus: () => trigger
	});
	const handleEscape = (event) => layer.onEscape(event);
	createEffect(() => open(), (isOpen) => {
		if (isOpen && !wasOpenForInitialFocus && props.initialFocus) {
			cancelAnimationFrame(focusFrame);
			focusFrame = requestAnimationFrame(() => {
				focusFrame = 0;
				props.initialFocus?.()?.focus();
			});
		} else if (!isOpen) {
			if (focusFrame) cancelAnimationFrame(focusFrame);
			focusFrame = 0;
		}
		wasOpenForInitialFocus = isOpen;
	});
	onCleanup(() => {
		if (focusFrame) cancelAnimationFrame(focusFrame);
	});
	const triggerProps = {
		ref: (node) => {
			trigger = node;
		},
		onClick: (event) => {
			event.stopPropagation();
			setOpen(true, "trigger");
		},
		onKeyDown: handleEscape,
		"aria-haspopup": "dialog",
		get "aria-expanded"() {
			return open();
		}
	};
	const overlay = createComponent(Show, {
		get when() {
			return presence.mounted();
		},
		get children() {
			return createComponent(Portal, {
				plane: "modal",
				role: "presentation",
				get focusContained() {
					return visualState().active;
				},
				get interactionBlocked() {
					return !visualState().active;
				},
				get class() {
					const visual = visualState();
					return mergeClasses(visual.retainBackdropVisuals && props.backdropClass);
				},
				get style() {
					const visual = visualState();
					return {
						position: "absolute",
						left: 0,
						top: 0,
						width: "100%",
						height: "100%",
						display: "flex",
						"align-items": "center",
						"justify-content": "center",
						...props.backdropStyle,
						...visual.transparentBackdrop ? { "background-color": rgba(0) } : void 0,
						"pointer-events": visual.active ? "auto" : "none",
						opacity: props.backdropFade === false ? 1 : presence.progress(),
						"z-index": layer.zIndex()
					};
				},
				onClick: layer.onOutside,
				onKeyDown: handleEscape,
				get children() {
					return createComponent(View, {
						get ref() {
							return props.contentRef;
						},
						get role() {
							return props.contentRole ?? "dialog";
						},
						"aria-modal": "true",
						get "aria-label"() {
							return props["aria-label"];
						},
						get class() {
							return props.contentClass;
						},
						get style() {
							return {
								...props.contentStyle,
								opacity: props.contentFade === false || props.backdropFade !== false ? 1 : presence.progress()
							};
						},
						get shadows() {
							return props.contentShadows;
						},
						get transform() {
							const base = modalMotionTransform(motionOptions, presence.progress());
							return props.contentTransform?.(base, presence.progress()) ?? base;
						},
						get interactionBlocked() {
							return !visualState().active;
						},
						get "aria-hidden"() {
							return visualState().active ? void 0 : "true";
						},
						onClick: (event) => event.stopPropagation(),
						get children() {
							return createComponent(OverlayPlaneProvider, {
								plane: "modal",
								get children() {
									return typeof props.children === "function" ? props.children(controls) : props.children;
								}
							});
						}
					});
				}
			});
		}
	});
	return [untrack(() => props.trigger?.(triggerProps)), overlay];
}
//#endregion
export { Modal, modalMotionTransform, modalVisualState };

//# sourceMappingURL=modal.mjs.map