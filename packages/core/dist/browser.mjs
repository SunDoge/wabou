import { h as EVENT_CODE, s as nodeKey } from "./protocol-B9cErdtk.mjs";
import { t as hasBrowserDom } from "./browser-dom-BrHP4mZQ.mjs";
import { c as dispatchEvent, j as writer, l as disposeMountedApp } from "./renderer-BZp4nUlS.mjs";
import { t as tickAnimationFrame } from "./animation-frame-BgKEm5Et.mjs";
//#region src/generated/browser-host-bindings.ts
function createBrowserHostBindings(bridge, invalidate) {
	return {
		__wabou_intern: (value) => bridge.intern(value),
		__wabou_motion_create: (descriptor) => bridge.motion_create(descriptor),
		__wabou_motion_sample: (id, elapsed, output) => bridge.motion_sample(id, elapsed, output),
		__wabou_motion_drop: (id) => bridge.motion_drop(id),
		__wabou_set_stylesheet: (json) => {
			const result = bridge.set_stylesheet(json);
			invalidate();
			return result;
		}
	};
}
//#endregion
//#region src/browser/host.ts
const hostKey = Symbol.for("@wabou/core.browser-host");
const realm = globalThis;
const ownership = realm[hostKey] ?? { starting: false };
realm[hostKey] = ownership;
/** Initialize the host before importing the application's ordinary mount entry. */
async function startBrowserApp(options) {
	if (!hasBrowserDom()) throw new Error("Browser host requires a browser DOM");
	if (ownership.dispose || ownership.starting) throw new Error("Only one Wabou browser host is supported per JavaScript realm; use an iframe for another preview.");
	if (typeof requestAnimationFrame !== "function" || typeof cancelAnimationFrame !== "function" || typeof ResizeObserver !== "function") throw new Error("Browser host requires requestAnimationFrame and ResizeObserver");
	ownership.starting = true;
	const { canvas } = options;
	const bindings = globalThis;
	const previous = /* @__PURE__ */ new Map();
	const installed = /* @__PURE__ */ new Map();
	const own = (name, value) => {
		previous.set(name, bindings[name]);
		installed.set(name, value);
		bindings[name] = value;
	};
	let runtime;
	let mounted = false;
	let disposed = false;
	let frameId = 0;
	let observer;
	let dirty = true;
	let lastScale = 0;
	let ready = false;
	const abort = new AbortController();
	const requestFrame = requestAnimationFrame.bind(globalThis);
	const cancelFrame = cancelAnimationFrame.bind(globalThis);
	const NativeResizeObserver = ResizeObserver;
	const emit = (name, detail) => canvas.dispatchEvent(new CustomEvent(name, { detail }));
	let hovered = "";
	const pointerPayload = (event) => {
		const rect = canvas.getBoundingClientRect();
		const x = event.clientX - rect.left - canvas.clientLeft;
		const y = event.clientY - rect.top - canvas.clientTop;
		return JSON.stringify({
			clientX: x,
			clientY: y,
			offsetX: x,
			offsetY: y,
			button: event.button,
			buttons: event.buttons
		});
	};
	const dispatchPointer = (target, code, event) => {
		if (!target) return;
		const [lo, hi] = target.split(",").map(Number);
		dispatchEvent(nodeKey(lo, hi), code, pointerPayload(event));
		dirty = true;
	};
	const targetAt = (event, code) => {
		const rect = canvas.getBoundingClientRect();
		return runtime.hit_test(event.clientX - rect.left - canvas.clientLeft, event.clientY - rect.top - canvas.clientTop, code);
	};
	const click = (event) => {
		if (disposed || !runtime) return;
		dispatchPointer(targetAt(event, EVENT_CODE.click), EVENT_CODE.click, event);
	};
	const wheel = (event) => {
		if (disposed || !runtime?.scroll) return;
		const rect = canvas.getBoundingClientRect();
		const delta = event.deltaY * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.clientHeight : 1);
		if (runtime.scroll(event.clientX - rect.left - canvas.clientLeft, event.clientY - rect.top - canvas.clientTop, delta)) {
			event.preventDefault();
			dirty = true;
		}
	};
	const move = (event) => {
		if (disposed || !runtime || event.pointerType === "touch") return;
		const target = targetAt(event, EVENT_CODE.pointerenter);
		if (target !== hovered) {
			dispatchPointer(hovered, EVENT_CODE.pointerleave, event);
			hovered = target;
			dispatchPointer(hovered, EVENT_CODE.pointerenter, event);
		}
		dispatchPointer(targetAt(event, EVENT_CODE.pointermove), EVENT_CODE.pointermove, event);
	};
	const leave = (event) => {
		if (disposed || !runtime) return;
		dispatchPointer(hovered, EVENT_CODE.pointerleave, event);
		hovered = "";
	};
	const dispose = () => {
		if (disposed) return;
		disposed = true;
		abort.abort();
		cancelFrame(frameId);
		observer?.disconnect();
		canvas.removeEventListener("click", click);
		canvas.removeEventListener("wheel", wheel);
		canvas.removeEventListener("pointermove", move);
		canvas.removeEventListener("pointerleave", leave);
		canvas.removeEventListener("mouseleave", leave);
		try {
			if (mounted) disposeMountedApp();
		} finally {
			writer.reset();
			for (const [name, value] of installed) {
				if (bindings[name] !== value) continue;
				if (previous.get(name) === void 0) delete bindings[name];
				else bindings[name] = previous.get(name);
			}
			if (ownership.dispose === dispose) delete ownership.dispose;
			runtime?.free();
		}
	};
	try {
		writer.reset();
		runtime = await options.createRuntime(canvas);
		const bridge = runtime;
		for (const [name, binding] of Object.entries(createBrowserHostBindings(bridge, () => {
			dirty = true;
		}))) own(name, binding);
		own("__wabou_log", (_level, message) => console.error(message));
		own("__wabou_preview", {
			canvas,
			snapshot: () => JSON.parse(bridge.snapshot())
		});
		for (const url of options.fonts ?? []) {
			const font = await fetch(url, { signal: abort.signal });
			if (!font.ok) throw new Error(`Font failed (${font.status}): ${url}`);
			bridge.load_font(new Uint8Array(await font.arrayBuffer()));
		}
		mounted = true;
		await options.entry();
		observer = new NativeResizeObserver(() => {
			dirty = true;
		});
		observer.observe(canvas);
		canvas.addEventListener("click", click);
		canvas.addEventListener("wheel", wheel, { passive: false });
		canvas.addEventListener("pointermove", move);
		canvas.addEventListener("pointerleave", leave);
		canvas.addEventListener("mouseleave", leave);
		function frame(time) {
			if (disposed) return;
			try {
				tickAnimationFrame(time, (bytes) => {
					bridge.apply(bytes);
					dirty = true;
				});
				const motion = JSON.parse(bridge.tick_motion(time));
				for (const completion of motion.completed) dispatchEvent(nodeKey(completion.lo, completion.hi), EVENT_CODE.transitionend, JSON.stringify({ generation: completion.generation }));
				if (motion.active || motion.completed.length > 0) dirty = true;
				const scale = devicePixelRatio;
				const width = canvas.clientWidth;
				const height = canvas.clientHeight;
				if ((dirty || scale !== lastScale) && width > 0 && height > 0) {
					bridge.render(width, height, scale);
					dirty = false;
					emit("wabou:render");
					lastScale = scale;
					if (!ready) {
						ready = true;
						emit("wabou:ready");
					}
				}
				if (!disposed) frameId = requestFrame(frame);
			} catch (error) {
				emit("wabou:error", error);
				dispose();
			}
		}
		ownership.dispose = dispose;
		frameId = requestFrame(frame);
		return dispose;
	} catch (error) {
		dispose();
		emit("wabou:error", error);
		throw error;
	} finally {
		ownership.starting = false;
	}
}
//#endregion
export { startBrowserApp };

//# sourceMappingURL=browser.mjs.map