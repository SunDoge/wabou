import { t as hasBrowserDom } from "./browser-dom-BrHP4mZQ.mjs";
import { D as runSweep, j as writer } from "./renderer-Bu7emt2l.mjs";
import { flush } from "solid-js";
//#region src/glue/animation-frame.ts
var AnimationFrameQueue = class {
	#callbacks = /* @__PURE__ */ new Map();
	#nextId = 1;
	request(callback) {
		const id = this.#nextId++;
		this.#callbacks.set(id, callback);
		return id;
	}
	cancel(id) {
		this.#callbacks.delete(id);
	}
	drain() {
		const entries = Array.from(this.#callbacks.entries());
		this.#callbacks.clear();
		return entries;
	}
	hasPending() {
		return this.#callbacks.size > 0;
	}
};
const animationFrames = new AnimationFrameQueue();
function requestAnimationFrameImpl(cb) {
	return animationFrames.request(cb);
}
function cancelAnimationFrameImpl(id) {
	animationFrames.cancel(id);
}
function tickAnimationFrame(frameTime, deliver = __wabou_flush, flushWriter = () => writer.flush(), commit = flush, queue = animationFrames) {
	const entries = queue.drain();
	commit(() => {
		for (const [_, cb] of entries) try {
			cb(frameTime);
		} catch (error) {
			__wabou_log("error", error instanceof Error && error.stack ? error.stack : String(error));
		}
	});
	runSweep();
	const bytes = flushWriter();
	if (bytes) deliver(bytes);
	return queue.hasPending();
}
function __wabou_tick(frameTime) {
	return tickAnimationFrame(frameTime);
}
function __wabou_has_raf() {
	return animationFrames.hasPending();
}
if (!hasBrowserDom()) {
	globalThis.requestAnimationFrame = requestAnimationFrameImpl;
	globalThis.cancelAnimationFrame = cancelAnimationFrameImpl;
}
globalThis.__wabou_tick = __wabou_tick;
globalThis.__wabou_has_raf = __wabou_has_raf;
//#endregion
export { tickAnimationFrame as t };

//# sourceMappingURL=animation-frame-BOdlCQ1h.mjs.map