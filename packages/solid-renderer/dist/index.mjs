import { EVENT_CODE, OP, Writer } from "@wabou/protocol";
import { assertInlineStyleValue, isTypedStyleValue } from "@wabou/style";
import { For, Show, createComponent as createComponent$1, createContext, createMemo, createSignal, getOwner, omit, onCleanup, untrack, useContext } from "solid-js";
import { createRenderer } from "@solidjs/universal";
import { createComponent as createComponent$2, createElement as createElement$1, effect as effect$1, insert as insert$1, insertNode as insertNode$1, ref as ref$1, setProp as setProp$1 } from "@wabou/solid-renderer";
import { Virtualizer } from "@tanstack/virtual-core";
//#region src/host.tsx
/** Checked adapter around the private Rust/QuickJS ABI. */
const nativeHost = {
	openUrl: (url) => __wabou_open_url(url),
	loadFont: (path) => __wabou_load_font(path),
	frameStats: () => JSON.parse(__wabou_frame_stats()),
	layoutSnapshot: (ids) => JSON.parse(__wabou_layout_snapshot(Uint32Array.from(ids))),
	systemLocale: () => __wabou_system_locale(),
	systemTimeZone: () => __wabou_system_time_zone(),
	systemCalendarDate: () => JSON.parse(__wabou_system_calendar_date())
};
const builtinHost = {
	system: { openUrl: nativeHost.openUrl },
	fonts: { load: nativeHost.loadFont },
	diagnostics: { frameStats: nativeHost.frameStats },
	intl: {
		locale: nativeHost.systemLocale,
		timeZone: nativeHost.systemTimeZone,
		today: nativeHost.systemCalendarDate
	},
	layout: {
		snapshot: (targets) => nativeHost.layoutSnapshot(targets.map((target) => typeof target === "number" ? target : target.id)),
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
	return createComponent$1(HostContext, {
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
//#region src/portal.ts
/** Render a native host subtree under its shared synthetic overlay root. */
function Portal(props) {
	const local = props;
	const containerProps = omit(props, "children", "plane");
	const plane = local.plane ?? "floating";
	const root = acquireOverlayRoot(plane);
	const container = createElement("view");
	spread(container, containerProps, false);
	if (plane === "modal") spread(container, {
		"aria-modal": "true",
		overlayPlane: "modal"
	}, false);
	insertNode(root, container, void 0);
	insert(container, () => local.children);
	onCleanup(() => {
		if (container.parent) removeNode(container.parent, container);
		releaseOverlayRoot(plane);
	});
	return null;
}
//#endregion
//#region src/use-fps.ts
/**
* Track frames-per-second. A self-perpetuating rAF loop counts frames; a
* 1s interval samples the count and resets it. The rAF loop keeps the host
* redrawing (it drives `has_anim`), so this measures the active vsync rate
* while mounted — ~60 on a 60Hz display, ~120 on 120Hz. When nothing animates,
* the host stops redrawing and the count drops.
*
* ```tsx
* const fps = createFps();
* <div>{fps()} fps</div>
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
/** @deprecated Use createFps; this primitive creates owned timers rather than consuming context. */
const useFps = createFps;
//#endregion
//#region src/virtual-list.tsx
/**
* Windowed Solid list backed by TanStack Virtual's framework-neutral core.
* Rust remains authoritative for scrolling, clipping, hit testing and the
* native scrollbar; this adapter supplies viewport/offset observations instead
* of relying on HTMLElement, ResizeObserver or getBoundingClientRect().
*/
function VirtualList(props) {
	const surface = {};
	let scrollHandle;
	let publishOffset;
	let scrollEndTimer;
	let lastOffset = 0;
	const [version, invalidate] = createSignal(0, { equals: false });
	const options = () => ({
		count: props.items().length,
		getScrollElement: () => scrollHandle ? surface : null,
		estimateSize: () => props.itemHeight,
		overscan: props.overscan ?? 4,
		initialRect: {
			width: 0,
			height: props.viewportHeight
		},
		observeElementRect: (_instance, notify) => {
			notify({
				width: 0,
				height: props.viewportHeight
			});
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
	const virtualizer = new Virtualizer(options());
	const dispose = virtualizer._didMount();
	onCleanup(() => {
		if (scrollEndTimer !== void 0) clearTimeout(scrollEndTimer);
		dispose();
	});
	const virtualItems = createMemo(() => {
		version();
		props.items();
		virtualizer.setOptions(options());
		virtualizer._willUpdate();
		return virtualizer.getVirtualItems();
	});
	const totalSize = createMemo(() => {
		virtualItems();
		return virtualizer.getTotalSize();
	});
	var _el$ = createElement$1("div", { onScroll: (event) => {
		lastOffset = event.scrollY ?? 0;
		publishOffset?.(lastOffset, true);
		if (scrollEndTimer !== void 0) clearTimeout(scrollEndTimer);
		scrollEndTimer = setTimeout(() => {
			scrollEndTimer = void 0;
			publishOffset?.(lastOffset, false);
		}, 150);
	} });
	var _el$2 = createElement$1("div");
	insertNode$1(_el$, _el$2);
	ref$1(() => {
		return (node) => {
			scrollHandle = node;
			virtualizer._willUpdate();
		};
	}, _el$);
	insert$1(_el$2, createComponent$2(For, {
		get each() {
			return virtualItems();
		},
		keyed: false,
		children: (virtualItem) => (() => {
			var _el$3 = createElement$1("div");
			insert$1(_el$3, createComponent$2(Show, {
				get when() {
					return virtualItem().index + 1;
				},
				keyed: true,
				children: (key) => {
					const index = key - 1;
					return props.children(props.items()[index], index);
				}
			}));
			effect$1(() => ({
				position: "absolute",
				top: `${virtualItem().start}px`,
				height: `${virtualItem().size}px`,
				width: "100%"
			}), (_v$, _$p) => {
				setProp$1(_el$3, "style", _v$, _$p);
			});
			return _el$3;
		})()
	}));
	effect$1(() => {
		return {
			e: props.role,
			t: props.accessibilityLabel,
			a: {
				overflow: "scroll",
				position: "relative",
				height: `${props.viewportHeight}px`,
				width: "100%"
			},
			o: {
				position: "relative",
				height: `${totalSize()}px`,
				width: "100%"
			}
		};
	}, ({ e, t, a, o }, _p$) => {
		e !== _p$?.e && setProp$1(_el$, "role", e, _p$?.e);
		t !== _p$?.t && setProp$1(_el$, "aria-label", t, _p$?.t);
		a !== _p$?.a && setProp$1(_el$, "style", a, _p$?.a);
		o !== _p$?.o && setProp$1(_el$2, "style", o, _p$?.o);
	});
	return _el$;
}
//#endregion
//#region src/index.ts
const isServer = false;
const getRequestEvent = () => void 0;
const delegateEvents = () => {};
const FREE_LIST = [];
const GENERATIONS = [];
let nextSlot = 2;
const listenersBySlot = [];
/** solid id -> WeakRef<Handle>, so event dispatch can walk the parent chain for bubbling without leaking memory. */
const nodesBySlot = [];
const classesByNode = /* @__PURE__ */ new WeakMap();
function emitClasses(writer, node) {
	const state = classesByNode.get(node);
	if (!state) return;
	const tokens = new Set(state.base.split(/\s+/).filter(Boolean));
	for (const [names, enabled] of Object.entries(state.toggles)) for (const token of names.split(/\s+/).filter(Boolean)) if (enabled) tokens.add(token);
	else tokens.delete(token);
	writer.setClassName(node.id, [...tokens].join(" "));
}
const finalizationRegistry = typeof FinalizationRegistry !== "undefined" ? new FinalizationRegistry((id) => {
	const slot = id & 1048575;
	const expectedGen = id >>> 20;
	if (GENERATIONS[slot] !== expectedGen) return;
	nodesBySlot[slot] = void 0;
	listenersBySlot[slot] = void 0;
	writer.dropNode(id);
	freeId(id);
}) : null;
const sweepSet = /* @__PURE__ */ new Set();
function runSweep() {
	if (sweepSet.size === 0) return;
	for (const node of sweepSet) {
		if (node.parent !== null) continue;
		const destroy = (n) => {
			const slot = n.id & 1048575;
			if (nodesBySlot[slot] === void 0) return;
			finalizationRegistry?.unregister(n);
			nodesBySlot[slot] = void 0;
			listenersBySlot[slot] = void 0;
			writer.dropNode(n.id);
			freeId(n.id);
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
function newId() {
	let slot;
	if (FREE_LIST.length > 0) slot = FREE_LIST.pop();
	else {
		slot = nextSlot++;
		GENERATIONS[slot] = 0;
	}
	return (GENERATIONS[slot] << 20 | slot) >>> 0;
}
function freeId(id) {
	const slot = id & 1048575;
	GENERATIONS[slot] = GENERATIONS[slot] + 1 & 4095;
	FREE_LIST.push(slot);
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
	const id = newId();
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
	if (typeof WeakRef !== "undefined") nodesBySlot[id & 1048575] = new WeakRef(h);
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
		if (node.tag === "svg") {
			if (value == null || value === false) writer.removeAttribute(node.id, "svg-source");
			else if (typeof value === "string") writer.setAttribute(node.id, "svg-source", value);
			else throw new TypeError("invalid native SVG source");
			return;
		}
		if (value == null || value === false) {
			writer.removeAttribute(node.id, "image-source");
			return;
		}
		if (typeof value !== "object" || value.kind !== "network" || typeof value.url !== "string" || value.format !== "raster" || value.cache !== "memory") throw new TypeError("invalid native image source");
		writer.setAttribute(node.id, "image-source", JSON.stringify(value));
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
				const slot = node.id & 1048575;
				writer.removeEventListener(node.id, t);
				listenersBySlot[slot]?.delete(t);
			}
			return;
		}
		if (name === "href") node.href = void 0;
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
	if (name === "href") node.href = String(value);
	if (name.startsWith("on") && typeof value === "function") {
		const t = EVENT_CODE[name.slice(2).toLowerCase()];
		if (t == null) return;
		writer.addEventListener(node.id, t);
		const slot = node.id & 1048575;
		let m = listenersBySlot[slot];
		if (!m) {
			m = /* @__PURE__ */ new Map();
			listenersBySlot[slot] = m;
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
const createComponent = renderer.createComponent;
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
	if (typeof WeakRef !== "undefined") nodesBySlot[root.id & 1048575] = new WeakRef(root);
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
		id: 1,
		tag: "#root",
		parent: null,
		firstChild: null,
		lastChild: null,
		prev: null,
		next: null,
		...imperativeMethods(1)
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
			if (eventCode === EVENT_CODE.pointerup || eventCode === EVENT_CODE.pointerdown || eventCode === EVENT_CODE.pointermove || eventCode === EVENT_CODE.click) {
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
	bubble(solidId, eventCode, {
		target: {
			id: solidId,
			...data
		},
		currentTarget: {
			id: solidId,
			...data
		},
		type: eventName(eventCode),
		...data,
		stopPropagation() {
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
	});
	return defaultPrevented;
}
function derefHandle(id) {
	const stored = nodesBySlot[id & 1048575];
	return stored instanceof WeakRef ? stored.deref() : stored;
}
/** Walk parent chain from `nodeId`, firing `code` listeners until stopped. */
function bubble(nodeId, code, ev) {
	let cur = nodeId;
	while (cur != null) {
		const slot = cur & 1048575;
		ev.currentTarget = cur === nodeId ? ev.target : { id: cur };
		const fn = listenersBySlot[slot]?.get(code);
		if (fn) try {
			fn(ev);
		} catch (e) {
			const detail = e && typeof e === "object" && "stack" in e ? String(e.stack ?? e) : String(e);
			__wabou_log("error", `[wabou-event] ${eventName(code)} handler failed at node ${cur} (target ${nodeId})\n${detail}`);
		}
		if (ev.propagationStopped) return;
		cur = derefHandle(cur)?.parent?.id ?? null;
	}
}
/** event code -> DOM event name (for ev.type). */
function eventName(code) {
	for (const [name, c] of Object.entries(EVENT_CODE)) if (c === code) return name;
	return "unknown";
}
//#endregion
export { Dynamic, EVENT_CODE, HostProvider, OP, Portal, VirtualList, acquireOverlayRoot, applyRef, createComponent, createElement, createFps, createTextNode, defaultHost, delegateEvents, dispatchEvent, effect, getMountRoot, getRequestEvent, insert, insertNode, isServer, memo, mergeProps, mount, ref, registerRoot, releaseOverlayRoot, removeNode, render, runSweep, setProp, setTransform2D, spread, useFps, useHost, writer };

//# sourceMappingURL=index.mjs.map