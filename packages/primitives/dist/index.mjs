import { For, Show, createComponent, createEffect, createMemo, createSignal, omit, onCleanup } from "solid-js";
import { Portal, applyRef, createComponent as createComponent$1, createElement, insert, memo, mergeProps, ref, setProp, spread, useHost } from "@wabou/solid-renderer";
import { number, px, rotate2d, translate2d } from "@wabou/style";
import { createPulse, createRotation, createTransition } from "@wabou/animation";
import { arrow, autoPlacement, computePosition, flip, offset, shift, size } from "@floating-ui/core";
//#region src/animation-frame.ts
/**
* Drive explicit paint state from the native host's animation clock.
* Return `false` to stop scheduling frames before the owner is disposed.
*/
function createAnimationFrame(callback) {
	let frame = 0;
	let active = true;
	const tick = (timestamp) => {
		if (!active) return;
		if (callback(timestamp) === false) {
			active = false;
			return;
		}
		frame = requestAnimationFrame(tick);
	};
	frame = requestAnimationFrame(tick);
	const stop = () => {
		if (!active) return;
		active = false;
		cancelAnimationFrame(frame);
	};
	onCleanup(stop);
	return stop;
}
//#endregion
//#region src/focus.ts
/** Reactive focus state and event bindings for a single target. */
function createFocus() {
	const [focused, setFocused] = createSignal(false);
	return {
		focused,
		bindings: {
			onFocus: () => setFocused(true),
			onBlur: () => setFocused(false)
		}
	};
}
/** Reactive equivalent of `:focus-within`, using bubbling focus events. */
function createFocusWithin() {
	const [focusWithin, setFocusWithin] = createSignal(false);
	return {
		focusWithin,
		bindings: {
			onFocusIn: () => setFocusWithin(true),
			onFocusOut: () => setFocusWithin(false)
		}
	};
}
//#endregion
//#region src/hover.ts
/** Reactive hover state and event bindings for a single target. */
function createHover() {
	const [hovered, setHovered] = createSignal(false);
	return {
		hovered,
		bindings: {
			onPointerEnter: () => setHovered(true),
			onPointerLeave: () => setHovered(false)
		}
	};
}
//#endregion
//#region src/press.ts
/** Reactive pointer-press state with disabled-aware activation. */
function createPress(options = {}) {
	const [pressed, setPressed] = createSignal(false);
	const disabled = () => typeof options.disabled === "function" ? options.disabled() : !!options.disabled;
	const release = () => setPressed(false);
	return {
		pressed,
		bindings: {
			onPointerDown: () => {
				if (!disabled()) setPressed(true);
			},
			onPointerUp: release,
			onPointerCancel: release,
			onPointerLeave: release,
			onClick: (event) => {
				if (!disabled()) options.onPress?.(event);
			}
		}
	};
}
/** CSS `:active`-like state without an activation callback. */
function createActive(disabled) {
	const { pressed, bindings } = createPress({ disabled });
	return {
		active: pressed,
		bindings
	};
}
//#endregion
//#region src/button.tsx
const ACCENTS = {
	neutral: "#475569",
	sky: "#0284c7",
	amber: "#d97706"
};
/** Headless button state and event normalization. */
function createButton(options = {}) {
	const hover = createHover();
	const focus = createFocus();
	const disabled = () => typeof options.disabled === "function" ? options.disabled() : options.disabled ?? false;
	const selected = () => typeof options.selected === "function" ? options.selected() : options.selected ?? false;
	const press = createPress({
		disabled,
		onPress: (event) => options.onPress?.(event)
	});
	return {
		state: () => ({
			hovered: hover.hovered(),
			pressed: press.pressed(),
			focused: focus.focused(),
			selected: selected(),
			disabled: disabled()
		}),
		bindings: {
			onPointerEnter: hover.bindings.onPointerEnter,
			onPointerLeave: () => {
				hover.bindings.onPointerLeave();
				press.bindings.onPointerLeave();
			},
			onPointerDown: press.bindings.onPointerDown,
			onPointerUp: press.bindings.onPointerUp,
			onPointerCancel: press.bindings.onPointerCancel,
			onFocus: focus.bindings.onFocus,
			onBlur: focus.bindings.onBlur,
			onClick: press.bindings.onClick,
			onKeyDown: (event) => {
				options.onKeyDown?.(event);
				if (event.defaultPrevented || event.repeat) return;
				if (event.key !== "Enter" && event.key !== " ") return;
				event.preventDefault();
				if (!disabled()) options.onPress?.(event);
			}
		}
	};
}
/**
* A native button with consistent hover, pressed, focus and disabled feedback.
*
* Interaction styling is deliberately implemented with reactive inline styles:
* applications do not need CSS pseudo-class support to get a responsive button.
*/
function Button(props) {
	const disabled = () => props.disabled ?? false;
	const primitive = createButton({
		disabled,
		selected: () => props.selected ?? false,
		onPress: (event) => props.onClick?.(event),
		onKeyDown: (event) => props.onKeyDown?.(event)
	});
	const variant = () => props.variant ?? "solid";
	const accent = () => ACCENTS[props.tone ?? "neutral"];
	const state = primitive.state;
	const customStyle = () => typeof props.style === "function" ? props.style(state()) : props.style;
	const defaultStyle = () => props.unstyled ? {
		display: "flex",
		"align-items": "center",
		"flex-shrink": 0,
		"white-space": "nowrap",
		"user-select": props.selectable ? "text" : "none"
	} : {
		display: "flex",
		"align-items": "center",
		"justify-content": "center",
		"flex-shrink": 0,
		"white-space": "nowrap",
		"user-select": props.selectable ? "text" : "none",
		"min-height": "32px",
		padding: "6px 12px",
		"border-radius": "6px",
		"border-width": "1px",
		"border-color": state().focused ? "#7dd3fc" : "#64748b",
		"background-color": background(),
		color: "#f8fafc",
		opacity: disabled() ? .45 : 1
	};
	const background = () => {
		if (variant() === "ghost" && !props.selected) {
			if (state().pressed) return "#1e293b";
			return state().hovered ? "#334155" : "transparent";
		}
		if (state().pressed) return "#1e293b";
		if (state().hovered && !props.selected) return "#334155";
		return accent();
	};
	var _el$ = createElement("button");
	var _ref$ = props.ref;
	(typeof _ref$ === "function" || Array.isArray(_ref$)) && ref(() => {
		return _ref$;
	}, _el$);
	setProp(_el$, "type", "button");
	spread(_el$, mergeProps({
		get disabled() {
			return disabled();
		},
		get title() {
			return props.title;
		},
		get role() {
			return props.role;
		},
		get ["aria-haspopup"]() {
			return memo(() => {
				return typeof props["aria-haspopup"] === "boolean";
			})() ? String(props["aria-haspopup"]) : props["aria-haspopup"];
		},
		get ["aria-expanded"]() {
			return memo(() => {
				return props["aria-expanded"] === void 0;
			})() ? void 0 : String(props["aria-expanded"]);
		},
		get ["aria-controls"]() {
			return props["aria-controls"];
		},
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-checked"]() {
			return memo(() => {
				return props["aria-checked"] === void 0;
			})() ? void 0 : String(props["aria-checked"]);
		},
		get ["aria-selected"]() {
			return memo(() => {
				return props["aria-selected"] === void 0;
			})() ? void 0 : String(props["aria-selected"]);
		},
		get ["aria-pressed"]() {
			return memo(() => {
				return props["aria-pressed"] === void 0;
			})() ? void 0 : String(props["aria-pressed"]);
		},
		get ["class"]() {
			return memo(() => {
				return typeof props.class === "function";
			})() ? props.class(state()) : props.class;
		},
		get classList() {
			return memo(() => {
				return typeof props.classList === "function";
			})() ? props.classList(state()) : props.classList;
		},
		get style() {
			return {
				...defaultStyle(),
				...customStyle()
			};
		}
	}, () => {
		return primitive.bindings;
	}), true);
	insert(_el$, () => {
		return props.children;
	});
	return _el$;
}
//#endregion
//#region src/view.ts
function primitive(tag, props) {
	const node = createElement(tag);
	spread(node, props, false);
	return node;
}
/** A layout container. Text content should be placed in a {@link Text}. */
function View(props) {
	return primitive("view", props);
}
/**
* A single measured text run.
*
* Static and reactive child text nodes are concatenated by the native host and
* participate in the parent layout as one item.
*/
function Text(props) {
	return primitive("text", props);
}
/** A static SVG asset rendered through the native usvg/Vello pipeline. */
function Svg(props) {
	return primitive("svg", props);
}
/** A theme-colored SVG icon with stable native sizing and semantics. */
function Icon(props) {
	const rest = omit(props, "source", "size", "fill", "label");
	const node = createElement("svg");
	spread(node, rest, false);
	spread(node, {
		get source() {
			return props.fill && props.fill !== "none" ? props.source.replace("fill=\"none\"", `fill="${props.fill}"`) : props.source;
		},
		get width() {
			return String(props.size ?? 24);
		},
		get height() {
			return String(props.size ?? 24);
		},
		get role() {
			return props.label ? "img" : void 0;
		},
		get "aria-label"() {
			return props.label;
		},
		get "aria-hidden"() {
			return props.label ? void 0 : "true";
		}
	}, false);
	return node;
}
/** A replaced image node rendered by the native host. */
function Image(props) {
	return primitive("img", props);
}
/** An explicit network-backed image with bounded decoding and host caching. */
function NetworkImage(props) {
	const rest = omit(props, "url", "format", "cache");
	const node = createElement("img");
	spread(node, rest, false);
	spread(node, { get source() {
		return {
			kind: "network",
			url: props.url,
			format: props.format,
			cache: props.cache
		};
	} }, false);
	return node;
}
/** A native multiline text editor with wrapping, selection, and scrolling. */
function TextArea(props) {
	return primitive("textarea", props);
}
/** Native password editor whose value remains in a Rust SecretStore. */
function PasswordInput(props) {
	return primitive("password-input", props);
}
/** Experimental native editor for config and script-sized documents. */
function CodeEditor(props) {
	return primitive("code-editor", props);
}
//#endregion
//#region src/layout.tsx
const join$1 = (...values) => values.filter(Boolean).join(" ");
/** Horizontal flex container. No wrapper node is added beyond the host View. */
function Row(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join$1("flex flex-row", props.class);
	} }));
}
/** Vertical flex container. No wrapper node is added beyond the host View. */
function Column(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join$1("flex flex-col", props.class);
	} }));
}
/** Flex container that centers children on both axes. */
function Center(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join$1("flex items-center justify-center", props.class);
	} }));
}
//#endregion
//#region src/measure.ts
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
//#endregion
//#region src/presence.ts
/** Explicit mount lifecycle for content whose exit must finish before removal. */
function createPresence(open) {
	const [phase, setPhase] = createSignal(open() ? "present" : "unmounted");
	createEffect(open, (isOpen) => {
		if (isOpen) {
			if (phase() === "unmounted" || phase() === "exiting") setPhase("entering");
		} else if (phase() === "present" || phase() === "entering") setPhase("exiting");
	});
	return {
		phase,
		mounted: () => phase() !== "unmounted",
		finishEnter() {
			if (open() && phase() === "entering") setPhase("present");
		},
		finishExit() {
			if (!open() && phase() === "exiting") setPhase("unmounted");
		}
	};
}
//#endregion
//#region src/collapsible-presence.tsx
/**
* Measured disclosure content with explicit presence and subtree isolation.
* Height participates in layout while a subtree opacity layer masks glyphs
* crossing the moving clip edge.
*/
function CollapsiblePresence(props) {
	const open = () => props.open;
	const initiallyOpen = open();
	const presence = createPresence(open);
	let initialMeasurement = true;
	let heightTransition;
	const opacityTransition = createTransition(() => open() ? 1 : 0, {
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion: () => props.reducedMotion ?? false
	});
	const measured = createMeasuredSize({ onChange(size) {
		if (initialMeasurement && initiallyOpen && !props.animateInitial) heightTransition?.jump(size.height);
		initialMeasurement = false;
	} });
	const transitionOptions = () => ({
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion: () => props.reducedMotion ?? false
	});
	heightTransition = createTransition(() => open() && measured.measured() ? measured.height() : 0, {
		...transitionOptions(),
		onComplete(value) {
			if (value === 0 && !open()) presence.finishExit();
			else if (open()) presence.finishEnter();
		}
	});
	createEffect(() => [
		open(),
		measured.measured(),
		measured.height()
	], ([isOpen, isMeasured, height]) => {
		if (isOpen && isMeasured && height === 0) presence.finishEnter();
	});
	const style = () => ({
		...props.style,
		height: px(heightTransition?.value() ?? 0),
		opacity: number(opacityTransition.value())
	});
	return createComponent$1(View, {
		get ["class"]() {
			return props.class;
		},
		classList: { "overflow-hidden": true },
		get style() {
			return style();
		},
		get inert() {
			return open() ? void 0 : "";
		},
		get ["aria-hidden"]() {
			return open() ? void 0 : "true";
		},
		get children() {
			return createComponent$1(Show, {
				get when() {
					return presence.mounted();
				},
				get children() {
					return createComponent$1(View, {
						ref(r$) {
							var _ref$ = measured.ref;
							typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
						},
						get children() {
							return createComponent$1(View, {
								get ["class"]() {
									return props.contentClass;
								},
								get style() {
									return props.contentStyle;
								},
								get children() {
									return props.children;
								}
							});
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/overlay-layer.ts
const openLayers = [];
function removeLayer(token) {
	const index = openLayers.lastIndexOf(token);
	if (index >= 0) openLayers.splice(index, 1);
}
function createOverlayLayer(options) {
	const token = Symbol("wabou-overlay-layer");
	let wasOpen = options.open();
	if (wasOpen) openLayers.push(token);
	const restoreFocus = () => {
		if (options.restoreFocus?.() ?? true) options.returnFocus?.()?.focus();
	};
	createEffect(options.open, (open) => {
		if (open && !wasOpen) {
			removeLayer(token);
			openLayers.push(token);
		} else if (!open && wasOpen) {
			removeLayer(token);
			restoreFocus();
		}
		wasOpen = open;
	});
	onCleanup(() => {
		removeLayer(token);
		if (wasOpen) restoreFocus();
		wasOpen = false;
	});
	const isTopmost = () => openLayers.at(-1) === token;
	return {
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
//#region src/modal.tsx
/**
* A native modal plane with host-enforced focus, hit-test, and accessibility
* isolation. Visual styling remains explicit so applications can own it.
*/
function Modal(props) {
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const open = () => props.open ?? uncontrolledOpen();
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
		onDismiss: (reason) => close(reason === "outside" ? "backdrop" : "escape"),
		closeOnEscape: () => props.closeOnEscape ?? true,
		closeOnOutside: () => props.closeOnBackdrop ?? true,
		restoreFocus: () => props.restoreFocus ?? true,
		returnFocus: () => trigger
	});
	const handleEscape = (event) => layer.onEscape(event);
	createEffect(open, (isOpen) => {
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
			return open();
		},
		get children() {
			return createComponent(Portal, {
				plane: "modal",
				role: "presentation",
				get class() {
					return props.backdropClass;
				},
				get style() {
					return {
						position: "absolute",
						left: 0,
						top: 0,
						width: "100%",
						height: "100%",
						display: "flex",
						"align-items": "center",
						"justify-content": "center",
						...props.backdropStyle
					};
				},
				onClick: layer.onOutside,
				onKeyDown: handleEscape,
				get children() {
					return createComponent(View, {
						get ref() {
							return props.contentRef;
						},
						role: "dialog",
						"aria-modal": "true",
						get "aria-label"() {
							return props["aria-label"];
						},
						get class() {
							return props.contentClass;
						},
						get style() {
							return props.contentStyle;
						},
						onClick: (event) => event.stopPropagation(),
						get children() {
							return typeof props.children === "function" ? props.children(controls) : props.children;
						}
					});
				}
			});
		}
	});
	return [props.trigger?.(triggerProps), overlay];
}
//#endregion
//#region src/motion.tsx
function bindPlayback(controls, props) {
	createEffect(() => props.speed ?? 1, (speed) => {
		controls.speed = speed;
	});
	createEffect(() => props.paused, (paused) => {
		if (paused) controls.pause();
		else controls.play();
	});
}
/** A single native View whose contents rotate around its border-box center. */
function Spin(props) {
	const motion = props;
	const view = omit(props, "duration", "speed", "paused");
	const rotation = createRotation({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1
	});
	bindPlayback(rotation.controls, motion);
	return createComponent$1(View, mergeProps(view, { get transform() {
		return rotation.transform();
	} }));
}
/** A single native View with a repeating opacity pulse. */
function Pulse(props) {
	const motion = props;
	const view = omit(props, "duration", "speed", "paused", "from", "to", "style");
	const pulse = createPulse({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1,
		from: motion.from,
		to: motion.to
	});
	bindPlayback(pulse.controls, motion);
	const style = () => ({
		...motion.style ?? {},
		opacity: pulse.value()
	});
	return createComponent$1(View, mergeProps(view, { get style() {
		return style();
	} }));
}
//#endregion
//#region src/notification.ts
const finiteNonNegative = (value, fallback) => Number.isFinite(value) ? Math.max(0, value) : fallback;
/** Create an owner-scoped notification queue with explicit JavaScript timers. */
function createNotifications(options = {}) {
	const [items, setItems] = createSignal([]);
	const records = /* @__PURE__ */ new Map();
	const defaultDuration = finiteNonNegative(options.defaultDuration ?? 5e3, 5e3);
	const configuredLimit = options.limit ?? 5;
	const limit = Number.isFinite(configuredLimit) ? Math.max(1, Math.floor(configuredLimit)) : 5;
	let nextId = 1;
	const dismiss = (id, reason = "programmatic") => {
		const record = records.get(id);
		if (!record) return false;
		if (record.timer !== void 0) clearTimeout(record.timer);
		records.delete(id);
		setItems((current) => current.filter((item) => item.id !== id));
		record.item.onDismiss?.(reason);
		return true;
	};
	const schedule = (record) => {
		if (!record.autoDismiss) return;
		if (record.remaining <= 0) {
			dismiss(record.item.id, "timeout");
			return;
		}
		record.startedAt = Date.now();
		record.timer = setTimeout(() => dismiss(record.item.id, "timeout"), record.remaining);
	};
	const notifications = {
		items,
		show(input) {
			while (records.size >= limit) {
				const oldest = items()[0];
				if (!oldest || !dismiss(oldest.id, "overflow")) break;
			}
			const duration = finiteNonNegative(input.duration ?? defaultDuration, defaultDuration);
			const item = {
				...input,
				id: nextId++,
				duration,
				priority: input.priority ?? "polite"
			};
			const record = {
				item,
				autoDismiss: duration > 0,
				pauseCount: 0,
				remaining: duration,
				startedAt: 0
			};
			records.set(item.id, record);
			setItems((current) => [...current, item]);
			schedule(record);
			return item.id;
		},
		dismiss,
		pause(id) {
			const record = records.get(id);
			if (!record) return;
			record.pauseCount++;
			if (record.pauseCount > 1 || record.timer === void 0) return;
			clearTimeout(record.timer);
			record.timer = void 0;
			record.remaining = Math.max(0, record.remaining - (Date.now() - record.startedAt));
		},
		resume(id) {
			const record = records.get(id);
			if (!record || record.pauseCount === 0) return;
			record.pauseCount--;
			if (record.pauseCount > 0 || record.timer !== void 0) return;
			schedule(record);
		},
		clear() {
			for (const id of [...records.keys()]) dismiss(id, "programmatic");
		}
	};
	onCleanup(() => {
		for (const record of records.values()) if (record.timer !== void 0) clearTimeout(record.timer);
		records.clear();
	});
	return notifications;
}
const alignment = (placement) => ({
	"align-items": placement.endsWith("start") ? "flex-start" : placement.endsWith("end") ? "flex-end" : "center",
	"justify-content": placement.startsWith("bottom") ? "flex-end" : "flex-start"
});
/** Render a non-blocking stack on the native floating overlay plane. */
function NotificationRegion(props) {
	const items = createComponent(For, {
		get each() {
			return props.notifications.items();
		},
		children: (item) => createComponent(View, {
			role: item.priority === "assertive" ? "alert" : "status",
			"aria-label": item["aria-label"],
			get class() {
				return `pointer-events-auto ${props.itemClass ?? ""}`;
			},
			get style() {
				return props.itemStyle;
			},
			onPointerEnter: () => props.notifications.pause(item.id),
			onPointerLeave: () => props.notifications.resume(item.id),
			onFocusIn: () => props.notifications.pause(item.id),
			onFocusOut: () => props.notifications.resume(item.id),
			get children() {
				return item.content({ dismiss: () => props.notifications.dismiss(item.id, "dismiss") });
			}
		})
	});
	return createComponent(Portal, {
		plane: "floating",
		role: "presentation",
		get class() {
			return `pointer-events-none ${props.class ?? ""}`;
		},
		get style() {
			const placement = props.placement ?? "top-end";
			return {
				position: "absolute",
				left: 0,
				top: 0,
				width: "100%",
				height: "100%",
				display: "flex",
				"flex-direction": "column",
				gap: 8,
				padding: 16,
				...alignment(placement),
				...props.style
			};
		},
		children: items
	});
}
//#endregion
//#region src/positioner.ts
/**
* Position two Wabou layout targets with Floating UI's renderer-independent
* geometry engine. Measurement remains host-owned and is supplied explicitly;
* no DOM-compatible Handle methods are required.
*/
function computeFloatingPosition(reference, floating, options) {
	const isRTL = options.platform.isRTL;
	return computePosition(reference, floating, {
		platform: {
			async getElementRects({ reference, floating }) {
				const [referenceRect, floatingRect] = await Promise.all([options.platform.getRect(reference), options.platform.getRect(floating)]);
				return {
					reference: referenceRect,
					floating: {
						x: 0,
						y: 0,
						width: floatingRect.width,
						height: floatingRect.height
					}
				};
			},
			getDimensions: (target) => options.platform.getRect(target),
			getClippingRect: ({ element }) => options.platform.getClippingRect(element),
			isElement: () => true,
			isRTL: isRTL ? (target) => isRTL(target) : void 0
		},
		placement: options.placement,
		strategy: options.strategy,
		middleware: options.middleware
	});
}
var LayoutTargetUnavailableError = class extends Error {
	name = "LayoutTargetUnavailableError";
};
/** Position two native handles from a single coherent Host layout snapshot. */
function computeHostFloatingPosition(reference, floating, host, options = {}) {
	const snapshot = host.layout.snapshot([reference, floating]);
	const nodes = new Map(snapshot.nodes.map((node) => [node.id, node]));
	const id = (target) => typeof target === "number" ? target : target.id;
	const rect = (target) => {
		const targetId = id(target);
		const node = nodes.get(targetId);
		if (!node) throw new LayoutTargetUnavailableError(`Layout target ${targetId} is not present in completed revision ${snapshot.revision}`);
		return node;
	};
	return computeFloatingPosition(reference, floating, {
		...options,
		platform: {
			getRect: (target) => rect(target).rect,
			getClippingRect: (target) => rect(target).clip
		}
	});
}
//#endregion
//#region src/popover.tsx
/** A root-layer floating panel positioned from native layout snapshots. */
function Popover(props) {
	const host = useHost();
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const [position, setPosition] = createSignal({
		x: 0,
		y: 0
	});
	const [positioned, setPositioned] = createSignal(false);
	const open = () => props.open ?? uncontrolledOpen();
	let anchor;
	let content;
	let frame = 0;
	let positionRequest = 0;
	let observer;
	const setOpen = (next, reason) => {
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next, reason);
	};
	const layer = createOverlayLayer({
		open,
		onDismiss: (reason) => setOpen(false, reason),
		closeOnEscape: () => props.closeOnEscape ?? true,
		returnFocus: () => anchor,
		restoreFocus: () => props.restoreFocus ?? true
	});
	const updatePosition = async () => {
		if (!open() || !anchor || !content) return;
		const request = ++positionRequest;
		try {
			const result = await computeHostFloatingPosition(anchor, content, host, {
				placement: props.placement ?? "bottom-start",
				middleware: [
					offset(props.offset ?? 6),
					flip(),
					shift({ padding: 8 })
				]
			});
			if (!open() || request !== positionRequest) return;
			setPosition({
				x: result.x,
				y: result.y
			});
			setPositioned(true);
		} catch (error) {
			if (error instanceof LayoutTargetUnavailableError && open() && request === positionRequest) {
				schedulePosition();
				return;
			}
			throw error;
		}
	};
	const schedulePosition = () => {
		cancelAnimationFrame(frame);
		frame = requestAnimationFrame(() => void updatePosition());
	};
	const observe = (node) => {
		observer?.observe(node);
		schedulePosition();
	};
	createEffect(open, (isOpen) => {
		if (!isOpen) {
			positionRequest++;
			setPositioned(false);
			observer?.disconnect();
			observer = void 0;
			return;
		}
		observer = new ResizeObserver(schedulePosition);
		if (anchor) observer.observe(anchor);
		frame = requestAnimationFrame(() => {
			frame = requestAnimationFrame(() => void updatePosition());
		});
	});
	onCleanup(() => {
		cancelAnimationFrame(frame);
		observer?.disconnect();
	});
	const handleEscape = (event) => layer.onEscape(event);
	return [memo(() => {
		return props.trigger({
			ref: (node) => {
				anchor = node;
				if (open()) observe(node);
			},
			onClick: (event) => {
				event.stopPropagation();
				setOpen(!open(), "trigger");
			},
			onKeyDown: handleEscape,
			"aria-haspopup": "dialog",
			"aria-expanded": open()
		});
	}), createComponent$1(Show, {
		get when() {
			return open();
		},
		get children() {
			return createComponent$1(Portal, {
				ref: (node) => {
					observe(node);
				},
				role: "presentation",
				style: {
					position: "absolute",
					left: 0,
					top: 0,
					width: "100%",
					height: "100%"
				},
				get onClick() {
					return layer.onOutside;
				},
				onKeyDown: handleEscape,
				onWheel: schedulePosition,
				get children() {
					return createComponent$1(View, {
						ref: (node) => {
							content = node;
							observe(node);
						},
						role: "dialog",
						get ["class"]() {
							return props.contentClass;
						},
						get style() {
							return {
								position: "absolute",
								left: positioned() ? `${position().x}px` : "-100000px",
								top: positioned() ? `${position().y}px` : "-100000px",
								...props.contentStyle
							};
						},
						onClick: (event) => event.stopPropagation(),
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
//#region src/scroll-area.tsx
const join = (...values) => values.filter(Boolean).join(" ");
const scrollAreaViewportClass = (className) => join("min-h-0 overflow-y-auto", className);
/**
* Vertical native scroll viewport with explicit sizing.
*
* The inner wrapper deliberately cannot shrink. This makes its intrinsic
* height become the viewport's scroll extent instead of allowing a flex
* parent to compress overflowing sections until no scroll range remains.
* The viewport itself deliberately does not grow: implicit `flex-1` makes a
* nested scroll area expand with an ancestor's intrinsic content instead of
* establishing its own scroll range.
*/
function ScrollArea(props) {
	return createComponent$1(View, {
		ref(r$) {
			var _ref$ = props.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.ref = r$;
		},
		get ["class"]() {
			return scrollAreaViewportClass(props.class);
		},
		get style() {
			return props.style;
		},
		get scrollbar() {
			return props.scrollbar;
		},
		get onScroll() {
			return props.onScroll;
		},
		get children() {
			return createComponent$1(View, {
				get ["class"]() {
					return join("flex-none flex flex-col min-h-full", props.contentClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/scroll-reset.ts
/** Reset one explicitly selected native viewport after its key changes. */
function createScrollReset(options) {
	const reset = () => {
		options.target()?.scrollTo({
			left: options.left ?? 0,
			top: options.top ?? 0
		});
	};
	createEffect(options.key, reset, { defer: true });
	return reset;
}
//#endregion
//#region src/shortcuts.ts
const MODIFIER_BITS = {
	shift: 1,
	control: 2,
	alt: 4,
	meta: 8
};
const ALL_MODIFIERS = 15;
/**
* Compile declarative application shortcuts into one keydown binding.
*
* Chords use names such as `Primary+T`, `Control+Tab`, and
* `Control+Shift+Tab`. `Primary` resolves to Command on macOS and Control on
* other platforms while still requiring an exact modifier match.
*/
function createShortcuts(shortcuts) {
	const compiled = Object.entries(shortcuts).map(([chord, value]) => compileShortcut(chord, value));
	assertNoAmbiguousShortcuts(compiled);
	const handleKeyDown = (event) => {
		const key = normalizeKey(event.key);
		const modifiers = event.mods & ALL_MODIFIERS;
		const shortcut = compiled.find((candidate) => candidate.key === key && (candidate.primary ? event.primary && candidate.modifierMasks.includes(modifiers & ~(MODIFIER_BITS.control | MODIFIER_BITS.meta)) : candidate.modifierMasks.includes(modifiers)));
		if (shortcut === void 0 || event.repeat && !shortcut.allowRepeat) return false;
		if (shortcut.preventDefault !== false) event.preventDefault();
		shortcut.handler(event);
		return true;
	};
	return {
		handleKeyDown,
		bindings: { onKeyDown: handleKeyDown }
	};
}
function compileShortcut(chord, value) {
	const parts = chord.split("+").map((part) => part.trim()).filter(Boolean);
	if (parts.length === 0) throw new Error("Shortcut chord cannot be empty");
	const key = normalizeKey(parts.pop());
	if (key.length === 0) throw new Error(`Shortcut chord has no key: ${chord}`);
	let mask = 0;
	let primary = false;
	for (const rawModifier of parts) {
		const modifier = rawModifier.toLowerCase();
		const bit = modifier === "shift" ? MODIFIER_BITS.shift : modifier === "control" || modifier === "ctrl" ? MODIFIER_BITS.control : modifier === "alt" || modifier === "option" ? MODIFIER_BITS.alt : modifier === "meta" || modifier === "cmd" || modifier === "command" ? MODIFIER_BITS.meta : void 0;
		if (modifier === "primary" || modifier === "mod") {
			if (primary) throw new Error(`Duplicate modifier in shortcut: ${chord}`);
			primary = true;
		} else if (bit === void 0) throw new Error(`Unknown shortcut modifier '${rawModifier}' in ${chord}`);
		else if ((mask & bit) !== 0) throw new Error(`Duplicate modifier in shortcut: ${chord}`);
		else mask |= bit;
	}
	if (primary && (mask & (MODIFIER_BITS.control | MODIFIER_BITS.meta)) !== 0) throw new Error(`Primary cannot be combined with Control or Meta: ${chord}`);
	return {
		...typeof value === "function" ? { handler: value } : value,
		chord,
		key,
		modifierMasks: [mask],
		primary
	};
}
function assertNoAmbiguousShortcuts(shortcuts) {
	const owners = /* @__PURE__ */ new Map();
	for (const shortcut of shortcuts) {
		const masks = shortcut.primary ? shortcut.modifierMasks.flatMap((mask) => [mask | MODIFIER_BITS.control, mask | MODIFIER_BITS.meta]) : shortcut.modifierMasks;
		for (const mask of masks) {
			const signature = `${mask}:${shortcut.key}`;
			const previous = owners.get(signature);
			if (previous !== void 0) throw new Error(`Ambiguous shortcuts '${previous}' and '${shortcut.chord}'`);
			owners.set(signature, shortcut.chord);
		}
	}
}
function normalizeKey(key) {
	return key.trim().toLowerCase();
}
//#endregion
//#region src/tabs.ts
/**
* Stateful tab collection with stable identity and deterministic activation.
*
* Closing the active tab selects its right-hand neighbour, or the previous
* tab when the closed tab was last. Reordering never changes the active key.
*/
function createTabs(options) {
	const initialTabs = [...options.initialTabs ?? []];
	assertUniqueKeys(initialTabs, options.key);
	const initialActiveKey = options.initialActiveKey !== void 0 && initialTabs.some((tab) => options.key(tab) === options.initialActiveKey) ? options.initialActiveKey : initialTabs[0] === void 0 ? void 0 : options.key(initialTabs[0]);
	const [tabs, setTabs] = createSignal(initialTabs);
	const [activeKey, setActiveKey] = createSignal(() => initialActiveKey);
	const focusTargets = /* @__PURE__ */ new Map();
	const commitActiveKey = (key) => {
		if (activeKey() === key) return false;
		setActiveKey(() => key);
		options.onActiveChange?.(key);
		return true;
	};
	const select = (key) => {
		if (!tabs().some((tab) => options.key(tab) === key)) return false;
		return commitActiveKey(key);
	};
	const selectAt = (index) => {
		const values = tabs();
		if (values.length === 0) return false;
		const normalized = (index % values.length + values.length) % values.length;
		return select(options.key(values[normalized]));
	};
	const activeIndex = () => {
		const current = activeKey();
		return current === void 0 ? -1 : tabs().findIndex((tab) => options.key(tab) === current);
	};
	const focus = (key) => {
		const target = focusTargets.get(key);
		if (target === void 0) return false;
		target.focus();
		return true;
	};
	const activateAt = (index, moveFocus) => {
		const values = tabs();
		if (values.length === 0) return false;
		const normalized = (index % values.length + values.length) % values.length;
		const key = options.key(values[normalized]);
		select(key);
		if (moveFocus) focus(key);
		return true;
	};
	return {
		tabs,
		activeKey,
		activeTab: createMemo(() => {
			const current = activeKey();
			return current === void 0 ? void 0 : tabs().find((tab) => options.key(tab) === current);
		}),
		select,
		selectNext: () => selectAt(activeIndex() + 1),
		selectPrevious: () => selectAt(activeIndex() <= 0 ? tabs().length - 1 : activeIndex() - 1),
		selectFirst: () => selectAt(0),
		selectLast: () => selectAt(tabs().length - 1),
		add: (tab, addOptions = {}) => {
			const key = options.key(tab);
			const current = tabs();
			if (current.some((candidate) => options.key(candidate) === key)) return false;
			const index = Math.max(0, Math.min(addOptions.index ?? current.length, current.length));
			const next = [...current];
			next.splice(index, 0, tab);
			setTabs(next);
			if (activeKey() === void 0 || addOptions.activate !== false) commitActiveKey(key);
			return true;
		},
		close: (key) => {
			const current = tabs();
			const index = current.findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const next = current.filter((_, candidateIndex) => candidateIndex !== index);
			setTabs(next);
			focusTargets.delete(key);
			if (activeKey() === key) {
				const neighbour = next[Math.min(index, next.length - 1)];
				commitActiveKey(neighbour === void 0 ? void 0 : options.key(neighbour));
			}
			return true;
		},
		move: (key, requestedIndex) => {
			const current = tabs();
			const index = current.findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const target = Math.max(0, Math.min(requestedIndex, current.length - 1));
			if (target === index) return false;
			const next = [...current];
			const [tab] = next.splice(index, 1);
			next.splice(target, 0, tab);
			setTabs(next);
			return true;
		},
		register: (key, node) => {
			if (tabs().some((tab) => options.key(tab) === key)) focusTargets.set(key, node);
		},
		focus,
		handleKeyDown: (key, event) => {
			const index = tabs().findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const horizontal = options.orientation !== "vertical";
			const target = event.key === "Home" ? 0 : event.key === "End" ? tabs().length - 1 : horizontal && event.key === "ArrowRight" || !horizontal && event.key === "ArrowDown" ? index + 1 : horizontal && event.key === "ArrowLeft" || !horizontal && event.key === "ArrowUp" ? index - 1 : void 0;
			if (target === void 0) return false;
			event.preventDefault?.();
			return activateAt(target, true);
		}
	};
}
function assertUniqueKeys(tabs, key) {
	const keys = /* @__PURE__ */ new Set();
	for (const tab of tabs) {
		const value = key(tab);
		if (keys.has(value)) throw new Error(`Duplicate tab key: ${String(value)}`);
		keys.add(value);
	}
}
//#endregion
export { Button, Center, CodeEditor, CollapsiblePresence, Column, Icon, Image, Modal, NetworkImage, NotificationRegion, PasswordInput, Popover, Pulse, Row, ScrollArea, Spin, Svg, Text, TextArea, View, arrow, autoPlacement, computeFloatingPosition, computeHostFloatingPosition, createActive, createAnimationFrame, createButton, createFocus, createFocusWithin, createHover, createMeasuredSize, createNotifications, createOverlayLayer, createPresence, createPress, createScrollReset, createShortcuts, createTabs, flip, offset, rotate2d, shift, size, translate2d };

//# sourceMappingURL=index.mjs.map