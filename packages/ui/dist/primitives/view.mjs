import { CodeEditorDocument } from "./code-editor-state.mjs";
import { PathBuilder } from "@wabou/core";
import { px, rotate2d as rotate2d$1, translate2d as translate2d$1 } from "@wabou/core/style";
import { createEffect, createMemo, createSignal, omit, untrack } from "solid-js";
import { TEXT_BEHAVIOR, createElement, mergeProps, spread } from "@wabou/core/renderer";
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
function editorLanguage(language) {
	return language === "json" || language === "diff" ? language : void 0;
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
	spread(node, { role }, true);
	spread(node, props, false);
	return node;
}
/** A layout container. Text content should be placed in a {@link Text}. */
function View(props) {
	return primitive("view", props);
}
/**
* Stable retained region with an independent native invalidation boundary.
*
* Use this around independently changing route content, scroll viewports,
* overlays, native-widget regions, animation surfaces, or diagnostic HUDs.
* It does not create application state and has the same layout semantics as a
* View; it only limits native invalidation and materialization.
*/
function ProjectionBoundary(props) {
	const node = createElement("view");
	spread(node, props, false);
	spread(node, { projectionBoundary: true }, true);
	return node;
}
function resolvedTextBehavior(maxLines, lineOverflow) {
	if (maxLines != null && (!Number.isInteger(maxLines) || maxLines < 1)) throw new RangeError("Text maxLines must be a positive integer");
	return {
		flags: TEXT_BEHAVIOR.AggregateDirectText | (maxLines === 1 ? TEXT_BEHAVIOR.SingleLine : 0) | (lineOverflow === "clip" ? TEXT_BEHAVIOR.ClipLines : 0),
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
	resolvedTextBehavior(untrack(() => props.maxLines), untrack(() => props.lineOverflow));
	const node = createElement("text");
	spread(node, omit(props, "maxLines", "lineOverflow"), false);
	spread(node, {
		role: props.role ?? "label",
		get textBehavior() {
			return resolvedTextBehavior(props.maxLines, props.lineOverflow);
		}
	}, true);
	return node;
}
/**
* One Parley paragraph assembled from explicitly styled text descendants.
*
* Unlike adjacent Text components, spans share wrapping, whitespace,
* selection, and copy semantics because the native host lays them out once.
*/
function RichText(props) {
	resolvedTextBehavior(untrack(() => props.maxLines), untrack(() => props.lineOverflow));
	const node = createElement("text");
	spread(node, omit(props, "maxLines", "lineOverflow"), false);
	spread(node, {
		role: props.role ?? "label",
		get textBehavior() {
			const behavior = resolvedTextBehavior(props.maxLines, props.lineOverflow);
			return {
				...behavior,
				flags: behavior.flags | TEXT_BEHAVIOR.AggregateStyledText
			};
		}
	}, true);
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
	}, true);
	return node;
}
/** A replaced image node rendered by the native host. */
function Image(props) {
	const rest = omit(props, "resource");
	const node = createElement("img");
	spread(node, { role: "img" }, true);
	spread(node, rest, false);
	spread(node, { get resource() {
		return props.resource;
	} }, true);
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
/**
* General-purpose native editor viewport backed by a DOM-free CodeMirror
* document. CodeMirror owns transactions, selection and history; Rust owns
* painting, pointer hit testing, scrolling and the platform IME bridge.
*/
function Editor(props) {
	const initialValue = untrack(() => props.value ?? "");
	const initialLanguage = untrack(() => editorLanguage(props.language));
	const document = new CodeEditorDocument(initialValue, initialLanguage);
	const [revision, setRevision] = createSignal(0);
	const invalidate = () => setRevision((value) => value + 1);
	const emitInput = () => props.onInput?.({ currentTarget: { value: document.value } });
	createEffect(() => props.value, (controlledValue) => {
		if (controlledValue !== void 0 && controlledValue !== document.value) {
			document.sync(controlledValue, editorLanguage(props.language));
			invalidate();
		}
	});
	const widgetConfig = createMemo(() => {
		revision();
		return document.config(editorLanguage(props.language));
	});
	const nativeProps = omit(props, "language", "onInput", "onKeyDown", "onImePreedit", "onImeCommit", "onImeDeleteSurrounding", "onImeDisabled", "onTextSelectionChange");
	return editorPrimitive("editor", mergeProps(nativeProps, {
		get value() {
			revision();
			return document.value;
		},
		get widgetConfig() {
			return widgetConfig();
		},
		onKeyDown(event) {
			if (!props.disabled) {
				const result = document.handleKey({
					key: event.key,
					shift: (event.mods & 1) !== 0,
					primary: event.primary,
					readOnly: props.readOnly
				});
				if (result.handled) {
					event.preventDefault();
					invalidate();
					if (result.changed && !props.readOnly) emitInput();
				}
			}
			props.onKeyDown?.(event);
		},
		onInput(event) {
			if (!props.disabled && !props.readOnly) {
				document.sync(event.currentTarget.value, editorLanguage(props.language));
				invalidate();
			}
			props.onInput?.(event);
		},
		onImePreedit(event) {
			if (!props.disabled && !props.readOnly) {
				document.setComposition(event.data, event.cursorStart, event.cursorEnd);
				event.preventDefault();
				invalidate();
			}
			props.onImePreedit?.(event);
		},
		onImeCommit(event) {
			if (!props.disabled && !props.readOnly) {
				const changed = document.commitText(event.data);
				event.preventDefault();
				invalidate();
				if (changed) emitInput();
			}
			props.onImeCommit?.(event);
		},
		onImeDeleteSurrounding(event) {
			if (!props.disabled && !props.readOnly) {
				const changed = document.deleteSurrounding(event.beforeBytes, event.afterBytes);
				event.preventDefault();
				invalidate();
				if (changed) emitInput();
			}
			props.onImeDeleteSurrounding?.(event);
		},
		onImeDisabled(event) {
			if (document.setComposition("", null, null)) invalidate();
			props.onImeDisabled?.(event);
		},
		onTextSelectionChange(event) {
			if (event.anchor !== void 0 && event.head !== void 0 && document.setSelection(event.anchor, event.head)) invalidate();
			props.onTextSelectionChange?.(event);
		}
	}));
}
/** Mount an explicitly registered Rust widget without web-element semantics. */
function NativeWidget(props) {
	const tag = untrack(() => props.tag.trim());
	if (!tag) throw new TypeError("NativeWidget tag must not be empty");
	const rest = omit(props, "tag", "config");
	const node = createElement(tag);
	spread(node, rest, false);
	spread(node, { get widgetConfig() {
		return props.config;
	} }, true);
	return node;
}
//#endregion
export { Editor, Icon, Image, NativeWidget, PasswordInput, Path, PathBuilder, ProjectionBoundary, RichText, RichTextSpan, Svg, Text, TextArea, TextInput, View, createInternalPrimitive, rotate2d$1 as rotate2d, translate2d$1 as translate2d };

//# sourceMappingURL=view.mjs.map