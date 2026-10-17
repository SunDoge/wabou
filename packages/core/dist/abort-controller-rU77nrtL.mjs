import AbortControllerPolyfill, { AbortSignal } from "abort-controller/dist/abort-controller";
//#region src/polyfills/globals.ts
/** Install a writable runtime global without replacing a host implementation. */
function installMissingGlobal(name, value) {
	if (name in globalThis) return false;
	Object.defineProperty(globalThis, name, {
		configurable: true,
		writable: true,
		value
	});
	return true;
}
/** Install a related set of runtime globals with the same preservation rule. */
function installMissingGlobals(values) {
	for (const [name, value] of Object.entries(values)) installMissingGlobal(name, value);
}
//#endregion
//#region src/polyfills/dom-exception.ts
var WabouDOMException = class extends Error {
	code = 0;
	constructor(message = "", name = "Error") {
		super(message);
		this.name = name;
	}
};
/** Install the exception type shared by browser-compatible host APIs. */
function installDOMExceptionPolyfill() {
	installMissingGlobal("DOMException", WabouDOMException);
}
installDOMExceptionPolyfill();
//#endregion
//#region src/polyfills/abort-controller.ts
const reasons = /* @__PURE__ */ new WeakMap();
const Signal = AbortSignal;
Object.defineProperties(Signal.prototype, {
	reason: {
		configurable: true,
		get() {
			return reasons.get(this);
		}
	},
	throwIfAborted: {
		configurable: true,
		value() {
			if (this.aborted) throw this.reason;
		}
	}
});
var Controller = class extends AbortControllerPolyfill {
	abort(reason = new DOMException("The operation was aborted", "AbortError")) {
		if (this.signal.aborted) return;
		reasons.set(this.signal, reason);
		super.abort();
	}
};
Signal.abort = (reason) => {
	const c = new Controller();
	c.abort(reason);
	return c.signal;
};
Signal.timeout = (milliseconds) => {
	if (!Number.isSafeInteger(milliseconds) || milliseconds < 0) throw new RangeError("Invalid timeout");
	const c = new Controller();
	setTimeout(() => c.abort(new DOMException("The operation timed out", "TimeoutError")), milliseconds);
	return c.signal;
};
Signal.any = (signals) => {
	const c = new Controller();
	const cleanups = [];
	for (const signal of signals) {
		if (signal.aborted) {
			c.abort(signal.reason);
			break;
		}
		const abort = () => c.abort(signal.reason);
		signal.addEventListener("abort", abort, { once: true });
		cleanups.push(() => signal.removeEventListener("abort", abort));
	}
	const cleanup = () => {
		for (const remove of cleanups) remove();
	};
	if (c.signal.aborted) cleanup();
	else c.signal.addEventListener("abort", cleanup, { once: true });
	return c.signal;
};
/** Install cancellation primitives when the embedding runtime lacks them. */
function installAbortControllerPolyfill() {
	installMissingGlobals({
		AbortController: Controller,
		AbortSignal: Signal
	});
}
installAbortControllerPolyfill();
//#endregion
export { installMissingGlobals as n, installMissingGlobal as t };

//# sourceMappingURL=abort-controller-rU77nrtL.mjs.map