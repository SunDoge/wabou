import { _ as isNodeKey, a as GRAPHIC_SOURCE, f as Writer, g as formatNodeKey, h as ROOT_NODE_KEY, i as GRAPHIC_DATA, l as INTERACTION_POLICY, m as NodeKeyTable, p as NodeKeyAllocator, t as EVENT_CODE, y as nodeKeyEquals } from "./protocol-Dkalv0Si.mjs";
import { r as assertInlineStyleValue, s as isTypedStyleValue } from "./style-03o9fSQN.mjs";
import { For, Show, createComponent, createContext, createMemo, createSignal, getOwner, omit, onCleanup, untrack, useContext } from "solid-js";
import { createRenderer } from "@solidjs/universal";
import { Virtualizer } from "@tanstack/virtual-core";
//#region src/vector-path.ts
/** Stable, renderer-independent vector path command stream. */
const PATH_MAGIC = 827343447;
const PATH_VERSION = 1;
const HEADER_SIZE = 36;
const MAX_PATH_BYTES = 16777216;
const COMMAND = {
	MoveTo: 1,
	LineTo: 2,
	QuadTo: 3,
	CubicTo: 4,
	Close: 5
};
function finite(name, values) {
	if (!values.every(Number.isFinite)) throw new RangeError(`${name} requires finite coordinates`);
}
function rgba(value, fallback) {
	if (value === void 0) return fallback;
	if (!Number.isInteger(value) || value < 0 || value > 4294967295) throw new RangeError("path colors must be packed 32-bit RGBA values");
	return value >>> 0;
}
function positive(name, value) {
	if (!Number.isFinite(value) || value <= 0) throw new RangeError(`${name} must be a positive finite number`);
	return value;
}
var PathBuilder = class {
	#commands = [];
	#hasSubpath = false;
	#drawable = false;
	/** Whether line/curve/close commands currently have an active subpath. */
	get hasCurrentPoint() {
		return this.#hasSubpath;
	}
	moveTo(x, y) {
		finite("moveTo", [x, y]);
		this.#commands.push([
			COMMAND.MoveTo,
			x,
			y
		]);
		this.#hasSubpath = true;
		return this;
	}
	lineTo(x, y) {
		if (!this.#hasSubpath) throw new TypeError("lineTo requires moveTo first");
		finite("lineTo", [x, y]);
		this.#commands.push([
			COMMAND.LineTo,
			x,
			y
		]);
		this.#drawable = true;
		return this;
	}
	quadTo(cx, cy, x, y) {
		if (!this.#hasSubpath) throw new TypeError("quadTo requires moveTo first");
		finite("quadTo", [
			cx,
			cy,
			x,
			y
		]);
		this.#commands.push([
			COMMAND.QuadTo,
			cx,
			cy,
			x,
			y
		]);
		this.#drawable = true;
		return this;
	}
	cubicTo(c1x, c1y, c2x, c2y, x, y) {
		if (!this.#hasSubpath) throw new TypeError("cubicTo requires moveTo first");
		finite("cubicTo", [
			c1x,
			c1y,
			c2x,
			c2y,
			x,
			y
		]);
		this.#commands.push([
			COMMAND.CubicTo,
			c1x,
			c1y,
			c2x,
			c2y,
			x,
			y
		]);
		this.#drawable = true;
		return this;
	}
	close() {
		if (!this.#hasSubpath) throw new TypeError("close requires moveTo first");
		this.#commands.push([COMMAND.Close]);
		this.#hasSubpath = false;
		return this;
	}
	/** Append a Catmull–Rom spline converted to native cubic Bézier segments. */
	splineThrough(points, tension = 1) {
		finite("splineThrough tension", [tension]);
		if (points.length === 0) return this;
		this.moveTo(points[0].x, points[0].y);
		if (points.length === 1) return this;
		const scale = tension / 6;
		for (let index = 0; index < points.length - 1; index++) {
			const before = points[Math.max(0, index - 1)];
			const start = points[index];
			const end = points[index + 1];
			const after = points[Math.min(points.length - 1, index + 2)];
			this.cubicTo(start.x + (end.x - before.x) * scale, start.y + (end.y - before.y) * scale, end.x - (after.x - start.x) * scale, end.y - (after.y - start.y) * scale, end.x, end.y);
		}
		return this;
	}
	/** Create an immutable snapshot. Later builder mutations cannot alter it. */
	build(paint = {}) {
		const resolved = Object.freeze({
			fill: rgba(paint.fill, 0),
			stroke: rgba(paint.stroke, 0),
			strokeWidth: positive("strokeWidth", paint.strokeWidth ?? 1),
			fillRule: paint.fillRule ?? "nonzero",
			lineCap: paint.lineCap ?? "butt",
			lineJoin: paint.lineJoin ?? "miter",
			miterLimit: positive("miterLimit", paint.miterLimit ?? 4)
		});
		const byteLength = HEADER_SIZE + this.#commands.reduce((size, command) => size + 4 + (command.length - 1) * 4, 0);
		if (byteLength > MAX_PATH_BYTES) throw new RangeError("vector path exceeds the 16 MiB protocol limit");
		const data = new Uint8Array(byteLength);
		const view = new DataView(data.buffer);
		view.setUint32(0, PATH_MAGIC, true);
		view.setUint16(4, PATH_VERSION, true);
		view.setUint16(6, 0, true);
		view.setUint32(8, this.#commands.length, true);
		view.setUint32(12, byteLength, true);
		view.setUint32(16, resolved.fill, true);
		view.setUint32(20, resolved.stroke, true);
		view.setFloat32(24, resolved.strokeWidth, true);
		view.setUint8(28, resolved.fillRule === "evenodd" ? 1 : 0);
		view.setUint8(29, resolved.lineCap === "round" ? 1 : resolved.lineCap === "square" ? 2 : 0);
		view.setUint8(30, resolved.lineJoin === "round" ? 1 : resolved.lineJoin === "bevel" ? 2 : 0);
		view.setUint8(31, 0);
		view.setFloat32(32, resolved.miterLimit, true);
		let offset = HEADER_SIZE;
		for (const command of this.#commands) {
			view.setUint8(offset, command[0]);
			offset += 4;
			for (let index = 1; index < command.length; index++) {
				view.setFloat32(offset, command[index], true);
				offset += 4;
			}
		}
		return Object.freeze({
			kind: "wabou-vector-path",
			drawable: this.#drawable,
			get data() {
				return data.slice();
			}
		});
	}
};
function isVectorPath(value) {
	return typeof value === "object" && value !== null && value.kind === "wabou-vector-path" && typeof value.drawable === "boolean" && value.data instanceof Uint8Array;
}
//#endregion
//#region src/renderer/host.tsx
/** Checked adapter around the private Rust/QuickJS ABI. */
const LAYOUT_SNAPSHOT_VERSION = 1;
const LAYOUT_SNAPSHOT_HEADER_LENGTH = 8;
const LAYOUT_SNAPSHOT_NODE_LENGTH = 14;
let layoutSnapshotBuffer = /* @__PURE__ */ new Float64Array(0);
function readRect(values, offset) {
	return {
		x: values[offset],
		y: values[offset + 1],
		width: values[offset + 2],
		height: values[offset + 3]
	};
}
function readLayoutSnapshot(ids) {
	const packedIds = new Uint32Array(ids.length * 2);
	for (let index = 0; index < ids.length; index++) {
		packedIds[index * 2] = ids[index].lo;
		packedIds[index * 2 + 1] = ids[index].hi;
	}
	let required = __wabou_layout_snapshot(packedIds, layoutSnapshotBuffer);
	if (layoutSnapshotBuffer.length < required) {
		layoutSnapshotBuffer = new Float64Array(required);
		required = __wabou_layout_snapshot(packedIds, layoutSnapshotBuffer);
	}
	if (required < LAYOUT_SNAPSHOT_HEADER_LENGTH || layoutSnapshotBuffer[0] !== LAYOUT_SNAPSHOT_VERSION) throw new Error("unsupported native layout snapshot format");
	const nodeCount = layoutSnapshotBuffer[7];
	if (!Number.isInteger(nodeCount) || required !== LAYOUT_SNAPSHOT_HEADER_LENGTH + nodeCount * LAYOUT_SNAPSHOT_NODE_LENGTH) throw new Error("invalid native layout snapshot length");
	const nodes = [];
	for (let index = 0; index < nodeCount; index++) {
		const offset = LAYOUT_SNAPSHOT_HEADER_LENGTH + index * LAYOUT_SNAPSHOT_NODE_LENGTH;
		nodes.push({
			id: {
				lo: layoutSnapshotBuffer[offset],
				hi: layoutSnapshotBuffer[offset + 1]
			},
			rect: readRect(layoutSnapshotBuffer, offset + 2),
			clip: readRect(layoutSnapshotBuffer, offset + 6),
			scroll: {
				offsetX: layoutSnapshotBuffer[offset + 10],
				offsetY: layoutSnapshotBuffer[offset + 11],
				rangeX: layoutSnapshotBuffer[offset + 12],
				rangeY: layoutSnapshotBuffer[offset + 13]
			}
		});
	}
	return {
		revision: layoutSnapshotBuffer[1] + layoutSnapshotBuffer[2] * 4294967296,
		viewport: readRect(layoutSnapshotBuffer, 3),
		nodes
	};
}
const nativeHost = {
	openUrl: (url) => __wabou_open_url(url),
	loadFont: (path) => __wabou_load_font(path),
	frameStats: () => JSON.parse(__wabou_frame_stats()),
	setDebugOverlay: (layout, clips, hitTarget) => __wabou_set_debug_overlay(layout, clips, hitTarget),
	debugOverlayPaintStats: () => JSON.parse(__wabou_debug_overlay_paint_stats()),
	layoutSnapshot: readLayoutSnapshot,
	systemLocale: () => __wabou_system_locale(),
	systemTimeZone: () => __wabou_system_time_zone(),
	systemCalendarDate: () => JSON.parse(__wabou_system_calendar_date())
};
const builtinHost = {
	system: { openUrl: nativeHost.openUrl },
	fonts: { load: nativeHost.loadFont },
	diagnostics: {
		frameStats: nativeHost.frameStats,
		overlayPaintStats: nativeHost.debugOverlayPaintStats,
		setOverlay: (options) => nativeHost.setDebugOverlay(options.layout ?? false, options.clips ?? false, options.hitTarget ?? false)
	},
	intl: {
		locale: nativeHost.systemLocale,
		timeZone: nativeHost.systemTimeZone,
		today: nativeHost.systemCalendarDate
	},
	layout: {
		snapshot: (targets) => nativeHost.layoutSnapshot(targets.map((target) => isNodeKey(target) ? target : target.id)),
		measure: (target) => {
			return builtinHost.layout.snapshot([target]).nodes[0]?.rect ?? null;
		},
		clippingRect: (target) => {
			return builtinHost.layout.snapshot([target]).nodes[0]?.clip ?? null;
		},
		viewport: () => builtinHost.layout.snapshot([]).viewport
	}
};
const defaultHost = Object.assign(builtinHost, typeof __wabou_capabilities === "undefined" ? {} : __wabou_capabilities);
const HostContext = createContext(defaultHost);
/** Bind host capabilities to a Solid subtree (normally one window). */
function HostProvider(props) {
	return createComponent(HostContext, {
		get value() {
			return props.value;
		},
		get children() {
			return props.children;
		}
	});
}
/** Return the host associated with the current Solid owner/window. */
function useHost() {
	return getOwner() ? useContext(HostContext) : defaultHost;
}
//#endregion
//#region src/renderer/portal.ts
/** Render a native host subtree under its shared synthetic overlay root. */
function Portal(props) {
	const local = props;
	const containerProps = omit(props, "children", "plane");
	const plane = local.plane ?? "floating";
	const root = acquireOverlayRoot(plane);
	const container = createElement("view");
	spread(container, containerProps, false);
	spread(container, { overlayPlane: plane }, false);
	insertNode(root, container, void 0);
	insert(container, () => local.children);
	onCleanup(() => {
		if (container.parent) removeNode(container.parent, container);
		releaseOverlayRoot(plane);
	});
	return null;
}
//#endregion
//#region src/renderer/use-fps.ts
/**
* Track frames-per-second. A self-perpetuating rAF loop counts frames; a
* 1s interval samples the count and resets it. The rAF loop keeps the host
* redrawing (it drives `has_anim`), so this measures the active vsync rate
* while mounted — ~60 on a 60Hz display, ~120 on 120Hz. When nothing animates,
* the host stops redrawing and the count drops.
*
* ```tsx
* const fps = createFps();
* <Text>{`${fps()} fps`}</Text>
* ```
*/
function createFps() {
	const [fps, setFps] = createSignal(0);
	let frames = 0;
	let last = performance.now();
	let rafId = 0;
	const loop = () => {
		frames++;
		rafId = requestAnimationFrame(loop);
	};
	rafId = requestAnimationFrame(loop);
	const intervalId = setInterval(() => {
		const now = performance.now();
		const dt = now - last;
		last = now;
		if (dt > 0) setFps(Math.round(frames * 1e3 / dt));
		frames = 0;
	}, 1e3);
	onCleanup(() => {
		cancelAnimationFrame(rafId);
		clearInterval(intervalId);
	});
	return fps;
}
//#endregion
//#region src/renderer/virtual-list.tsx
function createVirtualRow(items, index) {
	return createMemo(() => items()[index()]);
}
const encodedItemKey = (key) => typeof key === "number" ? `number:${key}` : `string:${key}`;
function validateVirtualItemKeys(items, getItemKey) {
	const keys = new Array(items.length);
	const seen = /* @__PURE__ */ new Set();
	for (let index = 0; index < items.length; index++) {
		const item = items[index];
		if (item === void 0) throw new TypeError(`VirtualList item at index ${index} is undefined`);
		const key = getItemKey(item, index);
		if (typeof key === "number" && !Number.isFinite(key)) throw new TypeError(`VirtualList key at index ${index} must be finite`);
		const encoded = encodedItemKey(key);
		if (seen.has(encoded)) throw new TypeError(`VirtualList key ${JSON.stringify(key)} is duplicated at index ${index}`);
		seen.add(encoded);
		keys[index] = key;
	}
	return keys;
}
function createVirtualItemIdentity(items, index, getItemKey) {
	return createMemo(() => {
		const currentIndex = index();
		const item = items()[currentIndex];
		return item === void 0 ? void 0 : { key: encodedItemKey(getItemKey(item, currentIndex)) };
	}, { equals: (previous, next) => previous?.key === next?.key });
}
/**
* Windowed Solid list backed by TanStack Virtual's framework-neutral core.
* Rust remains authoritative for scrolling, clipping, hit testing and the
* native scrollbar; this adapter supplies viewport/offset observations instead
* of relying on HTMLElement, ResizeObserver or getBoundingClientRect().
*/
function VirtualList(props) {
	const config = untrack(() => ({
		items: props.items,
		children: props.children,
		itemHeight: props.itemHeight,
		viewportHeight: props.viewportHeight,
		class: props.class,
		overscan: props.overscan,
		getItemKey: props.getItemKey,
		role: props.role,
		accessibilityLabel: props.accessibilityLabel
	}));
	const surface = {};
	let scrollHandle;
	let publishOffset;
	let publishRect;
	let resizeObserver;
	let scrollEndTimer;
	let lastOffset = 0;
	const [version, invalidate] = createSignal(0, { equals: false });
	let currentMeasuredRect = {
		width: 0,
		height: 0
	};
	const viewportHeight = () => config.viewportHeight ?? currentMeasuredRect.height;
	let currentItemKeys = validateVirtualItemKeys(untrack(config.items), config.getItemKey);
	const itemKeys = createMemo(() => {
		currentItemKeys = validateVirtualItemKeys(config.items(), config.getItemKey);
		return currentItemKeys;
	});
	const options = () => ({
		count: currentItemKeys.length,
		getItemKey: (index) => currentItemKeys[index] ?? index,
		getScrollElement: () => scrollHandle ? surface : null,
		estimateSize: () => config.itemHeight,
		overscan: config.overscan ?? 4,
		initialRect: {
			width: currentMeasuredRect.width,
			height: viewportHeight()
		},
		observeElementRect: (_instance, notify) => {
			publishRect = notify;
			notify({
				width: currentMeasuredRect.width,
				height: viewportHeight()
			});
			return () => {
				publishRect = void 0;
			};
		},
		observeElementOffset: (_instance, notify) => {
			publishOffset = notify;
			notify(0, false);
			return () => {
				publishOffset = void 0;
			};
		},
		scrollToFn: (offset) => scrollHandle?.scrollTo({ top: offset }),
		onChange: () => invalidate((value) => value + 1)
	});
	const virtualizer = new Virtualizer(untrack(options));
	const dispose = untrack(() => virtualizer._didMount());
	onCleanup(() => {
		if (scrollEndTimer !== void 0) clearTimeout(scrollEndTimer);
		resizeObserver?.disconnect();
		dispose();
	});
	const virtualItems = createMemo(() => {
		version();
		itemKeys();
		virtualizer.setOptions(options());
		virtualizer._willUpdate();
		return virtualizer.getVirtualItems();
	});
	const totalSize = createMemo(() => {
		virtualItems();
		return virtualizer.getTotalSize();
	});
	var _el$ = createElement("view", { onScroll: (event) => {
		lastOffset = event.scrollY ?? 0;
		publishOffset?.(lastOffset, true);
		if (scrollEndTimer !== void 0) clearTimeout(scrollEndTimer);
		scrollEndTimer = setTimeout(() => {
			scrollEndTimer = void 0;
			publishOffset?.(lastOffset, false);
		}, 150);
	} });
	var _el$2 = createElement("view");
	insertNode(_el$, _el$2);
	ref(() => {
		return (node) => {
			scrollHandle = node;
			if (config.viewportHeight === void 0) {
				resizeObserver?.disconnect();
				resizeObserver = new ResizeObserver(([entry]) => {
					if (!entry) return;
					const rect = {
						width: entry.contentRect.width,
						height: entry.contentRect.height
					};
					currentMeasuredRect = rect;
					publishRect?.(rect);
				});
				resizeObserver.observe(node);
			}
			untrack(() => virtualizer._willUpdate());
		};
	}, _el$);
	insert(_el$2, createComponent$1(For, {
		get each() {
			return virtualItems();
		},
		keyed: false,
		children: (virtualItem) => {
			const index = () => virtualItem().index;
			const item = createVirtualRow(config.items, index);
			const identity = createVirtualItemIdentity(config.items, index, (_item, currentIndex) => itemKeys()[currentIndex] ?? currentIndex);
			var _el$3 = createElement("view");
			insert(_el$3, createComponent$1(Show, {
				get when() {
					return identity();
				},
				keyed: true,
				children: (_identity) => config.children(() => {
					const current = item();
					if (current === void 0) throw new Error("VirtualList item disappeared while its row was mounted");
					return current;
				}, index)
			}));
			effect(() => ({
				position: "absolute",
				top: `${virtualItem().start}px`,
				height: `${virtualItem().size}px`,
				width: "100%"
			}), (_v$, _$p) => {
				setProp(_el$3, "style", _v$, _$p);
			});
			return _el$3;
		}
	}));
	effect(() => {
		return {
			e: config.class,
			t: config.role,
			a: config.accessibilityLabel,
			o: {
				overflow: "scroll",
				position: "relative",
				...config.viewportHeight === void 0 ? {} : { height: `${config.viewportHeight}px` },
				width: "100%"
			},
			i: {
				position: "relative",
				height: `${totalSize()}px`,
				width: "100%"
			}
		};
	}, ({ e, t, a, o, i }, _p$) => {
		e !== _p$?.e && setProp(_el$, "class", e, _p$?.e);
		t !== _p$?.t && setProp(_el$, "role", t, _p$?.t);
		a !== _p$?.a && setProp(_el$, "aria-label", a, _p$?.a);
		o !== _p$?.o && setProp(_el$, "style", o, _p$?.o);
		i !== _p$?.i && setProp(_el$2, "style", i, _p$?.i);
	});
	return _el$;
}
//#endregion
//#region src/renderer/index.ts
const isServer = false;
const getRequestEvent = () => void 0;
const delegateEvents = () => {};
/** Whether a bubbled handler is running on the node originally hit. */
function isDirectEvent(event) {
	return nodeKeyEquals(event.target.id, event.currentTarget.id);
}
const globalPointerListeners = /* @__PURE__ */ new Map();
/** Observe native pointer dispatch before ordinary bubbling. */
function observeGlobalPointerEvent(type, listener) {
	const listeners = globalPointerListeners.get(type) ?? /* @__PURE__ */ new Set();
	listeners.add(listener);
	globalPointerListeners.set(type, listeners);
	return () => {
		listeners.delete(listener);
		if (listeners.size === 0) globalPointerListeners.delete(type);
	};
}
const nodeKeys = new NodeKeyAllocator();
const listenersByNode = new NodeKeyTable();
/** NodeKey -> WeakRef<Handle>, so bubbling does not retain detached nodes. */
const nodesByKey = new NodeKeyTable();
const classesByNode = /* @__PURE__ */ new WeakMap();
const interactionByNode = /* @__PURE__ */ new WeakMap();
function emitInteractionPolicy(writer, node) {
	const state = interactionByNode.get(node) ?? {
		focusOrder: null,
		blocked: false,
		contained: false
	};
	let flags = 0;
	if (state.focusOrder !== null) flags |= INTERACTION_POLICY.Focusable;
	if (state.blocked) flags |= INTERACTION_POLICY.BlockSubtree;
	if (state.contained) flags |= INTERACTION_POLICY.ContainFocus;
	writer.setInteractionPolicy(node.id, flags, state.focusOrder ?? 0);
}
function emitClasses(writer, node) {
	const state = classesByNode.get(node);
	if (!state) return;
	const tokens = new Set(state.base.split(/\s+/).filter(Boolean));
	for (const [names, enabled] of Object.entries(state.toggles)) for (const token of names.split(/\s+/).filter(Boolean)) if (enabled) tokens.add(token);
	else tokens.delete(token);
	writer.setClassName(node.id, [...tokens].join(" "));
}
const finalizationRegistry = typeof FinalizationRegistry !== "undefined" ? new FinalizationRegistry((id) => {
	if (!nodeKeys.isLive(id)) return;
	nodesByKey.delete(id);
	listenersByNode.delete(id);
	writer.dropNode(id);
	nodeKeys.release(id);
}) : null;
const sweepSet = /* @__PURE__ */ new Set();
function runSweep() {
	if (sweepSet.size === 0) return;
	for (const node of sweepSet) {
		if (node.parent !== null) continue;
		const destroy = (n) => {
			if (!nodesByKey.has(n.id)) return;
			finalizationRegistry?.unregister(n);
			nodesByKey.delete(n.id);
			listenersByNode.delete(n.id);
			writer.dropNode(n.id);
			nodeKeys.release(n.id);
			let c = n.firstChild;
			while (c) {
				destroy(c);
				c = c.next;
			}
		};
		destroy(node);
	}
	sweepSet.clear();
}
function imperativeMethods(id) {
	const coordinates = (first, second) => typeof first === "number" ? [first, second ?? NaN] : [first.left ?? NaN, first.top ?? NaN];
	const scrollTo = ((first, second) => {
		const [x, y] = coordinates(first, second);
		writer.scrollTo(id, x, y);
	});
	const scrollBy = ((first, second) => {
		const [x, y] = coordinates(first, second);
		writer.scrollBy(id, x, y);
	});
	return {
		focus: () => writer.focusNode(id),
		scrollTo,
		scrollBy
	};
}
function makeHandle(tag) {
	const id = nodeKeys.allocate();
	const h = {
		id,
		tag,
		parent: null,
		firstChild: null,
		lastChild: null,
		prev: null,
		next: null,
		...imperativeMethods(id)
	};
	interactionByNode.set(h, {
		focusOrder: null,
		blocked: false,
		contained: false
	});
	if (typeof WeakRef !== "undefined") nodesByKey.set(id, new WeakRef(h));
	if (finalizationRegistry) finalizationRegistry.register(h, h.id, h);
	return h;
}
function linkChild(parent, child, ref) {
	child.parent = parent;
	if (ref == null) {
		child.prev = parent.lastChild;
		child.next = null;
		if (parent.lastChild) parent.lastChild.next = child;
		else parent.firstChild = child;
		parent.lastChild = child;
	} else {
		child.prev = ref.prev;
		child.next = ref;
		if (ref.prev) ref.prev.next = child;
		else parent.firstChild = child;
		ref.prev = child;
	}
}
function unlinkChild(parent, child) {
	if (child.prev) child.prev.next = child.next;
	else parent.firstChild = child.next;
	if (child.next) child.next.prev = child.prev;
	else parent.lastChild = child.prev;
	child.parent = child.prev = child.next = null;
}
/** Translate a setProperty call into protocol ops. Shared by both hooks. */
function applyProperty(writer, node, name, value, prev) {
	if (value === prev) return;
	if (name === "overlayPlane") {
		const plane = value === "modal" ? 2 : value === "floating" ? 1 : 0;
		writer.setOverlayPlane(node.id, plane);
		return;
	}
	if (name === "textBehavior") {
		const behavior = value && typeof value === "object" ? value : {
			flags: value,
			maxLines: 0
		};
		const flags = behavior.flags == null || behavior.flags === false ? 0 : Number(behavior.flags);
		const maxLines = behavior.maxLines == null || behavior.maxLines === false ? 0 : Number(behavior.maxLines);
		writer.setTextBehavior(node.id, flags);
		writer.setTextMaxLines(node.id, maxLines);
		return;
	}
	if (name === "focusOrder" || name === "interactionBlocked" || name === "focusContained") {
		const state = interactionByNode.get(node);
		if (name === "focusOrder") state.focusOrder = value == null || value === false ? null : Number(value);
		else if (name === "interactionBlocked") state.blocked = value === true;
		else state.contained = value === true;
		emitInteractionPolicy(writer, node);
		return;
	}
	if (name === "scrollbar") {
		const style = value && typeof value === "object" ? value : {};
		writer.setScrollbarStyle(node.id, {
			visibility: style.visibility === "always" ? 1 : style.visibility === "hidden" ? 2 : 0,
			hideDelay: style.hideDelay ?? 500,
			fadeDuration: style.fadeDuration ?? 200,
			thickness: style.thickness ?? 10,
			margin: style.margin ?? 2,
			minThumbLength: style.minThumbLength ?? 32,
			radius: style.radius ?? -1,
			trackColor: style.trackColor ?? 0,
			thumbColor: style.thumbColor ?? 1685360574,
			hoverColor: style.hoverColor ?? 1685360609,
			activeColor: style.activeColor ?? 1196780031
		});
		return;
	}
	if (name === "source") {
		if (node.tag === "vector-path") {
			if (value == null || value === false) writer.clearGraphicData(node.id, GRAPHIC_DATA.VectorPath);
			else if (isVectorPath(value)) {
				if (value.drawable) writer.setGraphicData(node.id, GRAPHIC_DATA.VectorPath, value.data);
				else writer.clearGraphicData(node.id, GRAPHIC_DATA.VectorPath);
			} else throw new TypeError("invalid native vector path source");
			return;
		}
		if (node.tag === "svg") {
			if (value == null || value === false) writer.clearGraphicSource(node.id, GRAPHIC_SOURCE.Svg);
			else if (typeof value === "string") writer.setGraphicSource(node.id, GRAPHIC_SOURCE.Svg, value);
			else throw new TypeError("invalid native SVG source");
			return;
		}
		if (value == null || value === false) {
			writer.clearGraphicSource(node.id, GRAPHIC_SOURCE.NetworkRaster);
			writer.clearGraphicSource(node.id, GRAPHIC_SOURCE.FileRaster);
			return;
		}
		if (typeof value === "object" && value.kind === "file" && typeof value.path === "string") {
			writer.setGraphicSource(node.id, GRAPHIC_SOURCE.FileRaster, value.path);
			return;
		}
		if (typeof value !== "object" || value.kind !== "network" || typeof value.url !== "string" || value.format !== "raster" || value.cache !== "memory") throw new TypeError("invalid native image source");
		writer.setGraphicSource(node.id, GRAPHIC_SOURCE.NetworkRaster, value.url);
		return;
	}
	if (name === "transform") {
		const matrix = value == null || value === false ? [
			1,
			0,
			0,
			1,
			0,
			0
		] : value;
		if (Array.isArray(matrix) && matrix.length === 6 && matrix.every((part) => typeof part === "number" && Number.isFinite(part))) writer.setTransform2D(node.id, matrix);
		return;
	}
	if (name === "class" || name === "className") {
		const state = classesByNode.get(node) ?? {
			base: "",
			toggles: {}
		};
		state.base = value == null || value === false ? "" : String(value);
		classesByNode.set(node, state);
		emitClasses(writer, node);
		return;
	}
	if (name === "classList") {
		const state = classesByNode.get(node) ?? {
			base: "",
			toggles: {}
		};
		state.toggles = {};
		if (value && typeof value === "object") for (const [token, enabled] of Object.entries(value)) state.toggles[token] = Boolean(enabled);
		classesByNode.set(node, state);
		emitClasses(writer, node);
		return;
	}
	if (name === "shadows") {
		if (value == null || value === false) writer.removeStyle(node.id, "box-shadow");
		else if (Array.isArray(value)) writer.setShadows(node.id, value);
		return;
	}
	if (name === "widgetConfig") {
		if (value == null || value === false) {
			writer.removeWidgetConfig(node.id);
			return;
		}
		if (!isStructuredConfigValue(value)) throw new TypeError("widgetConfig must be a plain object or array");
		writer.setWidgetConfig(node.id, stringifyWidgetConfig(value));
		return;
	}
	if (name.startsWith("aria-") && typeof value === "boolean") {
		writer.setAttribute(node.id, name, String(value));
		return;
	}
	if (value == null || value === false) {
		if (name.startsWith("on") && name.length > 2) {
			const t = EVENT_CODE[name.slice(2).toLowerCase()] ?? null;
			if (t != null) {
				writer.removeEventListener(node.id, t);
				listenersByNode.get(node.id)?.delete(t);
			}
			return;
		}
		writer.removeAttribute(node.id, name);
		return;
	}
	if (name === "style" && typeof value === "object" && value !== null) {
		const rec = value;
		const prec = prev && typeof prev === "object" ? prev : {};
		for (const k in rec) {
			const next = rec[k];
			if (k in prec && next === prec[k]) continue;
			if (next == null || next === false) {
				writer.removeStyle(node.id, k);
				continue;
			}
			assertInlineStyleValue(k, next);
			if (isTypedStyleValue(next)) {
				writer.setStyleValue(node.id, k, next.kind, next.value);
				continue;
			}
			writer.setStyle(node.id, k, String(next));
		}
		for (const k in prec) if (!(k in rec)) writer.removeStyle(node.id, k);
		return;
	}
	if (name === "textContent") {
		writer.setText(node.id, String(value));
		return;
	}
	if (name.startsWith("on") && typeof value === "function") {
		const t = EVENT_CODE[name.slice(2).toLowerCase()];
		if (t == null) return;
		writer.addEventListener(node.id, t);
		let m = listenersByNode.get(node.id);
		if (!m) {
			m = /* @__PURE__ */ new Map();
			listenersByNode.set(node.id, m);
		}
		m.set(t, value);
		return;
	}
	if (isStructuredConfigValue(value)) throw new TypeError(`object prop \`${name}\` is unsupported; use \`widgetConfig\` for native widget configuration`);
	writer.setAttribute(node.id, name, String(value));
}
const MAX_WIDGET_CONFIG_DEPTH = 32;
function isStructuredConfigValue(value) {
	if (Array.isArray(value)) return true;
	if (value === null || typeof value !== "object") return false;
	const proto = Object.getPrototypeOf(value);
	return proto === Object.prototype || proto === null;
}
function stringifyWidgetConfig(value) {
	const ancestors = /* @__PURE__ */ new Set();
	const visit = (current, depth) => {
		if (depth > MAX_WIDGET_CONFIG_DEPTH) throw new TypeError(`widgetConfig exceeds ${MAX_WIDGET_CONFIG_DEPTH} levels`);
		if (current === null || typeof current === "string" || typeof current === "boolean") return;
		if (typeof current === "number") {
			if (!Number.isFinite(current)) throw new TypeError("widgetConfig contains a non-finite number");
			return;
		}
		if (typeof current !== "object" || !isStructuredConfigValue(current)) throw new TypeError("widgetConfig contains a non-JSON value");
		if (ancestors.has(current)) throw new TypeError("widgetConfig contains a cycle");
		ancestors.add(current);
		if (Array.isArray(current)) for (const item of current) visit(item, depth + 1);
		else for (const item of Object.values(current)) visit(item, depth + 1);
		ancestors.delete(current);
	};
	visit(value, 0);
	return JSON.stringify(value);
}
const writer = new Writer();
const renderer = createRenderer({
	createElement(tag, staticProps) {
		const h = makeHandle(tag);
		writer.createElement(h.id, tag);
		if (staticProps) for (const [name, value] of Object.entries(staticProps)) applyProperty(writer, h, name, value, void 0);
		return h;
	},
	createTextNode(value) {
		const h = makeHandle("#text");
		writer.createText(h.id, String(value));
		return h;
	},
	replaceText(textNode, value) {
		writer.setText(textNode.id, String(value));
	},
	isTextNode(node) {
		return node.tag === "#text";
	},
	setProperty(node, name, value, prev) {
		applyProperty(writer, node, name, value, prev);
	},
	insertNode(parent, node, anchor) {
		if (node.parent) unlinkChild(node.parent, node);
		if (anchor) {
			linkChild(parent, node, anchor);
			writer.insertBefore(parent.id, node.id, anchor.id);
		} else {
			linkChild(parent, node, null);
			writer.appendChild(parent.id, node.id);
		}
	},
	removeNode(parent, node) {
		unlinkChild(parent, node);
		writer.removeChild(parent.id, node.id);
		sweepSet.add(node);
	},
	getParentNode(node) {
		return node.parent ?? void 0;
	},
	getFirstChild(node) {
		return node.firstChild ?? void 0;
	},
	getNextSibling(node) {
		return node.next ?? void 0;
	}
});
/** Imperative paint-only transform state for high-frequency animation. */
function setTransform2D(node, matrix) {
	if (!matrix.every(Number.isFinite)) return;
	writer.setTransform2D(node.id, matrix);
}
const render = renderer.render;
const createElement = renderer.createElement;
const createTextNode = renderer.createTextNode;
const insertNode = renderer.insertNode;
function removeNode(parent, node) {
	unlinkChild(parent, node);
	writer.removeChild(parent.id, node.id);
	sweepSet.add(node);
}
const insert = renderer.insert;
const setProp = renderer.setProp;
const createComponent$1 = renderer.createComponent;
const effect = renderer.effect;
const memo = renderer.memo;
const spread = renderer.spread;
const mergeProps = renderer.mergeProps;
const applyRef = renderer.applyRef;
const ref = renderer.ref;
function Dynamic(props) {
	const local = props;
	const others = omit(props, "component");
	const cached = createMemo(() => local.component);
	return createMemo(() => {
		const component = cached();
		switch (typeof component) {
			case "function": return untrack(() => component(others));
			case "string": {
				const el = createElement(component);
				spread(el, others, false);
				return el;
			}
		}
		return null;
	});
}
/** Register the root mount handle so bubbling reaches window-level listeners. */
function registerRoot(root) {
	if (typeof WeakRef !== "undefined") nodesByKey.set(root.id, new WeakRef(root));
}
/** Dispose callback for the last `mount()` — used by in-process HMR full reload. */
let activeMountDispose = null;
let mountedRoot = null;
const overlayRoots = /* @__PURE__ */ new Map();
/** Current native window root, used by renderer-level facilities like Portal. */
function getMountRoot() {
	if (!mountedRoot) throw new Error("Portal must be rendered inside mount()");
	return mountedRoot;
}
/** Acquire the shared synthetic host root for one public overlay plane. */
function acquireOverlayRoot(plane) {
	const existing = overlayRoots.get(plane);
	if (existing) {
		existing.users++;
		return existing.node;
	}
	const node = createElement("view");
	spread(node, {
		overlayPlane: plane,
		style: {
			position: "absolute",
			left: 0,
			top: 0,
			width: "100%",
			height: "100%",
			"pointer-events": "none"
		}
	}, false);
	insertNode(getMountRoot(), node, void 0);
	overlayRoots.set(plane, {
		node,
		users: 1
	});
	return node;
}
function releaseOverlayRoot(plane) {
	const entry = overlayRoots.get(plane);
	if (!entry || --entry.users > 0) return;
	overlayRoots.delete(plane);
	if (entry.node.parent) removeNode(entry.node.parent, entry.node);
}
/** Mount a Solid application into the host-provided root node. */
function mount(code) {
	if (activeMountDispose) {
		try {
			activeMountDispose();
		} catch (error) {
			__wabou_log("error", `mount dispose before remount failed: ${error}`);
		}
		activeMountDispose = null;
	}
	const root = {
		id: ROOT_NODE_KEY,
		tag: "#root",
		parent: null,
		firstChild: null,
		lastChild: null,
		prev: null,
		next: null,
		...imperativeMethods(ROOT_NODE_KEY)
	};
	mountedRoot = root;
	overlayRoots.clear();
	registerRoot(root);
	const dispose = render(code, root);
	activeMountDispose = () => {
		dispose();
		overlayRoots.clear();
		if (mountedRoot === root) mountedRoot = null;
		runSweep();
		writer.flush();
	};
	return () => {
		if (activeMountDispose) {
			activeMountDispose();
			activeMountDispose = null;
		}
	};
}
/**
* Solid compatibility adapter for a native Wabou event. It walks the Handle
* tree for bubbling and presents JSX handlers with a small familiar object;
* this is deliberately not a complete DOM Event implementation.
*/
function dispatchEvent(solidId, eventCode, payloadStr, numericData) {
	let data = {};
	if (payloadStr) try {
		data = JSON.parse(payloadStr);
	} catch {}
	else {
		const ed = numericData;
		if (ed) {
			if (eventCode === EVENT_CODE.pointerup || eventCode === EVENT_CODE.pointerdown || eventCode === EVENT_CODE.pointermove || eventCode === EVENT_CODE.click || eventCode === EVENT_CODE.contextmenu) {
				data.clientX = ed[0];
				data.clientY = ed[1];
				data.offsetX = ed[2];
				data.offsetY = ed[3];
				data.button = ed[4];
				data.buttons = ed[5];
				data.mods = ed[6];
			} else if (eventCode === EVENT_CODE.wheel) {
				data.clientX = ed[0];
				data.clientY = ed[1];
				data.offsetX = ed[2];
				data.offsetY = ed[3];
				data.deltaX = ed[7];
				data.deltaY = ed[8];
			} else if (eventCode === EVENT_CODE.scroll) {
				data.scrollX = ed[9];
				data.scrollY = ed[10];
			}
		}
	}
	let stopped = false;
	let defaultPrevented = false;
	const ev = {
		target: {
			id: solidId,
			...data
		},
		currentTarget: {
			id: solidId,
			...data
		},
		type: eventName(eventCode),
		payload: data,
		...data,
		stopPropagation() {
			stopped = true;
		},
		stopImmediatePropagation() {
			stopped = true;
		},
		preventDefault() {
			defaultPrevented = true;
		},
		get defaultPrevented() {
			return defaultPrevented;
		},
		get propagationStopped() {
			return stopped;
		}
	};
	const globalType = ev.type;
	const globalListeners = globalPointerListeners.get(globalType);
	if (globalListeners) {
		const target = derefHandle(solidId);
		for (const listener of [...globalListeners]) try {
			listener(target, ev);
		} catch (error) {
			logEventHandlerFailure(eventCode, solidId, solidId, error);
		}
	}
	bubble(solidId, eventCode, ev);
	return defaultPrevented;
}
function derefHandle(id) {
	return nodesByKey.get(id)?.deref();
}
/** Walk parent chain from `nodeId`, firing `code` listeners until stopped. */
function bubble(nodeId, code, ev) {
	let cur = nodeId;
	while (cur != null) {
		ev.currentTarget = nodeKeyEquals(cur, nodeId) ? ev.target : { id: cur };
		const fn = listenersByNode.get(cur)?.get(code);
		if (fn) try {
			const result = fn(ev);
			if (isPromiseLike(result)) {
				const current = cur;
				Promise.resolve(result).then(void 0, (error) => logEventHandlerFailure(code, current, nodeId, error));
			}
		} catch (e) {
			logEventHandlerFailure(code, cur, nodeId, e);
		}
		if (ev.propagationStopped) return;
		cur = derefHandle(cur)?.parent?.id ?? null;
	}
}
function isPromiseLike(value) {
	return value !== null && (typeof value === "object" || typeof value === "function") && typeof value.then === "function";
}
function logEventHandlerFailure(code, current, target, error) {
	const detail = error && typeof error === "object" && "stack" in error ? String(error.stack ?? error) : String(error);
	__wabou_log("error", `[wabou-event] ${eventName(code)} handler failed at node ${formatNodeKey(current)} (target ${formatNodeKey(target)})\n${detail}`);
}
/** event code -> DOM event name (for ev.type). */
function eventName(code) {
	for (const [name, c] of Object.entries(EVENT_CODE)) if (c === code) return name;
	return "unknown";
}
//#endregion
export { VirtualList as A, removeNode as C, setTransform2D as D, setProp as E, useHost as F, PathBuilder as I, isVectorPath as L, Portal as M, HostProvider as N, spread as O, defaultHost as P, releaseOverlayRoot as S, runSweep as T, mergeProps as _, createElement as a, ref as b, dispatchEvent as c, getRequestEvent as d, insert as f, memo as g, isServer as h, createComponent$1 as i, createFps as j, writer as k, effect as l, isDirectEvent as m, acquireOverlayRoot as n, createTextNode as o, insertNode as p, applyRef as r, delegateEvents as s, Dynamic as t, getMountRoot as u, mount as v, render as w, registerRoot as x, observeGlobalPointerEvent as y };

//# sourceMappingURL=renderer-CiLyfgWQ.mjs.map