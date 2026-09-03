import { t as __exportAll } from "./rolldown-runtime-D7D4PA-g.mjs";
import { PathBuilder, bindJsonCapability, useHost, useWindow } from "@wabou/core";
import { mergeClasses, number, px, rgba as rgba$1, rotate2d, rotate2d as rotate2d$1, scale2d, translate2d, translate2d as translate2d$1 } from "@wabou/core/style";
import { animateValue, interpolate } from "motion-dom";
import { For, Show, createComponent, createContext, createEffect, createMemo, createSignal, omit, onCleanup, untrack, useContext } from "solid-js";
import { Portal, TEXT_BEHAVIOR, applyRef, createComponent as createComponent$1, createElement, memo, mergeProps, observeGlobalPointerEvent, spread, useHost as useHost$1 } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/animation/config.tsx
const DEFAULT_MOTION_CONFIG = Object.freeze({ reducedMotion: () => useWindow().reducedMotion() });
const MotionConfigContext = createContext(DEFAULT_MOTION_CONFIG);
/** Application-level motion policy inherited by all styled Wabou components. */
function MotionConfigProvider(props) {
	const parent = useMotionConfig();
	return createComponent$1(MotionConfigContext, {
		value: { reducedMotion: () => props.reducedMotion ?? parent.reducedMotion() },
		get children() {
			return props.children;
		}
	});
}
function useMotionConfig() {
	return useContext(MotionConfigContext);
}
function useReducedMotion() {
	return useMotionConfig().reducedMotion;
}
//#endregion
//#region src/animation/index.ts
var Controls = class {
	backend;
	constructor(backend) {
		this.backend = backend;
	}
	get time() {
		return this.backend.time;
	}
	set time(value) {
		this.backend.time = value;
	}
	get speed() {
		return this.backend.speed;
	}
	set speed(value) {
		this.backend.speed = value;
	}
	get duration() {
		return this.backend.duration;
	}
	get state() {
		const state = this.backend.state;
		if (state === "running" || state === "paused" || state === "finished") return state;
		return "idle";
	}
	get finished() {
		return this.backend.finished.then(() => void 0);
	}
	play() {
		this.backend.play();
	}
	pause() {
		this.backend.pause();
	}
	stop() {
		this.backend.stop();
	}
	cancel() {
		this.backend.cancel();
	}
	complete() {
		this.backend.complete();
	}
	then(onfulfilled, onrejected) {
		return this.finished.then(onfulfilled, onrejected);
	}
};
function normalizeOptions(options) {
	const normalized = { ...options };
	for (const key of [
		"delay",
		"duration",
		"repeatDelay"
	]) {
		const seconds = options[key];
		if (typeof seconds === "number") normalized[key] = seconds * 1e3;
	}
	if (typeof options.visualDuration === "number") normalized.visualDuration = options.visualDuration * 1e3;
	return normalized;
}
/** Animate between two scalar values. */
function animate(from, to, options = {}) {
	return animateKeyframes([from, to], options);
}
/** Animate through two or more scalar keyframes. */
function animateKeyframes(keyframes, options = {}) {
	return new Controls(animateValue({
		...normalizeOptions(options),
		keyframes: [...keyframes]
	}));
}
const read = (value, fallback) => typeof value === "function" ? value() : value ?? fallback;
function positiveFinite(value, fallback) {
	return Number.isFinite(value) && value > 0 ? value : fallback;
}
/**
* Compile reactive Solid animation policy into a stable native timeline DTO.
* This accessor changes only when authored policy changes, never per frame.
*/
function createNativeLoopAnimation(options = {}) {
	return createMemo(() => ({
		kind: "loop",
		duration: positiveFinite(read(options.duration, 1), 1),
		speed: positiveFinite(read(options.speed, 1), 1),
		paused: read(options.paused, false),
		reducedMotion: read(options.reducedMotion, false)
	}));
}
/**
* Lifecycle-owned finite or repeating keyframe animation.
*
* This is the general primitive behind loops, pulses and component-specific
* effects. It owns cleanup and reduced-motion behavior so components don't
* need to coordinate raw Motion controls themselves.
*/
function createKeyframeAnimation(keyframes, options = {}) {
	const { reducedMotion, reducedValue = keyframes[keyframes.length - 1], onUpdate, ...animationOptions } = options;
	const authoredAutoplay = animationOptions.autoplay ?? true;
	const initiallyReduced = untrack(() => read(reducedMotion, false));
	const [box, setBox] = createSignal({ value: initiallyReduced ? reducedValue : keyframes[0] });
	const value = () => box().value;
	const controls = animateKeyframes(keyframes, {
		...animationOptions,
		autoplay: authoredAutoplay && !initiallyReduced,
		onUpdate(next) {
			setBox({ value: next });
			onUpdate?.(next);
		}
	});
	let initialized = false;
	let resumeAfterReduction = authoredAutoplay;
	createEffect(() => read(reducedMotion, false), (reduced) => {
		if (reduced) {
			if (initialized) resumeAfterReduction = controls.state === "running";
			controls.pause();
			setBox({ value: reducedValue });
			onUpdate?.(reducedValue);
		} else if (initialized && resumeAfterReduction) controls.play();
		initialized = true;
	});
	onCleanup(() => controls.stop());
	return {
		value,
		controls
	};
}
function createInterpolation(source, input, output, options = {}) {
	if (input.length === 0 || input.length !== output.length) throw new RangeError("animation input and output ranges must have equal non-zero lengths");
	if (!input.every(Number.isFinite)) throw new RangeError("animation input range must contain only finite numbers");
	const transform = interpolate([...input], [...output], options);
	return () => transform(source());
}
/**
* Lifecycle-owned scalar transition that retargets from its current value.
*
* Unlike a one-shot animation, changing `target` while a run is active does
* not restart from the previous keyframe. This makes it suitable for rapidly
* toggled disclosure, hover and selection state.
*/
function createTransition(target, options = {}) {
	const [value, setValue] = createSignal(untrack(() => read(options.initial, target())), { ownedWrite: true });
	const [state, setState] = createSignal("idle", { ownedWrite: true });
	let controls;
	let generation = 0;
	const stop = () => {
		generation++;
		controls?.stop();
		controls = void 0;
		setState("idle");
	};
	const jump = (next) => {
		stop();
		setValue(next);
		options.onUpdate?.(next);
		options.onComplete?.(next);
	};
	createEffect(() => [
		target(),
		read(options.reducedMotion, false),
		untrack(value)
	], ([next, reduced, current]) => {
		if (Object.is(next, current)) return;
		if (reduced || options.type === false || options.duration === 0) {
			jump(next);
			return;
		}
		const run = ++generation;
		controls?.stop();
		setState("running");
		const { initial: _initial, reducedMotion: _reducedMotion, onUpdate, onComplete, ...animationOptions } = options;
		controls = animate(current, next, {
			...animationOptions,
			onUpdate(current) {
				if (run !== generation) return;
				setValue(current);
				onUpdate?.(current);
			},
			onComplete() {
				if (run !== generation) return;
				controls = void 0;
				setValue(next);
				setState("finished");
				onComplete?.(next);
			}
		});
	});
	onCleanup(stop);
	return {
		value,
		state,
		jump,
		stop
	};
}
function createRepeatingAnimation(keyframes, options) {
	return createKeyframeAnimation(keyframes, options);
}
/**
* Lifecycle-owned repeating scalar animation for Solid components.
*
* The controls stop automatically with the current Solid owner.
*/
function createLoop(options = {}) {
	const from = options.from ?? 0;
	const to = options.to ?? 1;
	const { from: _from, to: _to, reducedMotion, reducedValue = from, onUpdate, ...animationOptions } = options;
	return createRepeatingAnimation([from, to], {
		duration: 1,
		ease: "linear",
		repeat: Infinity,
		...animationOptions,
		reducedMotion,
		reducedValue,
		onUpdate
	});
}
function normalizeSweepGeometry(extent, itemRatio) {
	return {
		extent: Number.isFinite(extent) ? Math.max(0, extent) : 0,
		itemRatio: Number.isFinite(itemRatio) && itemRatio > 0 ? Math.min(1, itemRatio) : .4
	};
}
/**
* Move an item completely across one measured axis using only a runtime
* transform. Both repeat boundaries remain outside the container, avoiding a
* visible reset and avoiding per-frame layout invalidation.
*/
function createSweep(options) {
	const { extent, itemRatio, axis = "horizontal", ...loopOptions } = options;
	const loop = createLoop(loopOptions);
	const geometry = () => normalizeSweepGeometry(read(extent, 0), read(itemRatio, .4));
	const offset = () => {
		const current = geometry();
		return current.extent * (loop.value() * (1 + current.itemRatio) - current.itemRatio);
	};
	return {
		...loop,
		offset,
		transform: () => axis === "vertical" ? translate2d(0, offset()) : translate2d(offset(), 0)
	};
}
/** Repeating center-pivoted rotation backed by Motion value animation. */
function createRotation(options = {}) {
	const loop = createLoop({
		...options,
		from: options.from ?? 0,
		to: options.to ?? Math.PI * 2
	});
	return {
		...loop,
		angle: loop.value,
		transform: () => rotate2d(loop.value())
	};
}
/** Repeating from→to→from value animation with automatic cleanup. */
function createPulse(options = {}) {
	const from = options.from ?? .5;
	const to = options.to ?? 1;
	const { from: _from, to: _to, reducedMotion, reducedValue = to, onUpdate, ...animationOptions } = options;
	return createRepeatingAnimation([
		from,
		to,
		from
	], {
		duration: 1,
		ease: "easeInOut",
		repeat: Infinity,
		...animationOptions,
		reducedMotion,
		reducedValue,
		onUpdate
	});
}
//#endregion
//#region src/primitives/animation-frame.ts
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
//#region src/primitives/focus.ts
let keyboardModality = false;
/** Reactive focus state and event bindings for a single target. */
function createFocus() {
	const [focused, setFocused] = createSignal(false);
	const [focusVisible, setFocusVisible] = createSignal(false);
	return {
		focused,
		focusVisible,
		pointerModality: () => {
			keyboardModality = false;
			if (focused()) setFocusVisible(false);
		},
		keyboardModality: () => {
			keyboardModality = true;
			if (focused()) setFocusVisible(true);
		},
		bindings: {
			onFocus: (event) => {
				setFocused(true);
				setFocusVisible(event?.payload?.focusVisible ?? keyboardModality);
			},
			onBlur: () => {
				setFocused(false);
				setFocusVisible(false);
			}
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
//#region src/primitives/hover.ts
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
//#region src/primitives/press.ts
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
//#region src/primitives/view.ts
const ICON_SIZE_UNITLESS_RE = /^-?\d*\.?\d+$/;
function normalizeIconSize(size) {
	if (size == null) return "1em";
	if (typeof size === "number") return size;
	const value = size.trim();
	if (!value) return "1em";
	const parsed = Number.parseFloat(value);
	if (Number.isFinite(parsed) && ICON_SIZE_UNITLESS_RE.test(value)) return parsed;
	return value;
}
function applyIconFill(source, fill) {
	return source.replace(/fill=(["'])none\1/, `fill="${fill}"`);
}
/** @internal Host tags are renderer details, not public JSX elements. */
function createInternalPrimitive(tag, props) {
	const node = createElement(tag);
	spread(node, props, false);
	return node;
}
function primitive(tag, props) {
	return createInternalPrimitive(tag, props);
}
function editorPrimitive(tag, props) {
	return primitive(tag, mergeProps(props, {
		get role() {
			return props.role ?? "textbox";
		},
		get focusOrder() {
			return props.disabled ? -1 : props.focusOrder ?? 0;
		},
		get "aria-disabled"() {
			return props.disabled ?? false;
		}
	}));
}
function semanticPrimitive(tag, role, props) {
	const node = createElement(tag);
	spread(node, { role }, false);
	spread(node, props, false);
	return node;
}
/** A layout container. Text content should be placed in a {@link Text}. */
function View(props) {
	return primitive("view", props);
}
/**
* Stable retained region projected through its own GPUI Entity.
*
* Use this around independently changing route content, scroll viewports,
* overlays, native-widget regions, animation surfaces, or diagnostic HUDs.
* It does not create application state and has the same layout semantics as a
* View; it only limits native invalidation and materialization.
*/
function ProjectionBoundary(props) {
	const node = createElement("view");
	spread(node, props, false);
	spread(node, { projectionBoundary: true }, false);
	return node;
}
function resolvedTextBehavior(maxLines) {
	if (maxLines != null && (!Number.isInteger(maxLines) || maxLines < 1)) throw new RangeError("Text maxLines must be a positive integer");
	return {
		flags: TEXT_BEHAVIOR.AggregateDirectText | (maxLines === 1 ? TEXT_BEHAVIOR.SingleLine : 0),
		maxLines: maxLines ?? 0
	};
}
/**
* A measured text run that wraps within its available width by default.
*
* Static and reactive child text nodes are concatenated by the native host and
* participate in the parent layout as one item. Use `maxLines={1}` or
* `whitespace-nowrap` when the text must remain on one line.
*/
function Text(props) {
	resolvedTextBehavior(untrack(() => props.maxLines));
	const node = createElement("text");
	spread(node, omit(props, "maxLines"), false);
	spread(node, {
		role: props.role ?? "label",
		get textBehavior() {
			return resolvedTextBehavior(props.maxLines);
		}
	}, false);
	return node;
}
/**
* One Parley paragraph assembled from explicitly styled text descendants.
*
* Unlike adjacent Text components, spans share wrapping, whitespace,
* selection, and copy semantics because the native host lays them out once.
*/
function RichText(props) {
	resolvedTextBehavior(untrack(() => props.maxLines));
	const node = createElement("text");
	spread(node, omit(props, "maxLines"), false);
	spread(node, {
		role: props.role ?? "label",
		get textBehavior() {
			const behavior = resolvedTextBehavior(props.maxLines);
			return {
				...behavior,
				flags: behavior.flags | TEXT_BEHAVIOR.AggregateStyledText
			};
		}
	}, false);
	return node;
}
/** A text-style boundary inside RichText; it never creates a layout box. */
function RichTextSpan(props) {
	return primitive("text-span", props);
}
/** A static SVG asset rendered through the native usvg/Vello pipeline. */
function Svg(props) {
	return semanticPrimitive("svg", "img", props);
}
/** A native Vello vector path in local logical-pixel coordinates. */
function Path(props) {
	return primitive("vector-path", props);
}
/** A theme-colored SVG icon with stable native sizing and semantics. */
function Icon(props) {
	const rest = omit(props, "source", "size", "fill", "label", "class");
	const node = createElement("svg");
	spread(node, rest, false);
	spread(node, {
		get class() {
			return props.class ? `self-center shrink-0 ${props.class}` : "self-center shrink-0";
		},
		get style() {
			const iconSize = normalizeIconSize(props.size);
			const layoutSize = typeof iconSize === "number" ? px(iconSize) : iconSize;
			return {
				display: "flex",
				"align-items": "center",
				"justify-content": "center",
				"align-self": "center",
				width: layoutSize,
				height: layoutSize,
				"flex-shrink": 0,
				"line-height": "1",
				"pointer-events": "none",
				...props.style ?? {}
			};
		},
		get width() {
			const iconSize = normalizeIconSize(props.size);
			return typeof iconSize === "number" ? String(iconSize) : void 0;
		},
		get source() {
			return props.fill && props.fill !== "none" ? applyIconFill(props.source, props.fill) : props.source;
		},
		get height() {
			const iconSize = normalizeIconSize(props.size);
			return typeof iconSize === "number" ? String(iconSize) : void 0;
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
	const rest = omit(props, "resource");
	const node = createElement("img");
	spread(node, { role: "img" }, false);
	spread(node, rest, false);
	spread(node, { get resource() {
		return props.resource;
	} }, false);
	return node;
}
/** A native single-line text editor with selection and scrolling. */
function TextInput(props) {
	return editorPrimitive("input", props);
}
/** A native multiline text editor with wrapping, selection, and scrolling. */
function TextArea(props) {
	return editorPrimitive("textarea", props);
}
/** Native password editor whose value remains in a Rust SecretStore. */
function PasswordInput(props) {
	return editorPrimitive("password-input", props);
}
/** General-purpose editor whose document and input lifecycle are owned by GPUI. */
function Editor(props) {
	return editorPrimitive("editor", props);
}
/** Mount an explicitly registered Rust/GPUI widget without web-element semantics. */
function NativeWidget(props) {
	const tag = untrack(() => props.tag.trim());
	if (!tag) throw new TypeError("NativeWidget tag must not be empty");
	const rest = omit(props, "tag", "config");
	const node = createElement(tag);
	spread(node, rest, false);
	spread(node, { get widgetConfig() {
		return props.config;
	} }, false);
	return node;
}
//#endregion
//#region src/primitives/button.tsx
const ACCENTS = {
	neutral: "#475569",
	sky: "#0284c7",
	amber: "#d97706"
};
function InternalButton(props) {
	return createInternalPrimitive("button", props);
}
function resolveButtonFocusOrder(disabled, focusOrder) {
	return disabled ? -1 : focusOrder ?? 0;
}
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
			focusVisible: focus.focusVisible(),
			selected: selected(),
			disabled: disabled()
		}),
		bindings: {
			onPointerEnter: hover.bindings.onPointerEnter,
			onPointerLeave: () => {
				hover.bindings.onPointerLeave();
				press.bindings.onPointerLeave();
			},
			onPointerDown: () => {
				focus.pointerModality();
				press.bindings.onPointerDown();
			},
			onPointerUp: press.bindings.onPointerUp,
			onPointerCancel: press.bindings.onPointerCancel,
			onFocus: focus.bindings.onFocus,
			onBlur: focus.bindings.onBlur,
			onClick: press.bindings.onClick,
			onKeyDown: (event) => {
				focus.keyboardModality();
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
	const forwardedRef = untrack(() => props.ref);
	const refProps = forwardedRef ? { ref: forwardedRef } : {};
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
	const structuralStyle = () => ({
		display: "flex",
		"align-items": "center",
		"flex-shrink": 0,
		"white-space": "nowrap",
		cursor: disabled() ? "not-allowed" : "pointer"
	});
	const defaultStyle = () => {
		if (props.unstyled) return structuralStyle();
		return {
			...structuralStyle(),
			"justify-content": "center",
			"min-height": "32px",
			padding: "6px 12px",
			"border-radius": "6px",
			"border-width": "1px",
			"border-color": state().focusVisible ? "#7dd3fc" : "#64748b",
			"background-color": background(),
			color: "#f8fafc",
			opacity: disabled() ? .45 : 1
		};
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
	return createComponent$1(InternalButton, mergeProps(refProps, {
		get disabled() {
			return disabled();
		},
		get ["aria-disabled"]() {
			return disabled();
		},
		get focusOrder() {
			return resolveButtonFocusOrder(disabled(), props.focusOrder);
		},
		get role() {
			return props.role ?? "button";
		},
		get ["aria-haspopup"]() {
			return props["aria-haspopup"];
		},
		get ["aria-expanded"]() {
			return props["aria-expanded"];
		},
		get ["aria-controls"]() {
			return props["aria-controls"];
		},
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-checked"]() {
			return props["aria-checked"];
		},
		get ["aria-current"]() {
			return props["aria-current"];
		},
		get ["aria-selected"]() {
			return props["aria-selected"];
		},
		get ["aria-pressed"]() {
			return props["aria-pressed"];
		},
		get ["aria-busy"]() {
			return props["aria-busy"];
		},
		get ["aria-valuetext"]() {
			return props["aria-valuetext"];
		},
		get ["class"]() {
			return mergeClasses("select-none", typeof props.class === "function" ? props.class(state()) : props.class);
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
		},
		onPointerEnter: (event) => {
			primitive.bindings.onPointerEnter();
			props.onPointerEnter?.(event);
		},
		onPointerLeave: (event) => {
			primitive.bindings.onPointerLeave();
			props.onPointerLeave?.(event);
		},
		onPointerDown: (event) => {
			primitive.bindings.onPointerDown();
			props.onPointerDown?.(event);
		},
		onPointerUp: (event) => {
			primitive.bindings.onPointerUp();
			props.onPointerUp?.(event);
		},
		onPointerCancel: (event) => {
			primitive.bindings.onPointerCancel();
			props.onPointerCancel?.(event);
		},
		get onPointerMove() {
			return props.onPointerMove;
		},
		get onPointerOver() {
			return props.onPointerOver;
		},
		get onPointerOut() {
			return props.onPointerOut;
		},
		onFocus: (event) => {
			primitive.bindings.onFocus();
			props.onFocus?.(event);
		},
		onBlur: (event) => {
			primitive.bindings.onBlur();
			props.onBlur?.(event);
		},
		get onFocusIn() {
			return props.onFocusIn;
		},
		get onFocusOut() {
			return props.onFocusOut;
		},
		get onClick() {
			return primitive.bindings.onClick;
		},
		get onContextMenu() {
			return props.onContextMenu;
		},
		get onDblClick() {
			return props.onDblClick;
		},
		get onKeyDown() {
			return primitive.bindings.onKeyDown;
		},
		get onKeyUp() {
			return props.onKeyUp;
		},
		get onWheel() {
			return props.onWheel;
		},
		get children() {
			return props.renderContent?.(state()) ?? props.children;
		}
	}));
}
/**
* An explicit external-link interaction.
*
* Wabou does not assign browser behavior to an `a` tag or `href` attribute;
* the JS primitive owns activation while Rust only executes `openUrl`.
*/
function Link(props) {
	const host = useHost$1();
	return createComponent$1(Button, mergeProps(props, {
		role: "link",
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) host.system.openUrl(props.url);
		}
	}));
}
//#endregion
//#region src/primitives/measure.ts
function validateSizeQuery(query) {
	const entries = [
		["minWidth", query.minWidth],
		["maxWidth", query.maxWidth],
		["minHeight", query.minHeight],
		["maxHeight", query.maxHeight]
	];
	for (const [name, value] of entries) if (value !== void 0 && (!Number.isFinite(value) || value < 0)) throw new RangeError(`${name} must be a finite non-negative number`);
	if (query.minWidth !== void 0 && query.maxWidth !== void 0 && query.minWidth > query.maxWidth) throw new RangeError("minWidth cannot exceed maxWidth");
	if (query.minHeight !== void 0 && query.maxHeight !== void 0 && query.minHeight > query.maxHeight) throw new RangeError("minHeight cannot exceed maxHeight");
}
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
/**
* Match constraints against a component's completed native content-box size.
* The result remains false until the first measurement, avoiding a compact
* layout flash during boot.
*/
function createContainerMatch(query, options = {}) {
	validateSizeQuery(query);
	const size = createMeasuredSize(options);
	const matches = () => size.measured() && (query.minWidth === void 0 || size.width() >= query.minWidth) && (query.maxWidth === void 0 || size.width() <= query.maxWidth) && (query.minHeight === void 0 || size.height() >= query.minHeight) && (query.maxHeight === void 0 || size.height() <= query.maxHeight);
	return {
		...size,
		matches
	};
}
//#endregion
//#region src/primitives/presence.ts
/** Explicit mount lifecycle for content whose exit must finish before removal. */
function createPresence(open) {
	const [phase, setPhase] = createSignal(untrack(open) ? "present" : "unmounted");
	createEffect(open, (isOpen) => {
		setPhase((current) => {
			if (isOpen && (current === "unmounted" || current === "exiting")) return "entering";
			if (!isOpen && (current === "present" || current === "entering")) return "exiting";
			return current;
		});
	});
	return {
		phase,
		mounted: () => phase() !== "unmounted",
		finishEnter() {
			if (untrack(open)) setPhase((current) => current === "entering" ? "present" : current);
		},
		finishExit() {
			if (!untrack(open)) setPhase((current) => current === "exiting" ? "unmounted" : current);
		}
	};
}
//#endregion
//#region src/primitives/collapsible-presence.tsx
/**
* Measured disclosure content with explicit presence and subtree isolation.
* Height participates in layout while a subtree opacity layer masks glyphs
* crossing the moving clip edge.
*/
function CollapsiblePresence(props) {
	const inheritedReducedMotion = useReducedMotion();
	const reducedMotion = () => props.reducedMotion ?? inheritedReducedMotion();
	const open = () => props.open;
	const initiallyOpen = untrack(open);
	const presence = createPresence(open);
	let initialMeasurement = true;
	let heightTransition;
	const opacityTransition = createTransition(() => open() ? 1 : 0, {
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion
	});
	const measured = createMeasuredSize({ onChange(size) {
		if (initialMeasurement && initiallyOpen && !props.animateInitial) heightTransition?.jump(size.height);
		initialMeasurement = false;
	} });
	const transitionOptions = () => ({
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion
	});
	heightTransition = createTransition(() => open() && measured.measured() ? measured.height() : 0, {
		...transitionOptions(),
		onComplete(value) {
			const isOpen = untrack(open);
			if (value === 0 && !isOpen) presence.finishExit();
			else if (isOpen) presence.finishEnter();
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
		get interactionBlocked() {
			return !open();
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
							return createComponent$1(View, mergeProps(() => {
								return props.contentProps;
							}, {
								get ["class"]() {
									return props.contentClass;
								},
								get style() {
									return props.contentStyle;
								},
								get children() {
									return props.children;
								}
							}));
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/primitives/image-resource.ts
function call() {
	return bindJsonCapability(useHost().imageResources, {
		name: "imageResources",
		version: 1
	});
}
/** Explicitly create a new resource from a host file. No identity deduplication occurs. */
function createFileImageResource(path) {
	return call()("createFile", { path });
}
/** Explicitly create a new resource from an HTTP(S) response. */
function createNetworkImageResource(url) {
	return call()("createNetwork", { url });
}
/** Deterministically release a resource. Images only borrow their handle. */
function releaseImageResource(handle) {
	return call()("release", handle);
}
/**
* Create a resource owned by the current Solid owner. Source replacement and
* owner cleanup clear the borrowed handle before releasing the native resource.
*/
function createOwnedImageResource(request) {
	const [resource, setResource] = createSignal();
	const [loading, setLoading] = createSignal(false);
	const [error, setError] = createSignal();
	let owned;
	let revision = 0;
	const releaseOwned = () => {
		const previous = owned;
		owned = void 0;
		setResource(void 0);
		if (previous) releaseImageResource(previous.handle);
	};
	createEffect(request, (source) => {
		const currentRevision = ++revision;
		releaseOwned();
		setError(void 0);
		if (!source) {
			setLoading(false);
			return;
		}
		setLoading(true);
		(source.kind === "file" ? createFileImageResource(source.path) : createNetworkImageResource(source.url)).then((next) => {
			if (currentRevision !== revision) {
				releaseImageResource(next.handle);
				return;
			}
			owned = next;
			setResource(next);
			setLoading(false);
		}, (reason) => {
			if (currentRevision !== revision) return;
			setError(reason);
			setLoading(false);
		});
	});
	onCleanup(() => {
		revision++;
		releaseOwned();
	});
	return {
		resource,
		loading,
		error
	};
}
//#endregion
//#region src/primitives/interactions/form-draft.ts
/** Validation key used for errors that do not belong to one field. */
const FORM_ERROR = Symbol("wabou.form-error");
/**
* A small immutable draft for form fields with explicit reset and commit
* semantics. Transient request/error state belongs outside this model.
*/
function createFormDraft(initial, options = {}) {
	const [baselineBox, setBaselineBox] = createSignal({ value: { ...initial } });
	const [box, setBox] = createSignal({ value: { ...initial } });
	const value = () => box().value;
	const equals = options.equals ?? shallowEqual;
	const dirty = createMemo(() => !equals(value(), baselineBox().value));
	const errors = createMemo(() => options.validate?.(value()) ?? {});
	const valid = createMemo(() => Reflect.ownKeys(errors()).length === 0);
	const replace = (next) => setBox({ value: next });
	const resetTo = (next) => {
		setBaselineBox({ value: { ...next } });
		replace({ ...next });
	};
	const setField = (key, updater) => {
		setBox((currentBox) => {
			const current = currentBox.value;
			const previous = current[key];
			const next = typeof updater === "function" ? updater(previous) : updater;
			return Object.is(previous, next) ? currentBox : { value: {
				...current,
				[key]: next
			} };
		});
	};
	return {
		value,
		dirty,
		errors,
		valid,
		formError: () => errors()[FORM_ERROR],
		fieldError: (key) => errors()[key],
		field: (key) => value()[key],
		control: (key) => [() => value()[key], (next) => setField(key, next)],
		set: setField,
		patch: (patch) => {
			setBox((currentBox) => {
				const current = currentBox.value;
				for (const key of Reflect.ownKeys(patch)) if (!Object.is(current[key], patch[key])) return { value: {
					...current,
					...patch
				} };
				return currentBox;
			});
		},
		reset: () => replace({ ...baselineBox().value }),
		resetTo,
		commit: () => {
			setBox((currentBox) => {
				setBaselineBox({ value: { ...currentBox.value } });
				return currentBox;
			});
		}
	};
}
function shallowEqual(left, right) {
	const keys = Reflect.ownKeys(left);
	if (keys.length !== Reflect.ownKeys(right).length) return false;
	return keys.every((key) => Object.is(left[key], right[key]));
}
//#endregion
//#region src/primitives/interactions/selection.ts
function toggleSelection(current, item, mode, allowEmpty = false) {
	return match(mode).with("single", () => current === item && allowEmpty ? void 0 : item).with("multiple", () => {
		const values = Array.isArray(current) ? current : [];
		return values.includes(item) ? values.filter((value) => value !== item) : [...values, item];
	}).exhaustive();
}
function isSelected(selection, item) {
	return Array.isArray(selection) ? selection.includes(item) : selection === item;
}
/**
* Selection state owned by stable keys while values remain host-owned.
* Selected items always resolve to the latest objects from `items`; keys that
* disappear from the source are removed instead of becoming ghost selections.
*/
function createKeyedSelection(options) {
	const initialAvailable = new Set(untrack(options.items).map(options.key));
	const initial = normalizeKeys(options.initialKeys ?? [], initialAvailable, options.mode);
	const [keys, setKeys] = createSignal(initial);
	createEffect(() => new Set(options.items().map(options.key)), (available) => {
		setKeys((current) => {
			const next = normalizeKeys(current, available, options.mode);
			return setsEqual(current, next) ? current : next;
		});
	});
	const selectedItems = createMemo(() => {
		const selected = keys();
		return options.items().filter((item) => selected.has(options.key(item)));
	});
	const set = (nextKeys) => {
		const next = normalizeKeys(nextKeys, new Set(options.items().map(options.key)), options.mode);
		setKeys((current) => setsEqual(current, next) ? current : next);
	};
	const select = (key) => {
		if (options.mode === "single") {
			set([key]);
			return;
		}
		setKeys((current) => {
			if (current.has(key)) return current;
			if (!new Set(options.items().map(options.key)).has(key)) return current;
			return /* @__PURE__ */ new Set([...current, key]);
		});
	};
	const deselect = (key) => {
		setKeys((current) => {
			if (!current.has(key)) return current;
			const next = new Set(current);
			next.delete(key);
			return next;
		});
	};
	return {
		keys,
		items: selectedItems,
		item: () => selectedItems()[0],
		isSelected: (key) => keys().has(key),
		select,
		deselect,
		toggle: (key) => keys().has(key) ? deselect(key) : select(key),
		set,
		clear: () => setKeys((current) => current.size === 0 ? current : /* @__PURE__ */ new Set())
	};
}
function normalizeKeys(keys, available, mode) {
	const next = /* @__PURE__ */ new Set();
	for (const key of keys) {
		if (!available.has(key)) continue;
		next.add(key);
		if (mode === "single") break;
	}
	return next;
}
function setsEqual(left, right) {
	if (left.size !== right.size) return false;
	for (const key of left) if (!right.has(key)) return false;
	return true;
}
//#endregion
//#region src/primitives/layout.tsx
/** Horizontal flex container. No wrapper node is added beyond the host View. */
function Row(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex flex-row", props.class);
	} }));
}
/** Vertical flex container. No wrapper node is added beyond the host View. */
function Column(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex flex-col", props.class);
	} }));
}
/** Flex container that centers children on both axes. */
function Center(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex items-center justify-center", props.class);
	} }));
}
//#endregion
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
	const presence = createPresence(open);
	const visualState = () => modalVisualState(open(), props.backdropFade);
	const [transitionGeneration, setTransitionGeneration] = createSignal(0);
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
	createEffect(() => [open(), reducedMotion()], ([isOpen, prefersReducedMotion]) => {
		setTransitionGeneration((value) => value + 1);
		if (!motionEnabled || prefersReducedMotion || transitionDuration(isOpen) <= 0) {
			if (isOpen) presence.finishEnter();
			else presence.finishExit();
		}
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
	const nativeTransition = (entering, fromTransform, toTransform, fromOpacity, toOpacity) => {
		if (!motionEnabled || reducedMotion()) return void 0;
		const authoredEase = (entering ? motionOptions?.enterEase : motionOptions?.exitEase) ?? motionOptions?.ease;
		const easing = authoredEase === "linear" || authoredEase === "easeInOut" || authoredEase === "easeOut" ? authoredEase : "easeInOut";
		return {
			generation: transitionGeneration(),
			duration: transitionDuration(entering),
			easing,
			fromTransform,
			toTransform,
			fromOpacity,
			toOpacity
		};
	};
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
				"aria-modal": "true",
				get focusContained() {
					return visualState().active;
				},
				get interactionBlocked() {
					return !visualState().active;
				},
				get class() {
					const visual = visualState();
					return mergeClasses(visual.active && "backdrop-blur-sm", visual.retainBackdropVisuals && props.backdropClass);
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
						...visual.transparentBackdrop ? { "background-color": rgba$1(0) } : void 0,
						"pointer-events": visual.active ? "auto" : "none",
						"z-index": layer.zIndex()
					};
				},
				get nativeTransition() {
					if (props.backdropFade === false) return void 0;
					const entering = open();
					return nativeTransition(entering, [
						1,
						0,
						0,
						1,
						0,
						0
					], [
						1,
						0,
						0,
						1,
						0,
						0
					], entering ? 0 : 1, entering ? 1 : 0);
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
							return props.contentStyle;
						},
						get shadows() {
							return props.contentShadows;
						},
						get transform() {
							const base = modalMotionTransform(motionOptions, open() ? 1 : 0);
							return props.contentTransform?.(base, open() ? 1 : 0) ?? base;
						},
						get nativeTransition() {
							const entering = open();
							const fromProgress = entering ? 0 : 1;
							const toProgress = entering ? 1 : 0;
							const from = modalMotionTransform(motionOptions, fromProgress);
							const to = modalMotionTransform(motionOptions, toProgress);
							return nativeTransition(entering, props.contentTransform?.(from, fromProgress) ?? from, props.contentTransform?.(to, toProgress) ?? to, props.contentFade === false ? 1 : fromProgress, props.contentFade === false ? 1 : toProgress);
						},
						onTransitionEnd: (event) => {
							if (event.generation !== transitionGeneration()) return;
							if (open()) presence.finishEnter();
							else presence.finishExit();
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
//#region src/primitives/motion.tsx
function bindPlayback(controls, props, reducedMotion) {
	createEffect(() => props.speed ?? 1, (speed) => {
		controls.speed = speed;
	});
	createEffect(() => (props.paused ?? false) || reducedMotion(), (stopped) => {
		if (stopped) controls.pause();
		else controls.play();
	});
}
/** A single native View whose contents rotate around its border-box center. */
function Spin(props) {
	const motion = props;
	const reducedMotion = useReducedMotion();
	const view = omit(props, "duration", "speed", "paused");
	const rotation = createRotation({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1,
		reducedMotion,
		reducedValue: 0
	});
	bindPlayback(rotation.controls, motion, reducedMotion);
	return createComponent$1(View, mergeProps(view, { get transform() {
		return rotation.transform();
	} }));
}
/** A single native View with a repeating opacity pulse. */
function Pulse(props) {
	const motion = props;
	const reducedMotion = useReducedMotion();
	const view = omit(props, "duration", "speed", "paused", "from", "to", "style");
	const pulse = createPulse({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1,
		from: motion.from,
		to: motion.to,
		reducedMotion,
		reducedValue: motion.to ?? 1
	});
	bindPlayback(pulse.controls, motion, reducedMotion);
	const style = () => ({
		...motion.style ?? {},
		opacity: pulse.value()
	});
	return createComponent$1(View, mergeProps(view, { get style() {
		return style();
	} }));
}
/** A center-originating ring that expands while fading out, then repeats. */
function Ripple(props) {
	const motion = props;
	const reducedMotion = useReducedMotion();
	const view = omit(props, "duration", "speed", "paused", "fromScale", "style", "transform");
	const ripple = createLoop({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1.4,
		from: 0,
		to: 1,
		reducedMotion,
		reducedValue: 1
	});
	bindPlayback(ripple.controls, motion, reducedMotion);
	const progress = () => ripple.value();
	return createComponent$1(View, mergeProps(view, {
		get transform() {
			return scale2d((motion.fromScale ?? .35) + progress() * (1 - (motion.fromScale ?? .35)));
		},
		get style() {
			return {
				...motion.style ?? {},
				opacity: reducedMotion() ? 0 : 1 - progress()
			};
		}
	}));
}
//#endregion
//#region src/primitives/retained-items.ts
const assertUniqueKeys$1 = (items, key) => {
	const keyed = /* @__PURE__ */ new Map();
	for (const item of items) {
		const itemKey = key(item);
		if (keyed.has(itemKey)) throw new Error(`duplicate retained item key: ${String(itemKey)}`);
		keyed.set(itemKey, item);
	}
	return keyed;
};
/**
* Keep keyed values mounted after logical removal until `release` is called.
*
* Entries are stable by key, expose the latest source value, and report
* logical presence independently from visual retention. This is the common
* lifecycle needed by exit animations without delaying state or semantics.
*/
function createRetainedItems(source, key) {
	const [revision, setRevision] = createSignal(0, { ownedWrite: true });
	let active = assertUniqueKeys$1(untrack(source), key);
	const createEntry = (itemKey, item) => {
		const entry = {
			key: itemKey,
			current: item,
			value: () => {
				revision();
				return entry.current;
			},
			present: () => {
				revision();
				return active.has(itemKey);
			}
		};
		return entry;
	};
	const initial = [...active].map(([itemKey, item]) => createEntry(itemKey, item));
	const [entries, setEntries] = createSignal(initial, { ownedWrite: true });
	createEffect(source, (current) => {
		const nextActive = assertUniqueKeys$1(current, key);
		const previous = untrack(entries);
		const previousByKey = new Map(previous.map((entry) => [entry.key, entry]));
		const exiting = previous.filter((entry) => !nextActive.has(entry.key));
		const next = current.map((item) => {
			const itemKey = key(item);
			const entry = previousByKey.get(itemKey) ?? createEntry(itemKey, item);
			entry.current = item;
			return entry;
		});
		active = nextActive;
		setEntries([...exiting, ...next]);
		setRevision((value) => value + 1);
	});
	return {
		entries,
		release(itemKey) {
			if (active.has(itemKey)) return false;
			const previous = untrack(entries);
			const next = previous.filter((entry) => entry.key !== itemKey);
			if (next.length === previous.length) return false;
			setEntries(next);
			setRevision((value) => value + 1);
			return true;
		}
	};
}
//#endregion
//#region src/primitives/notification.ts
const finiteNonNegative = (value, fallback) => Number.isFinite(value) ? Math.max(0, value) : fallback;
/** Create an owner-scoped notification queue with explicit JavaScript timers. */
function createNotifications(options = {}) {
	const records = /* @__PURE__ */ new Map();
	const [revision, setRevision] = createSignal(0, { ownedWrite: true });
	const items = () => {
		revision();
		return [...records.values()].map((record) => record.item);
	};
	const defaultDuration = finiteNonNegative(options.defaultDuration ?? 5e3, 5e3);
	const configuredLimit = options.limit ?? 5;
	const limit = Number.isFinite(configuredLimit) ? Math.max(1, Math.floor(configuredLimit)) : 5;
	let nextId = 1;
	const dismiss = (id, reason = "programmatic") => {
		const record = records.get(id);
		if (!record) return false;
		if (record.timer !== void 0) clearTimeout(record.timer);
		records.delete(id);
		setRevision((current) => current + 1);
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
				const oldest = records.values().next().value?.item;
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
			setRevision((current) => current + 1);
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
function notificationNativeTransition(options) {
	const offset = translate2d(options.fromX ?? 0, options.fromY ?? 0);
	const resting = translate2d(0, 0);
	const easing = options.ease === "linear" || options.ease === "easeInOut" || options.ease === "easeOut" ? options.ease : "easeOut";
	return {
		generation: options.generation,
		duration: options.duration,
		easing,
		fromTransform: options.entering ? offset : resting,
		toTransform: options.entering ? resting : offset,
		fromOpacity: options.entering ? 0 : 1,
		toOpacity: options.entering ? 1 : 0
	};
}
const alignment = (placement) => ({
	"align-items": placement.endsWith("start") ? "flex-start" : placement.endsWith("end") ? "flex-end" : "center",
	"justify-content": placement.startsWith("bottom") ? "flex-end" : "flex-start"
});
const renderNotificationPortal = (props, children) => {
	const stack = createInternalPrimitive("toast-stack", {
		get placement() {
			return props.placement ?? "top-end";
		},
		get class() {
			return props.stackClass ?? "w-96 max-w-full";
		},
		children
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
		children: stack
	});
};
/** Render a non-blocking stack on the native floating overlay plane. */
function NotificationRegion(props) {
	const motion = untrack(() => props.motion);
	if (motion === void 0 || motion === false) {
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
		return renderNotificationPortal(props, items);
	}
	const reducedMotion = useReducedMotion();
	const retained = createRetainedItems(props.notifications.items, (item) => item.id);
	const renderAnimatedItem = (retainedItem) => {
		const logicallyPresent = retainedItem.present;
		const presence = createPresence(logicallyPresent);
		const duration = motion.duration ?? .18;
		const [transitionGeneration, setTransitionGeneration] = createSignal(0);
		createEffect(() => [logicallyPresent(), reducedMotion()], ([isPresent, prefersReducedMotion]) => {
			setTransitionGeneration((generation) => generation + 1);
			if (prefersReducedMotion || duration <= 0) {
				if (isPresent) presence.finishEnter();
				else {
					presence.finishExit();
					retained.release(retainedItem.key);
				}
			}
		});
		return createComponent(View, {
			get role() {
				return retainedItem.value().priority === "assertive" ? "alert" : "status";
			},
			get "aria-label"() {
				return retainedItem.value()["aria-label"];
			},
			get "aria-hidden"() {
				return logicallyPresent() ? void 0 : "true";
			},
			get interactionBlocked() {
				return !logicallyPresent();
			},
			get transform() {
				return translate2d(logicallyPresent() ? 0 : motion.fromX ?? 0, logicallyPresent() ? 0 : motion.fromY ?? 0);
			},
			get nativeTransition() {
				if (reducedMotion() || duration <= 0) return void 0;
				return notificationNativeTransition({
					generation: transitionGeneration(),
					duration,
					ease: motion.ease,
					fromX: motion.fromX,
					fromY: motion.fromY,
					entering: logicallyPresent()
				});
			},
			get class() {
				return `pointer-events-auto ${props.itemClass ?? ""}`;
			},
			get style() {
				return {
					...props.itemStyle,
					opacity: logicallyPresent() ? 1 : 0
				};
			},
			onTransitionEnd: (event) => {
				if (event.generation !== transitionGeneration()) return;
				if (logicallyPresent()) presence.finishEnter();
				else {
					presence.finishExit();
					retained.release(retainedItem.key);
				}
			},
			onPointerEnter: () => props.notifications.pause(retainedItem.key),
			onPointerLeave: () => props.notifications.resume(retainedItem.key),
			onFocusIn: () => props.notifications.pause(retainedItem.key),
			onFocusOut: () => props.notifications.resume(retainedItem.key),
			get children() {
				const item = retainedItem.value();
				return item.content({ dismiss: () => props.notifications.dismiss(item.id, "dismiss") });
			}
		});
	};
	const items = createComponent(For, {
		get each() {
			return retained.entries();
		},
		children: renderAnimatedItem
	});
	return renderNotificationPortal(props, items);
}
//#endregion
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
//#region src/primitives/popover.tsx
function popoverNativeTransition(options) {
	const easing = options.ease === "linear" || options.ease === "easeInOut" || options.ease === "easeOut" ? options.ease : "easeOut";
	return {
		generation: options.generation,
		duration: options.duration,
		easing,
		fromTransform: scale2d(options.entering ? options.fromScale : 1),
		toTransform: scale2d(options.entering ? 1 : options.fromScale),
		fromOpacity: options.entering ? 0 : 1,
		toOpacity: options.entering ? 1 : 0
	};
}
/** A root-layer floating panel positioned from native layout snapshots. */
function Popover(props) {
	const inheritedPlane = useOverlayPlane();
	const reducedMotion = useReducedMotion();
	const plane = () => props.plane ?? inheritedPlane;
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(untrack(() => props.defaultOpen ?? false));
	const open = () => props.open ?? uncontrolledOpen();
	const motion = untrack(() => props.motion);
	const duration = motion === false ? 0 : motion?.duration ?? .14;
	const presence = createPresence(open);
	const [transitionGeneration, setTransitionGeneration] = createSignal(0);
	let anchor;
	let content;
	let suppressPointerClick = false;
	const motionFromScale = () => motion === false ? 1 : motion?.fromScale ?? .98;
	const nativeTransition = () => {
		if (motion === false || reducedMotion()) return void 0;
		return popoverNativeTransition({
			generation: transitionGeneration(),
			duration,
			ease: motion?.ease,
			fromScale: motionFromScale(),
			entering: open()
		});
	};
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
	createEffect(() => [open(), reducedMotion()], ([isOpen, prefersReducedMotion]) => {
		setTransitionGeneration((value) => value + 1);
		if (motion === false || prefersReducedMotion || duration <= 0) {
			if (isOpen) presence.finishEnter();
			else presence.finishExit();
		}
	});
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
						get ["aria-label"]() {
							return props["aria-label"];
						},
						get ["class"]() {
							return props.contentClass;
						},
						get shadows() {
							return props.contentShadows;
						},
						get transform() {
							return scale2d(open() ? 1 : motionFromScale());
						},
						get nativeTransition() {
							return nativeTransition();
						},
						get floatingPosition() {
							return floatingPosition();
						},
						get interactionBlocked() {
							return !open() || props.contentInteractionBlocked;
						},
						get ["aria-hidden"]() {
							return open() ? void 0 : "true";
						},
						get style() {
							return {
								position: "absolute",
								...props.contentStyle,
								opacity: open() ? 1 : 0
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
						onTransitionEnd: (event) => {
							if (event.generation !== transitionGeneration()) return;
							if (open()) presence.finishEnter();
							else presence.finishExit();
						},
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
//#region src/primitives/scroll-area.tsx
const scrollAreaViewportClass = (className) => mergeClasses("min-w-0 min-h-0 overflow-x-hidden overflow-y-auto", className);
/**
* Vertical native scroll viewport with explicit sizing.
*
* The inner wrapper deliberately cannot shrink. This makes its intrinsic
* height become the viewport's scroll extent instead of allowing a flex
* parent to compress overflowing sections until no scroll range remains.
* The viewport also locks its cross axis. Otherwise focus reveal can move a
* nominally vertical viewport sideways when a descendant is slightly wider,
* making split-pane edges appear clipped. It deliberately does not grow:
* implicit `flex-1` makes a
* nested scroll area expand with an ancestor's intrinsic content instead of
* establishing its own scroll range.
*/
function ScrollArea(props) {
	const forwarded = omit(props, "children", "class", "contentClass", "style", "ref", "scrollbar", "onScroll");
	return createComponent$1(View, mergeProps(forwarded, {
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
					return mergeClasses("flex-none flex flex-col min-h-full", props.contentClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	}));
}
//#endregion
//#region src/primitives/scroll-reset.ts
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
//#region src/primitives/shortcuts.ts
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
	const invokeKeyDown = (event) => {
		const key = normalizeKey(event.key);
		const modifiers = event.mods & ALL_MODIFIERS;
		const shortcut = compiled.find((candidate) => candidate.key === key && (candidate.primary ? event.primary && candidate.modifierMasks.includes(modifiers & ~(MODIFIER_BITS.control | MODIFIER_BITS.meta)) : candidate.modifierMasks.includes(modifiers)));
		if (shortcut === void 0 || event.repeat && !shortcut.allowRepeat) return { handled: false };
		if (shortcut.preventDefault !== false) event.preventDefault();
		return {
			handled: true,
			result: shortcut.handler(event)
		};
	};
	return {
		handleKeyDown: (event) => invokeKeyDown(event).handled,
		bindings: { onKeyDown: (event) => invokeKeyDown(event).result }
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
//#region src/primitives/tabs.ts
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
//#region src/primitives/transition-presence.ts
/**
* Couples logical presence to an interruptible visual transition.
*
* Closing disables the logical surface immediately while keeping its visual
* subtree mounted until progress reaches zero. Reopening during exit simply
* retargets the current transition instead of remounting the subtree.
*/
function createTransitionPresence(open, options = {}) {
	const presence = createPresence(open);
	const visuallyPresent = () => open() && (options.ready?.() ?? true);
	const transition = createTransition(() => visuallyPresent() ? 1 : 0, {
		initial: options.initialProgress,
		duration: options.duration ?? .16,
		ease: options.ease ?? "easeOut",
		reducedMotion: options.reducedMotion,
		onComplete(value) {
			if (value === 1 && untrack(open)) presence.finishEnter();
			else if (value === 0 && !untrack(open)) presence.finishExit();
		}
	});
	createEffect(() => [
		open(),
		visuallyPresent(),
		transition.value(),
		presence.phase()
	], ([isOpen, isVisible, progress, phase]) => {
		if (isOpen && isVisible && progress === 1 && phase === "entering") presence.finishEnter();
		else if (!isOpen && progress === 0 && phase === "exiting") presence.finishExit();
	});
	return {
		phase: presence.phase,
		mounted: presence.mounted,
		progress: transition.value,
		transition
	};
}
//#endregion
//#region src/primitives/index.ts
var primitives_exports = /* @__PURE__ */ __exportAll({
	Button: () => Button,
	Center: () => Center,
	CollapsiblePresence: () => CollapsiblePresence,
	Column: () => Column,
	Editor: () => Editor,
	FORM_ERROR: () => FORM_ERROR,
	Icon: () => Icon,
	Image: () => Image,
	Link: () => Link,
	Modal: () => Modal,
	NativeWidget: () => NativeWidget,
	NotificationRegion: () => NotificationRegion,
	OverlayPlaneProvider: () => OverlayPlaneProvider,
	PasswordInput: () => PasswordInput,
	Path: () => Path,
	PathBuilder: () => PathBuilder,
	Popover: () => Popover,
	ProjectionBoundary: () => ProjectionBoundary,
	Pulse: () => Pulse,
	RichText: () => RichText,
	RichTextSpan: () => RichTextSpan,
	Ripple: () => Ripple,
	Row: () => Row,
	ScrollArea: () => ScrollArea,
	Spin: () => Spin,
	Svg: () => Svg,
	Text: () => Text,
	TextArea: () => TextArea,
	TextInput: () => TextInput,
	View: () => View,
	createActive: () => createActive,
	createAnimationFrame: () => createAnimationFrame,
	createButton: () => createButton,
	createContainerMatch: () => createContainerMatch,
	createFileImageResource: () => createFileImageResource,
	createFocus: () => createFocus,
	createFocusWithin: () => createFocusWithin,
	createFormDraft: () => createFormDraft,
	createHover: () => createHover,
	createKeyedSelection: () => createKeyedSelection,
	createMeasuredSize: () => createMeasuredSize,
	createNetworkImageResource: () => createNetworkImageResource,
	createNotifications: () => createNotifications,
	createOverlayLayer: () => createOverlayLayer,
	createOwnedImageResource: () => createOwnedImageResource,
	createPresence: () => createPresence,
	createPress: () => createPress,
	createRetainedItems: () => createRetainedItems,
	createScrollReset: () => createScrollReset,
	createShortcuts: () => createShortcuts,
	createTabs: () => createTabs,
	createTransitionPresence: () => createTransitionPresence,
	floatingFromNode: () => floatingFromNode,
	floatingFromPoint: () => floatingFromPoint,
	releaseImageResource: () => releaseImageResource,
	rotate2d: () => rotate2d$1,
	translate2d: () => translate2d$1,
	useOverlayPlane: () => useOverlayPlane
});
//#endregion
export { View as $, createOwnedImageResource as A, Icon as B, createKeyedSelection as C, createFormDraft as D, FORM_ERROR as E, createMeasuredSize as F, PathBuilder as G, NativeWidget as H, Button as I, RichTextSpan as J, ProjectionBoundary as K, Link as L, CollapsiblePresence as M, createPresence as N, createFileImageResource as O, createContainerMatch as P, TextInput as Q, createButton as R, Row as S, toggleSelection as T, PasswordInput as U, Image as V, Path as W, Text as X, Svg as Y, TextArea as Z, OverlayPlaneProvider as _, createTransition as _t, createScrollReset as a, createFocus as at, Center as b, useMotionConfig as bt, floatingFromNode as c, animate as ct, createNotifications as d, createKeyframeAnimation as dt, rotate2d$1 as et, createRetainedItems as f, createLoop as ft, Modal as g, createSweep as gt, Spin as h, createRotation as ht, createShortcuts as i, createHover as it, releaseImageResource as j, createNetworkImageResource as k, floatingFromPoint as l, animateKeyframes as lt, Ripple as m, createPulse as mt, createTransitionPresence as n, createActive as nt, ScrollArea as o, createFocusWithin as ot, Pulse as p, createNativeLoopAnimation as pt, RichText as q, createTabs as r, createPress as rt, Popover as s, createAnimationFrame as st, primitives_exports as t, translate2d$1 as tt, NotificationRegion as u, createInterpolation as ut, createOverlayLayer as v, normalizeSweepGeometry as vt, isSelected as w, Column as x, useReducedMotion as xt, useOverlayPlane as y, MotionConfigProvider as yt, Editor as z };

//# sourceMappingURL=primitives-BG5zyPXK.mjs.map