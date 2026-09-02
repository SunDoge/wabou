import { S as createResourceKeyFamily, c as HOST_RECORD_KIND, n as EVENT_DATA_LEN, o as HOST_FRAME, s as HOST_NODE_PAYLOAD, v as nodeKey } from "./protocol-CdThEzKd.mjs";
import { _ as utilityConflictProperties, a as bool, c as mergeClasses, d as px, f as rgba, g as translate2d, h as shadow, i as auto, l as number, m as scale2d, n as StyleValueKind, o as classes, p as rotate2d, r as assertInlineStyleValue, s as isTypedStyleValue, t as STYLE_VALUE, u as percent, v as INLINE_STYLE_CONTRACT } from "./style-DgJ-RVg4.mjs";
import { A as writer, E as runSweep, F as defaultHost, I as useHost, L as PathBuilder, M as createFps, N as Portal, O as setTransform2D, P as HostProvider, R as isVectorPath, b as reconcileControlledInputValues, c as dispatchEvent, j as VirtualList, m as isDirectEvent, t as Dynamic, v as mount, y as observeGlobalPointerEvent } from "./renderer-BYDRnnWQ.mjs";
import { a as subscribeAll, i as subscribe, n as dispatchHostMessage, o as subscribeJson, r as hostMessages, t as dispatchResizeObservation } from "./resize-observer-BKduhWC2.mjs";
import { n as effectOps } from "./effect-abi-BzPW8STE.mjs";
import "./registry.mjs";
import AbortControllerPolyfill, { AbortSignal } from "abort-controller/dist/abort-controller";
import { ByteLengthQueuingStrategy, CountQueuingStrategy, ReadableByteStreamController, ReadableStream as ReadableStream$1, ReadableStreamBYOBReader, ReadableStreamBYOBRequest, ReadableStreamDefaultController, ReadableStreamDefaultReader, TransformStream, TransformStreamDefaultController, WritableStream, WritableStreamDefaultController, WritableStreamDefaultWriter } from "web-streams-polyfill";
import { TextDecoderStream, TextEncoderStream } from "@stardazed/streams-text-encoding";
import { For, createComponent, createContext, createEffect, createMemo, createSignal, flush, getOwner, latest, onCleanup, refresh, resolve, untrack, useContext } from "solid-js";
//#region src/polyfills/abort-controller.ts
/** Install cancellation primitives when the embedding runtime lacks them. */
function installAbortControllerPolyfill() {
	if (!("AbortSignal" in globalThis)) Object.defineProperty(globalThis, "AbortSignal", {
		configurable: true,
		writable: true,
		value: AbortSignal
	});
	if (!("AbortController" in globalThis)) Object.defineProperty(globalThis, "AbortController", {
		configurable: true,
		writable: true,
		value: AbortControllerPolyfill
	});
}
installAbortControllerPolyfill();
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
	if (!("DOMException" in globalThis)) Object.defineProperty(globalThis, "DOMException", {
		configurable: true,
		writable: true,
		value: WabouDOMException
	});
}
installDOMExceptionPolyfill();
//#endregion
//#region src/polyfills/crypto.ts
const DIGEST_IDS = {
	"SHA-1": 1,
	"SHA-256": 2,
	"SHA-384": 3,
	"SHA-512": 4
};
function digestName(algorithm) {
	const raw = typeof algorithm === "string" ? algorithm : algorithm.name;
	const name = raw.toUpperCase();
	if (!(name in DIGEST_IDS)) throw new DOMException(`Unsupported digest algorithm: ${raw}`, "NotSupportedError");
	return name;
}
function bytesOf(source) {
	if (ArrayBuffer.isView(source)) return new Uint8Array(source.buffer, source.byteOffset, source.byteLength);
	return new Uint8Array(source);
}
function isIntegerArray(value) {
	return value instanceof Int8Array || value instanceof Uint8Array || value instanceof Uint8ClampedArray || value instanceof Int16Array || value instanceof Uint16Array || value instanceof Int32Array || value instanceof Uint32Array || typeof BigInt64Array !== "undefined" && value instanceof BigInt64Array || typeof BigUint64Array !== "undefined" && value instanceof BigUint64Array;
}
var WabouSubtleCrypto = class {
	async digest(algorithm, data) {
		const name = digestName(algorithm);
		return (await globalThis.__wabou_crypto_digest(DIGEST_IDS[name], bytesOf(data))).buffer;
	}
};
var WabouCrypto = class {
	subtle = new WabouSubtleCrypto();
	getRandomValues(array) {
		if (array === null || !isIntegerArray(array)) throw new DOMException("getRandomValues requires an integer TypedArray", "TypeMismatchError");
		if (array.byteLength > 65536) throw new DOMException("getRandomValues cannot fill more than 65536 bytes", "QuotaExceededError");
		globalThis.__wabou_crypto_random(bytesOf(array));
		return array;
	}
	randomUUID() {
		const bytes = this.getRandomValues(/* @__PURE__ */ new Uint8Array(16));
		bytes[6] = bytes[6] & 15 | 64;
		bytes[8] = bytes[8] & 63 | 128;
		const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0"));
		return `${hex.slice(0, 4).join("")}-${hex.slice(4, 6).join("")}-${hex.slice(6, 8).join("")}-${hex.slice(8, 10).join("")}-${hex.slice(10).join("")}`;
	}
};
/** Install the native random and digest subset when Wabou's ABI is present. */
function installCryptoPolyfill() {
	if (!("__wabou_crypto_random" in globalThis)) return;
	if (!("crypto" in globalThis)) Object.defineProperty(globalThis, "crypto", {
		configurable: true,
		writable: true,
		value: new WabouCrypto()
	});
}
installCryptoPolyfill();
//#endregion
//#region src/polyfills/streams.ts
const streamGlobals = {
	ByteLengthQueuingStrategy,
	CountQueuingStrategy,
	ReadableByteStreamController,
	ReadableStream: ReadableStream$1,
	ReadableStreamBYOBReader,
	ReadableStreamBYOBRequest,
	ReadableStreamDefaultController,
	ReadableStreamDefaultReader,
	TransformStream,
	TransformStreamDefaultController,
	WritableStream,
	WritableStreamDefaultController,
	WritableStreamDefaultWriter
};
/** Install the WHATWG Streams constructors missing from the current runtime. */
function installStreamsPolyfill() {
	for (const [name, constructor] of Object.entries(streamGlobals)) {
		if (name in globalThis) continue;
		Object.defineProperty(globalThis, name, {
			configurable: true,
			writable: true,
			value: constructor
		});
	}
}
installStreamsPolyfill();
//#endregion
//#region src/polyfills/encoding-streams.ts
const encodingStreamGlobals = {
	TextDecoderStream,
	TextEncoderStream
};
/** Install the Encoding Standard stream transforms missing from QuickJS. */
function installEncodingStreamsPolyfill() {
	for (const [name, constructor] of Object.entries(encodingStreamGlobals)) {
		if (name in globalThis) continue;
		Object.defineProperty(globalThis, name, {
			configurable: true,
			writable: true,
			value: constructor
		});
	}
}
installEncodingStreamsPolyfill();
//#endregion
//#region src/polyfills/fetch.ts
function normalizeHeaderName(name) {
	return String(name).toLowerCase();
}
var WabouHeaders = class {
	entriesByName = /* @__PURE__ */ new Map();
	constructor(init) {
		if (!init) return;
		if (Symbol.iterator in Object(init)) {
			for (const [name, value] of init) this.append(name, value);
			return;
		}
		for (const [name, value] of Object.entries(init)) this.append(name, value);
	}
	append(name, value) {
		const key = normalizeHeaderName(name);
		const current = this.entriesByName.get(key);
		this.entriesByName.set(key, current ? `${current}, ${String(value)}` : String(value));
	}
	delete(name) {
		this.entriesByName.delete(normalizeHeaderName(name));
	}
	get(name) {
		return this.entriesByName.get(normalizeHeaderName(name)) ?? null;
	}
	has(name) {
		return this.entriesByName.has(normalizeHeaderName(name));
	}
	set(name, value) {
		this.entriesByName.set(normalizeHeaderName(name), String(value));
	}
	entries() {
		return this.entriesByName.entries();
	}
	keys() {
		return this.entriesByName.keys();
	}
	values() {
		return this.entriesByName.values();
	}
	forEach(callback, thisArg) {
		for (const [key, value] of this.entriesByName) callback.call(thisArg, value, key, this);
	}
	[Symbol.iterator]() {
		return this.entries();
	}
	toRecord() {
		return Object.fromEntries(this.entriesByName);
	}
};
function encodeResponseBody(body, copy) {
	if (typeof body === "string") return new TextEncoder().encode(body);
	if (body instanceof Uint8Array) return copy ? body.slice() : body;
	if (body instanceof ArrayBuffer) return new Uint8Array(body.slice(0));
	return /* @__PURE__ */ new Uint8Array();
}
var WabouResponse = class WabouResponse {
	body;
	headers;
	status;
	statusText;
	url;
	bodyBytes;
	consumed = false;
	constructor(body = null, init = {}, url = "", copyBody = true) {
		this.bodyBytes = encodeResponseBody(body, copyBody);
		this.status = init.status ?? 200;
		this.statusText = init.statusText ?? "";
		this.headers = new WabouHeaders(init.headers);
		this.url = url;
		if (body === null) this.body = null;
		else {
			const stream = new ReadableStream({ start: (controller) => {
				controller.enqueue(this.bodyBytes);
				controller.close();
			} });
			const getReader = stream.getReader.bind(stream);
			stream.getReader = ((...args) => {
				this.consumed = true;
				return getReader(...args);
			});
			this.body = stream;
		}
	}
	get ok() {
		return this.status >= 200 && this.status < 300;
	}
	get bodyUsed() {
		return this.consumed || this.body?.locked === true;
	}
	async consumeBody() {
		if (this.bodyUsed) throw new TypeError("Response body has already been consumed");
		if (this.body === null) {
			this.consumed = true;
			return /* @__PURE__ */ new Uint8Array();
		}
		const reader = this.body.getReader();
		const chunks = [];
		let length = 0;
		try {
			while (true) {
				const { done, value } = await reader.read();
				if (done) break;
				chunks.push(value);
				length += value.length;
			}
		} finally {
			reader.releaseLock();
		}
		const bytes = new Uint8Array(length);
		let offset = 0;
		for (const chunk of chunks) {
			bytes.set(chunk, offset);
			offset += chunk.length;
		}
		return bytes;
	}
	async text() {
		return new TextDecoder().decode(await this.consumeBody());
	}
	async json() {
		return JSON.parse(await this.text());
	}
	bytes() {
		return this.consumeBody();
	}
	async arrayBuffer() {
		return (await this.consumeBody()).slice().buffer;
	}
	clone() {
		if (this.bodyUsed) throw new TypeError("Cannot clone a consumed Response body");
		return new WabouResponse(this.bodyBytes, {
			status: this.status,
			statusText: this.statusText,
			headers: this.headers
		}, this.url);
	}
	static json(value, init = {}) {
		const headers = new WabouHeaders(init.headers);
		if (!headers.has("content-type")) headers.set("content-type", "application/json");
		return new WabouResponse(JSON.stringify(value), {
			...init,
			headers
		});
	}
	static fromHost(data, url) {
		return new WabouResponse(data.body, {
			status: data.status,
			statusText: data.statusText,
			headers: data.headers
		}, url, false);
	}
};
/** Install the host-backed Fetch API surface. Safe to call again in tests. */
function installFetchPolyfill() {
	if (!("Headers" in globalThis)) Object.defineProperty(globalThis, "Headers", {
		configurable: true,
		writable: true,
		value: WabouHeaders
	});
	if (!("Response" in globalThis)) Object.defineProperty(globalThis, "Response", {
		configurable: true,
		writable: true,
		value: WabouResponse
	});
	if (!("__wabou_fetch" in globalThis)) return;
	globalThis.fetch = ((input, init) => {
		const url = typeof input === "string" ? input : input.url;
		const serializedInit = init ? {
			...init,
			headers: init.headers instanceof WabouHeaders ? init.headers.toRecord() : init.headers
		} : {};
		return globalThis.__wabou_fetch(url, JSON.stringify(serializedInit)).then((data) => {
			const ResponseConstructor = globalThis.Response;
			if (ResponseConstructor === WabouResponse) return WabouResponse.fromHost(data, url);
			return new ResponseConstructor(data.body, {
				status: data.status,
				statusText: data.statusText,
				headers: data.headers
			});
		});
	});
}
installFetchPolyfill();
//#endregion
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
globalThis.requestAnimationFrame = requestAnimationFrameImpl;
globalThis.cancelAnimationFrame = cancelAnimationFrameImpl;
globalThis.__wabou_tick = __wabou_tick;
globalThis.__wabou_has_raf = __wabou_has_raf;
//#endregion
//#region src/glue/app-lifecycle.ts
function decodeAppLifecycle(value) {
	if (typeof value !== "object" || value === null) throw new TypeError("application lifecycle event must be an object");
	const state = value.state;
	if (state !== "resumed" && state !== "suspended" && state !== "memory-warning") throw new TypeError("application lifecycle event has an invalid state");
	return { state };
}
/** Subscribe to operating-system lifecycle notifications. */
function subscribeAppLifecycle(handler) {
	return subscribeJson("wabou:app-lifecycle", handler, { decode: decodeAppLifecycle });
}
/** Subscribe for the lifetime of the current Solid owner. */
function useAppLifecycle(handler) {
	onCleanup(subscribeAppLifecycle(handler));
}
//#endregion
//#region src/glue/timers.ts
let nextTimerId = 1;
const active = /* @__PURE__ */ new Set();
const nativeSetTimeout = globalThis.setTimeout?.bind(globalThis);
function sleep(delay) {
	if (typeof __wabou_sleep === "function") return __wabou_sleep(delay);
	if (nativeSetTimeout) return new Promise((resolve) => nativeSetTimeout(resolve, delay));
	return Promise.reject(/* @__PURE__ */ new Error("Wabou timer host is unavailable"));
}
function reportTimerError(error) {
	const message = error instanceof Error && error.stack ? error.stack : String(error);
	if (typeof __wabou_log === "function") __wabou_log("error", message);
	else console.error(message);
}
function schedule(callback, delay, repeat, args) {
	const id = nextTimerId++;
	active.add(id);
	const run = async () => {
		await sleep(delay);
		if (!active.has(id)) return;
		try {
			callback(...args);
		} catch (error) {
			reportTimerError(error);
		}
		if (repeat && active.has(id)) run();
		else active.delete(id);
	};
	run();
	return id;
}
globalThis.setTimeout = (callback, delay = 0, ...args) => schedule(callback, Number(delay) || 0, false, args);
globalThis.setInterval = (callback, delay = 0, ...args) => schedule(callback, Number(delay) || 0, true, args);
function clearTimer(id) {
	active.delete(id);
}
globalThis.clearTimeout = clearTimer;
globalThis.clearInterval = clearTimer;
//#endregion
//#region src/glue/host-frame.ts
const RECORD_HEADER_LEN = 8;
const FLAG_CANCELLABLE = 1;
const textDecoder = new TextDecoder();
function viewOf(input) {
	const bytes = input instanceof ArrayBuffer ? new Uint8Array(input) : new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
	return {
		bytes,
		view: new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
	};
}
/**
* Decode the complete frame before dispatching any record. Malformed frames
* are atomic: no listener, observer or application subscriber is called.
*/
function decodeAndDispatchHostFrame(input) {
	const { bytes, view } = viewOf(input);
	if (view.byteLength < HOST_FRAME.HeaderLen) throw new TypeError("short HostEventFrame header");
	if (view.getUint32(0, true) !== HOST_FRAME.Magic) throw new TypeError("invalid HostEventFrame magic");
	if (view.getUint16(4, true) !== HOST_FRAME.Version) throw new TypeError("unsupported HostEventFrame version");
	const count = view.getUint32(24, true);
	const byteLen = view.getUint32(28, true);
	if (byteLen !== view.byteLength) throw new TypeError("HostEventFrame byte length mismatch");
	let offset = HOST_FRAME.HeaderLen;
	const records = [];
	const requireBytes = (length, end) => {
		if (length < 0 || offset + length > end) throw new TypeError("truncated HostEventFrame record");
	};
	for (let index = 0; index < count; index++) {
		if (offset + RECORD_HEADER_LEN > byteLen) throw new TypeError("truncated HostEventFrame record header");
		const kind = view.getUint8(offset);
		const flags = view.getUint8(offset + 1);
		const recordLen = view.getUint32(offset + 4, true);
		if (recordLen < RECORD_HEADER_LEN || offset + recordLen > byteLen) throw new TypeError("invalid HostEventFrame record length");
		const end = offset + recordLen;
		offset += RECORD_HEADER_LEN;
		if (kind === HOST_RECORD_KIND.NodeEvent) {
			requireBytes(16, end);
			const target = nodeKey(view.getUint32(offset, true), view.getUint32(offset + 4, true));
			const eventCode = view.getUint8(offset + 8);
			const payloadKind = view.getUint8(offset + 9);
			const numericLen = view.getUint16(offset + 10, true);
			const eventId = view.getUint32(offset + 12, true);
			offset += 16;
			if (payloadKind === HOST_NODE_PAYLOAD.None) records.push({
				kind: "node",
				flags,
				target,
				eventCode,
				eventId,
				json: ""
			});
			else if (payloadKind === HOST_NODE_PAYLOAD.Numeric) {
				if (numericLen > EVENT_DATA_LEN) throw new TypeError("HostEventFrame numeric payload exceeds ABI slots");
				requireBytes(8 * numericLen, end);
				const absoluteOffset = bytes.byteOffset + offset;
				let numeric;
				if (absoluteOffset % Float64Array.BYTES_PER_ELEMENT === 0) numeric = new Float64Array(bytes.buffer, absoluteOffset, numericLen);
				else {
					numeric = new Float64Array(numericLen);
					for (let slot = 0; slot < numeric.length; slot++) numeric[slot] = view.getFloat64(offset + slot * 8, true);
				}
				offset += 8 * numericLen;
				records.push({
					kind: "node",
					flags,
					target,
					eventCode,
					eventId,
					json: "",
					numeric
				});
			} else if (payloadKind === HOST_NODE_PAYLOAD.Json) {
				requireBytes(4, end);
				const len = view.getUint32(offset, true);
				offset += 4;
				requireBytes(len, end);
				const json = textDecoder.decode(bytes.subarray(offset, offset + len));
				offset += len;
				records.push({
					kind: "node",
					flags,
					target,
					eventCode,
					eventId,
					json
				});
			} else throw new TypeError(`unknown node payload kind ${payloadKind}`);
		} else if (kind === HOST_RECORD_KIND.Resize) {
			requireBytes(16, end);
			records.push({
				kind: "resize",
				target: nodeKey(view.getUint32(offset, true), view.getUint32(offset + 4, true)),
				width: view.getFloat32(offset + 8, true),
				height: view.getFloat32(offset + 12, true)
			});
			offset += 16;
		} else if (kind === HOST_RECORD_KIND.ApplicationMessage) {
			requireBytes(2, end);
			const topicLen = view.getUint16(offset, true);
			offset += 2;
			requireBytes(topicLen + 1, end);
			const topic = textDecoder.decode(bytes.subarray(offset, offset + topicLen));
			offset += topicLen;
			const payloadKind = view.getUint8(offset++);
			let payload;
			if (payloadKind === 0) payload = null;
			else if (payloadKind === 1) {
				requireBytes(1, end);
				payload = view.getUint8(offset++) !== 0;
			} else if (payloadKind === 2) {
				requireBytes(4, end);
				payload = view.getInt32(offset, true);
				offset += 4;
			} else if (payloadKind === 3) {
				requireBytes(8, end);
				payload = view.getFloat64(offset, true);
				offset += 8;
			} else if (payloadKind === 4) {
				requireBytes(2, end);
				const len = view.getUint16(offset, true);
				offset += 2;
				requireBytes(len, end);
				payload = textDecoder.decode(bytes.subarray(offset, offset + len));
				offset += len;
			} else if (payloadKind === 5) {
				requireBytes(4, end);
				const len = view.getUint32(offset, true);
				offset += 4;
				requireBytes(len, end);
				payload = bytes.subarray(offset, offset + len).slice();
				offset += len;
			} else throw new TypeError(`unknown application payload kind ${payloadKind}`);
			records.push({
				kind: "message",
				topic,
				payload
			});
		} else records.push({ kind: "unknown" });
		if (offset > end) throw new TypeError("HostEventFrame record overflow");
		offset = end;
	}
	if (offset !== byteLen) throw new TypeError("trailing HostEventFrame bytes");
	const prevented = [];
	let needsTick = false;
	flush(() => {
		for (const record of records) if (record.kind === "node") {
			if (dispatchEvent(record.target, record.eventCode, record.json, record.numeric) && (record.flags & FLAG_CANCELLABLE) !== 0 && record.eventId !== 0) prevented.push(record.eventId);
			needsTick = true;
		} else if (record.kind === "resize") {
			dispatchResizeObservation(record.target, record.width, record.height);
			needsTick = true;
		} else if (record.kind === "message") {
			dispatchHostMessage(record.topic, record.payload);
			needsTick = true;
		}
	});
	reconcileControlledInputValues();
	const protocolFrame = writer.flush() ?? void 0;
	return {
		preventedEventIds: prevented.length > 0 ? Uint32Array.from(prevented) : void 0,
		needsTick,
		protocolFrame
	};
}
function __wabou_dispatch_host_frame(frame) {
	return decodeAndDispatchHostFrame(frame);
}
globalThis.__wabou_dispatch_host_frame = __wabou_dispatch_host_frame;
//#endregion
//#region src/glue/keyboard-modifiers.ts
const SHIFT = 1;
const CONTROL = 2;
const ALT = 4;
const META = 8;
const PRIMARY = 16;
function decodeKeyboardModifiers(value) {
	if (!Number.isInteger(value) || value < 0 || (value & -32) !== 0) throw new TypeError("keyboard modifier bits are invalid");
	const bits = value;
	return Object.freeze({
		bits: bits & 15,
		shift: (bits & SHIFT) !== 0,
		control: (bits & CONTROL) !== 0,
		alt: (bits & ALT) !== 0,
		meta: (bits & META) !== 0,
		primary: (bits & PRIMARY) !== 0
	});
}
const empty = decodeKeyboardModifiers(0);
const [keyboardModifiers, setKeyboardModifiers] = createSignal(empty, {
	equals: (previous, next) => previous.bits === next.bits && previous.primary === next.primary,
	ownedWrite: true
});
const subscribers = /* @__PURE__ */ new Set();
subscribe("wabou:keyboard-modifiers", (payload) => {
	try {
		const modifiers = decodeKeyboardModifiers(payload);
		setKeyboardModifiers(modifiers);
		for (const subscriber of subscribers) try {
			subscriber(modifiers);
		} catch (error) {
			console.error("[wabou-host] keyboard modifier subscriber threw", error);
		}
	} catch (error) {
		console.error("[wabou-host] invalid keyboard modifiers", error);
	}
});
/** Reactive, Host-authoritative physical modifier-key state. */
function useKeyboardModifiers() {
	return keyboardModifiers;
}
/** Subscribe to physical modifier changes without creating a Solid owner. */
function subscribeKeyboardModifiers(handler) {
	subscribers.add(handler);
	return () => subscribers.delete(handler);
}
/** Subscribe for the lifetime of the current Solid owner. */
function useKeyboardModifierChanges(handler) {
	onCleanup(subscribeKeyboardModifiers(handler));
}
//#endregion
//#region src/glue/platform-context.ts
const PlatformContext = createContext({});
/** Override native services for one Solid subtree, primarily for tests and previews. */
function PlatformProvider(props) {
	const parent = useContext(PlatformContext) ?? {};
	return createComponent(PlatformContext, {
		value: {
			get clipboard() {
				return props.value.clipboard ?? parent.clipboard;
			},
			get dialog() {
				return props.value.dialog ?? parent.dialog;
			},
			get notification() {
				return props.value.notification ?? parent.notification;
			},
			get window() {
				return props.value.window ?? parent.window;
			}
		},
		get children() {
			return props.children;
		}
	});
}
function usePlatformServices() {
	return getOwner() ? useContext(PlatformContext) : {};
}
//#endregion
//#region src/glue/effects.ts
const pending = /* @__PURE__ */ new Map();
function assertAbi() {
	if (__wabou_effect_abi !== 6) throw new Error(`Wabou effect ABI mismatch: bundle=6, host=${__wabou_effect_abi}`);
}
function submit(op, payload) {
	assertAbi();
	return __wabou_effect_submit(op.capability, op.method, JSON.stringify(payload ?? null));
}
function dispatchEffect(op, payload) {
	return new Promise((resolve, reject) => {
		const id = submit(op, payload);
		pending.set(id, {
			op,
			resolve,
			reject
		});
	});
}
/** Submit a command without retaining a Promise or callback. */
function dispatchFireAndForget(op, payload) {
	submit(op, payload);
}
function complete(id, capability, method, status, payloadJson) {
	const request = pending.get(id);
	if (!request) return;
	pending.delete(id);
	if (request.op.capability !== capability || request.op.method !== method) {
		request.reject(/* @__PURE__ */ new Error(`Native effect ${id} completed with the wrong operation`));
		return;
	}
	if (status === 1) {
		const error = /* @__PURE__ */ new Error("Native effect was cancelled");
		error.name = "AbortError";
		request.reject(error);
		return;
	}
	const payload = JSON.parse(payloadJson);
	if (status === 2) {
		const error = payload;
		request.reject(new Error(error.message ?? "Native effect failed"));
		return;
	}
	request.resolve(payload);
}
globalThis.__wabou_effect_complete = complete;
//#endregion
//#region src/glue/window.ts
const windowKeys = createResourceKeyFamily("window");
function windowKeyFromJSON(value) {
	return windowKeys.fromJSON(value);
}
/** Immutable native creation options for the JavaScript runtime's window. */
function currentWindowOptions() {
	const serialized = globalThis.__wabou_window_options_json;
	if (!serialized) return Object.freeze({});
	return Object.freeze(JSON.parse(serialized));
}
function handle(id) {
	return Object.freeze({
		id,
		close: () => dispatchFireAndForget(effectOps.windowClose, { windowId: id }),
		minimize: () => dispatchFireAndForget(effectOps.windowMinimize, { windowId: id }),
		setMaximized: (value) => dispatchFireAndForget(effectOps.windowSetMaximized, {
			windowId: id,
			value
		}),
		setTitle: (title) => dispatchFireAndForget(effectOps.windowSetTitle, {
			windowId: id,
			title
		}),
		startDragging: () => dispatchFireAndForget(effectOps.windowStartDragging, { windowId: id }),
		show: () => dispatchFireAndForget(effectOps.windowShow, { windowId: id })
	});
}
/** Create an independent native window running this application's bundle. */
function createWindow(options = {}) {
	return dispatchEffect(effectOps.windowCreate, options).then((key) => handle(windowKeyFromJSON(key)));
}
/** An imperative handle for the native window that owns this JS runtime. */
function currentWindow() {
	return handle(windowKeys.fromParts(__wabou_window_id_lo, __wabou_window_id_hi));
}
//#endregion
//#region src/glue/window-metrics.ts
/**
* Create a reactive native-window size query without CSS media-query semantics.
* A zero-sized pre-boot viewport never matches, avoiding a compact-layout flash.
*/
function createWindowMatch(query, window = useWindow()) {
	const entries = [
		["minWidth", query.minWidth],
		["maxWidth", query.maxWidth],
		["minHeight", query.minHeight],
		["maxHeight", query.maxHeight]
	];
	for (const [name, value] of entries) if (value !== void 0 && (!Number.isFinite(value) || value < 0)) throw new RangeError(`${name} must be a finite non-negative number`);
	if (query.minWidth !== void 0 && query.maxWidth !== void 0 && query.minWidth > query.maxWidth) throw new RangeError("minWidth cannot exceed maxWidth");
	if (query.minHeight !== void 0 && query.maxHeight !== void 0 && query.minHeight > query.maxHeight) throw new RangeError("minHeight cannot exceed maxHeight");
	return createMemo(() => {
		const width = window.width();
		const height = window.height();
		if (width <= 0 || height <= 0) return false;
		return (query.minWidth === void 0 || width >= query.minWidth) && (query.maxWidth === void 0 || width <= query.maxWidth) && (query.minHeight === void 0 || height >= query.minHeight) && (query.maxHeight === void 0 || height <= query.maxHeight);
	}, { sync: true });
}
const initial = {
	windowId: windowKeyFromJSON({
		lo: globalThis.__wabou_window_id_lo ?? 1,
		hi: globalThis.__wabou_window_id_hi ?? 1
	}),
	logicalWidth: 0,
	logicalHeight: 0,
	physicalWidth: 0,
	physicalHeight: 0,
	scaleFactor: 1,
	maximized: false,
	focused: false,
	outerX: null,
	outerY: null,
	occluded: false,
	colorScheme: "light",
	reducedMotion: false
};
function sameMetrics(previous, next) {
	return previous.windowId.lo === next.windowId.lo && previous.windowId.hi === next.windowId.hi && previous.logicalWidth === next.logicalWidth && previous.logicalHeight === next.logicalHeight && previous.physicalWidth === next.physicalWidth && previous.physicalHeight === next.physicalHeight && previous.scaleFactor === next.scaleFactor && previous.maximized === next.maximized && previous.focused === next.focused && previous.outerX === next.outerX && previous.outerY === next.outerY && previous.occluded === next.occluded && previous.colorScheme === next.colorScheme && previous.reducedMotion === next.reducedMotion;
}
const [metrics, setMetrics] = createSignal(initial, {
	equals: sameMetrics,
	ownedWrite: true
});
function decodeWindowMetrics(value) {
	if (typeof value !== "object" || value === null) throw new TypeError("window metrics must be an object");
	const next = value;
	const finiteNumber = (field) => {
		const number = next[field];
		if (typeof number !== "number" || !Number.isFinite(number)) throw new TypeError(`window metrics ${field} must be a finite number`);
		return number;
	};
	if (typeof next.maximized !== "boolean" || typeof next.focused !== "boolean" || typeof next.occluded !== "boolean" || typeof next.reducedMotion !== "boolean") throw new TypeError("window metrics flags must be booleans");
	for (const field of ["outerX", "outerY"]) if (next[field] !== null && (typeof next[field] !== "number" || !Number.isFinite(next[field]))) throw new TypeError(`window metrics ${field} must be null or a finite number`);
	if (next.colorScheme !== null && next.colorScheme !== "light" && next.colorScheme !== "dark") throw new TypeError("window metrics colorScheme is invalid");
	return {
		windowId: windowKeyFromJSON(next.windowId),
		logicalWidth: finiteNumber("logicalWidth"),
		logicalHeight: finiteNumber("logicalHeight"),
		physicalWidth: finiteNumber("physicalWidth"),
		physicalHeight: finiteNumber("physicalHeight"),
		scaleFactor: finiteNumber("scaleFactor"),
		maximized: next.maximized,
		focused: next.focused,
		outerX: next.outerX,
		outerY: next.outerY,
		occluded: next.occluded,
		colorScheme: next.colorScheme,
		reducedMotion: next.reducedMotion
	};
}
subscribeJson("wabou:window-metrics", setMetrics, { decode: decodeWindowMetrics });
const state = {
	get id() {
		return metrics().windowId;
	},
	close: () => currentWindow().close(),
	minimize: () => currentWindow().minimize(),
	setMaximized: (value) => currentWindow().setMaximized(value),
	setTitle: (title) => currentWindow().setTitle(title),
	startDragging: () => currentWindow().startDragging(),
	show: () => currentWindow().show(),
	metrics,
	width: () => metrics().logicalWidth,
	height: () => metrics().logicalHeight,
	scaleFactor: () => metrics().scaleFactor,
	maximized: () => metrics().maximized,
	focused: () => metrics().focused,
	outerX: () => metrics().outerX,
	outerY: () => metrics().outerY,
	occluded: () => metrics().occluded,
	colorScheme: () => metrics().colorScheme ?? "light",
	reducedMotion: () => metrics().reducedMotion
};
/** Reactive state and controls for the native window owning this JS runtime. */
function useWindow() {
	return usePlatformServices().window ?? state;
}
//#endregion
//#region src/glue/file-drop.ts
function decodeFileDrop(value) {
	if (typeof value !== "object" || value === null) throw new TypeError("file drop event must be an object");
	const event = value;
	if (event.phase !== "entered" && event.phase !== "moved" && event.phase !== "left" && event.phase !== "dropped") throw new TypeError("file drop event has an invalid phase");
	if (!Array.isArray(event.paths) || !event.paths.every((path) => typeof path === "string")) throw new TypeError("file drop event paths must be strings");
	const position = event.position;
	if (position !== null && (typeof position !== "object" || typeof position.x !== "number" || typeof position.y !== "number")) throw new TypeError("file drop event position must be logical coordinates");
	return {
		phase: event.phase,
		paths: event.paths,
		position: position ?? null
	};
}
/** Subscribe to native file drag-and-drop events for the current window. */
function subscribeFileDrop(handler) {
	return subscribeJson("wabou:file-drop", handler, { decode: decodeFileDrop });
}
/**
* Subscribe for the lifetime of the current Solid owner.
* Use `subscribeFileDrop` when no Solid owner is active.
*/
function useFileDrop(handler) {
	onCleanup(subscribeFileDrop(handler));
}
//#endregion
//#region src/glue/gesture.ts
function isPhase(value) {
	return value === "started" || value === "changed" || value === "ended" || value === "cancelled";
}
function decodeGesture(value) {
	if (typeof value !== "object" || value === null) throw new TypeError("gesture event must be an object");
	const event = value;
	if (event.type === "double-tap") return { type: "double-tap" };
	if (event.type === "pressure" && typeof event.pressure === "number" && typeof event.stage === "number") return {
		type: "pressure",
		pressure: event.pressure,
		stage: event.stage
	};
	if (!isPhase(event.phase)) throw new TypeError("continuous gesture event has an invalid phase");
	if ((event.type === "pinch" || event.type === "rotation") && typeof event.delta === "number") return {
		type: event.type,
		delta: event.delta,
		phase: event.phase
	};
	if (event.type === "pan" && typeof event.deltaX === "number" && typeof event.deltaY === "number") return {
		type: "pan",
		deltaX: event.deltaX,
		deltaY: event.deltaY,
		phase: event.phase
	};
	throw new TypeError("gesture event has an invalid payload");
}
/** Subscribe to native trackpad and touchscreen gestures for the current window. */
function subscribeGesture(handler) {
	return subscribeJson("wabou:gesture", handler, { decode: decodeGesture });
}
/** Subscribe for the lifetime of the current Solid owner. */
function useGesture(handler) {
	onCleanup(subscribeGesture(handler));
}
//#endregion
//#region src/glue/clipboard.ts
const clipboard = Object.freeze({
	readText: () => dispatchEffect(effectOps.clipboardRead),
	writeText: (text) => dispatchEffect(effectOps.clipboardWrite, { text: String(text) }).then(() => void 0)
});
/** Stable clipboard capability for use inside Solid components. */
function useClipboard() {
	return usePlatformServices().clipboard ?? clipboard;
}
//#endregion
//#region src/glue/app-dirs.ts
let resolved;
function resolve$1() {
	return resolved ??= dispatchEffect(effectOps.appDirsResolve);
}
/** Resolve app-private native roots, caching the host result for this runtime. */
const appDirs = Object.freeze({
	resolve: resolve$1,
	config: () => resolve$1().then((paths) => paths.configDir),
	data: () => resolve$1().then((paths) => paths.dataDir),
	localData: () => resolve$1().then((paths) => paths.localDataDir),
	cache: () => resolve$1().then((paths) => paths.cacheDir),
	log: () => resolve$1().then((paths) => paths.logDir),
	resource: () => resolve$1().then((paths) => paths.resourceDir),
	temp: () => resolve$1().then((paths) => paths.tempDir)
});
//#endregion
//#region src/glue/application.ts
const application = Object.freeze({
	exit: () => dispatchFireAndForget(effectOps.applicationExit),
	relaunch: () => dispatchFireAndForget(effectOps.applicationRelaunch)
});
//#endregion
//#region src/glue/dialog.ts
function normalizeFilters(filters) {
	return (filters ?? []).map((filter) => ({
		name: String(filter.name),
		extensions: filter.extensions.map((extension) => String(extension).replace(/^\./, "")).filter(Boolean)
	}));
}
const dialog = Object.freeze({
	open(options = {}) {
		return dispatchEffect(effectOps.dialogOpen, {
			...options,
			filters: normalizeFilters(options.filters),
			multiple: options.multiple ?? false
		});
	},
	save(options = {}) {
		return dispatchEffect(effectOps.dialogSave, {
			...options,
			filters: normalizeFilters(options.filters)
		}).then((paths) => paths?.[0] ?? null);
	},
	pickDirectory(options = {}) {
		return dispatchEffect(effectOps.dialogPickDirectory, options).then((paths) => paths?.[0] ?? null);
	},
	message(options) {
		return dispatchEffect(effectOps.dialogMessage, {
			...options,
			message: String(options.message),
			level: options.level ?? "info",
			buttons: options.buttons ?? "ok"
		});
	}
});
function useDialog() {
	return usePlatformServices().dialog ?? dialog;
}
//#endregion
//#region src/glue/notification.ts
const notification = Object.freeze({ show(options) {
	return dispatchEffect(effectOps.notificationShow, {
		...options,
		title: String(options.title),
		silent: options.silent ?? false
	}).then(() => void 0);
} });
function useNotification() {
	return usePlatformServices().notification ?? notification;
}
//#endregion
//#region src/glue/intl.ts
/**
* Operating-system locale facts. Standards-compatible formatting is installed
* separately by the FormatJS-backed Intl polyfill.
*/
const intl = Object.freeze({
	locale() {
		return defaultHost.intl.locale();
	},
	timeZone() {
		return defaultHost.intl.timeZone();
	},
	today() {
		return defaultHost.intl.today();
	}
});
//#endregion
//#region src/glue/async-action.ts
/** A concurrent call tried to replace the arguments of an in-flight action. */
var AsyncActionConflictError = class extends Error {
	constructor() {
		super("async action is already running with different arguments; use a keyed action for independent operations");
		this.name = "AsyncActionConflictError";
	}
};
/**
* Run an imperative async operation as a single flight with explicit state.
* Repeated calls with the same argument identities join the pending operation.
* A call with different arguments returns [`AsyncActionConflictError`] rather
* than silently discarding those arguments. Use `createKeyedAsyncAction` when
* independently keyed operations should run concurrently.
*/
function createAsyncAction(action) {
	const [pending, setPending] = createSignal(false);
	const [pendingArgs, setPendingArgs] = createSignal();
	const [error, setError] = createSignal();
	let disposed = false;
	let inFlight;
	let inFlightArgs;
	const run = (...args) => {
		if (disposed) return Promise.resolve({
			ok: false,
			error: /* @__PURE__ */ new Error("cannot run a disposed async action")
		});
		if (inFlight) {
			if (sameArguments(inFlightArgs, args)) return inFlight;
			return Promise.resolve({
				ok: false,
				error: new AsyncActionConflictError()
			});
		}
		setPending(true);
		setPendingArgs(() => args);
		setError(void 0);
		inFlightArgs = args;
		let resolveRequest;
		const request = new Promise((resolve) => {
			resolveRequest = resolve;
		});
		inFlight = request;
		let outcome;
		try {
			outcome = action(...args);
		} catch (cause) {
			outcome = Promise.reject(cause);
		}
		Promise.resolve(outcome).then((value) => settle({
			ok: true,
			value
		}), (cause) => settle({
			ok: false,
			error: cause
		}));
		return request;
		function settle(result) {
			if (!result.ok && !disposed) setError(result.error);
			inFlight = void 0;
			inFlightArgs = void 0;
			if (!disposed) {
				setPending(false);
				setPendingArgs(void 0);
			}
			resolveRequest(result);
		}
	};
	const reset = () => {
		if (!disposed) setError(void 0);
	};
	if (getOwner()) onCleanup(() => {
		disposed = true;
		setPending(false);
		setPendingArgs(void 0);
		setError(void 0);
	});
	return {
		pending,
		pendingArgs,
		error,
		run,
		reset
	};
}
function sameArguments(previous, next) {
	return previous !== void 0 && previous.length === next.length && previous.every((value, index) => Object.is(value, next[index]));
}
/**
* Run one async single-flight per stable key. Calls for the same key join the
* existing operation, while unrelated keys remain independently concurrent.
*/
function createKeyedAsyncAction(keyOf, action) {
	const [pendingKeys, setPendingKeys] = createSignal(/* @__PURE__ */ new Set());
	const [errors, setErrors] = createSignal(/* @__PURE__ */ new Map());
	const inFlight = /* @__PURE__ */ new Map();
	let disposed = false;
	const run = (...args) => {
		if (disposed) return Promise.resolve({
			ok: false,
			error: /* @__PURE__ */ new Error("cannot run a disposed keyed async action")
		});
		let key;
		try {
			key = keyOf(...args);
		} catch (error) {
			return Promise.resolve({
				ok: false,
				error
			});
		}
		const existing = inFlight.get(key);
		if (existing) return existing;
		setPendingKeys((current) => /* @__PURE__ */ new Set([...current, key]));
		setErrors((current) => {
			if (!current.has(key)) return current;
			const next = new Map(current);
			next.delete(key);
			return next;
		});
		let resolveRequest;
		const request = new Promise((resolve) => {
			resolveRequest = resolve;
		});
		inFlight.set(key, request);
		let outcome;
		try {
			outcome = action(...args);
		} catch (cause) {
			outcome = Promise.reject(cause);
		}
		Promise.resolve(outcome).then((value) => settle({
			ok: true,
			value
		}), (cause) => settle({
			ok: false,
			error: cause
		}));
		return request;
		function settle(result) {
			if (!result.ok && !disposed) setErrors((current) => new Map(current).set(key, result.error));
			if (inFlight.get(key) === request) inFlight.delete(key);
			if (!disposed) setPendingKeys((current) => {
				if (!current.has(key)) return current;
				const next = new Set(current);
				next.delete(key);
				return next;
			});
			resolveRequest(result);
		}
	};
	const reset = (key) => {
		if (disposed) return;
		setErrors((current) => {
			if (!current.has(key)) return current;
			const next = new Map(current);
			next.delete(key);
			return next;
		});
	};
	const resetAll = () => {
		if (!disposed) setErrors(/* @__PURE__ */ new Map());
	};
	if (getOwner()) onCleanup(() => {
		disposed = true;
		inFlight.clear();
		setPendingKeys(/* @__PURE__ */ new Set());
		setErrors(/* @__PURE__ */ new Map());
	});
	return {
		pendingKeys,
		pending: (key) => pendingKeys().has(key),
		error: (key) => errors().get(key),
		run,
		reset,
		resetAll
	};
}
//#endregion
//#region src/glue/async-query.ts
/**
* Create a latest-wins query using Solid 2's native async graph.
*
* Promise ownership, stale-result suppression, pending propagation, and error
* propagation belong to Solid. Wabou only adds AbortSignal lifecycle and an
* explicit refresh operation.
*/
function createAsyncQuery(options) {
	let controller;
	const value = createMemo(() => {
		const key = options.source();
		controller?.abort();
		controller = void 0;
		if (key === void 0) return options.initialValue;
		controller = new AbortController();
		return options.load(key, { signal: controller.signal });
	});
	const latestValue = createMemo(() => latest(value), { loadingValue: options.initialValue });
	onCleanup(() => controller?.abort());
	return {
		value,
		latest: latestValue,
		async refresh() {
			refresh(value);
			return resolve(value);
		}
	};
}
//#endregion
//#region src/glue/color-theme.tsx
const [current, setCurrent] = createSignal();
let currentPalette;
let activeAnimation;
function paletteFor(name) {
	if (!name) throw new Error("Wabou color theme name cannot be empty");
	let palette;
	try {
		const length = globalThis.__wabou_get_color_theme_palette(name, void 0);
		if (!Number.isSafeInteger(length) || length < 0) throw new TypeError("invalid palette length");
		const output = new Uint32Array(length);
		if (globalThis.__wabou_get_color_theme_palette(name, output) !== length) throw new TypeError("palette length changed");
		palette = output;
	} catch {
		throw new Error(`Unknown Wabou color theme \`${name}\`; declare it in the \`theme.themes\` section of vite.config.ts`);
	}
	if (!(palette instanceof Uint32Array)) throw new Error(`Wabou color theme \`${name}\` returned an invalid palette`);
	return palette;
}
function easingFunction(easing) {
	if (typeof easing === "function") return easing;
	switch (easing) {
		case "linear": return (t) => t;
		case "ease-in": return (t) => t * t * t;
		case "ease-in-out": return (t) => t < .5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2;
		default: return (t) => 1 - (1 - t) ** 3;
	}
}
function srgbToLinear(value) {
	return value <= .04045 ? value / 12.92 : ((value + .055) / 1.055) ** 2.4;
}
function linearToSrgb(value) {
	return value <= .0031308 ? 12.92 * value : 1.055 * value ** (1 / 2.4) - .055;
}
function srgbToOklab(red, green, blue) {
	const r = srgbToLinear(red);
	const g = srgbToLinear(green);
	const b = srgbToLinear(blue);
	const l = Math.cbrt(.4122214708 * r + .5363325363 * g + .0514459929 * b);
	const m = Math.cbrt(.2119034982 * r + .6806995451 * g + .1073969566 * b);
	const s = Math.cbrt(.0883024619 * r + .2817188376 * g + .6299787005 * b);
	return [
		.2104542553 * l + .793617785 * m - .0040720468 * s,
		1.9779984951 * l - 2.428592205 * m + .4505937099 * s,
		.0259040371 * l + .7827717662 * m - .808675766 * s
	];
}
function oklabToSrgb(lightness, a, b) {
	const l = (lightness + .3963377774 * a + .2158037573 * b) ** 3;
	const m = (lightness - .1055613458 * a - .0638541728 * b) ** 3;
	const s = (lightness - .0894841775 * a - 1.291485548 * b) ** 3;
	return [
		linearToSrgb(4.0767416621 * l - 3.3077115913 * m + .2309699292 * s),
		linearToSrgb(-1.2684380046 * l + 2.6097574011 * m - .3413193965 * s),
		linearToSrgb(-.0041960863 * l - .7034186147 * m + 1.707614701 * s)
	];
}
function channel(value) {
	return Math.round(Math.min(1, Math.max(0, value)) * 255);
}
function mixColor(from, to, progress, oklab) {
	const fromRgb = [
		(from >>> 24 & 255) / 255,
		(from >>> 16 & 255) / 255,
		(from >>> 8 & 255) / 255
	];
	const toRgb = [
		(to >>> 24 & 255) / 255,
		(to >>> 16 & 255) / 255,
		(to >>> 8 & 255) / 255
	];
	const fromValue = oklab ? srgbToOklab(...fromRgb) : fromRgb;
	const toValue = oklab ? srgbToOklab(...toRgb) : toRgb;
	const mixed = [
		fromValue[0] + (toValue[0] - fromValue[0]) * progress,
		fromValue[1] + (toValue[1] - fromValue[1]) * progress,
		fromValue[2] + (toValue[2] - fromValue[2]) * progress
	];
	const [red, green, blue] = oklab ? oklabToSrgb(...mixed) : mixed;
	const alpha = channel(((from & 255) + ((to & 255) - (from & 255)) * progress) / 255);
	return (channel(red) << 24 | channel(green) << 16 | channel(blue) << 8 | alpha) >>> 0;
}
function submitPalette(colors) {
	currentPalette = colors;
	globalThis.__wabou_set_color_palette(colors);
}
const colorTheme = {
	current,
	set(name) {
		activeAnimation?.cancel();
		const palette = paletteFor(name);
		globalThis.__wabou_set_color_theme(name);
		currentPalette = palette;
		setCurrent(name);
	},
	getPalette: paletteFor,
	setPalette(colors) {
		if (!(colors instanceof Uint32Array)) throw new TypeError("Wabou color palette must be a Uint32Array");
		submitPalette(colors.slice());
	},
	animateTo(name, options = {}) {
		const target = paletteFor(name);
		const source = currentPalette?.slice();
		if (!source || options.duration === 0) {
			this.set(name);
			return {
				finished: Promise.resolve(),
				cancel() {}
			};
		}
		if (source.length !== target.length) throw new Error("Wabou color theme palettes have inconsistent lengths");
		activeAnimation?.cancel();
		const durationMs = Math.max(0, options.duration ?? .28) * 1e3;
		const ease = easingFunction(options.easing);
		const frame = new Uint32Array(source.length);
		let raf = 0;
		let start;
		let settled = false;
		let finish;
		const controls = {
			finished: new Promise((resolve) => {
				finish = resolve;
			}),
			cancel() {
				if (settled) return;
				settled = true;
				cancelAnimationFrame(raf);
				finish();
			}
		};
		activeAnimation = controls;
		const tick = (timestamp) => {
			if (settled) return;
			start ??= timestamp;
			const linear = durationMs === 0 ? 1 : Math.min(1, (timestamp - start) / durationMs);
			const progress = Math.min(1, Math.max(0, ease(linear)));
			for (let index = 0; index < frame.length; index++) frame[index] = mixColor(source[index], target[index], progress, options.colorSpace !== "srgb");
			submitPalette(frame.slice());
			if (linear < 1) {
				raf = requestAnimationFrame(tick);
				return;
			}
			settled = true;
			currentPalette = target;
			globalThis.__wabou_set_color_theme(name);
			setCurrent(name);
			if (activeAnimation === controls) activeAnimation = void 0;
			finish();
		};
		raf = requestAnimationFrame(tick);
		return controls;
	}
};
const ColorThemeContext = createContext(colorTheme);
/** Selects one compiled color palette for the current native window. */
function ColorThemeProvider(props) {
	let initialized = false;
	createEffect(() => [props.theme, props.transition], ([theme, transition]) => {
		const animation = initialized && transition ? colorTheme.animateTo(theme, transition) : (colorTheme.set(theme), void 0);
		initialized = true;
		return animation ? () => animation.cancel() : void 0;
	});
	return createComponent(ColorThemeContext, {
		value: colorTheme,
		get children() {
			return props.children;
		}
	});
}
function useColorTheme() {
	return useContext(ColorThemeContext);
}
//#endregion
//#region src/glue/entity-list.tsx
function validateEntityKeys(values, by) {
	const keys = /* @__PURE__ */ new Set();
	for (const entity of values) {
		const key = by(entity);
		if (keys.has(key)) throw new Error(`ForEntity received duplicate key ${String(key)}`);
		keys.add(key);
	}
	return values;
}
/**
* Render stateful entities by a stable application key.
*
* The entity object itself is part of the identity contract: mutate its
* internal signals/stores instead of replacing it with a new snapshot carrying
* the same key. This keeps native widgets and other owned resources mounted.
*/
function ForEntity(props) {
	const by = untrack(() => props.by);
	const entities = createMemo(() => {
		const values = props.each;
		if (!values) return values;
		return validateEntityKeys(values, by);
	});
	return createComponent(For, {
		get each() {
			return entities();
		},
		keyed: by,
		get fallback() {
			return props.fallback;
		},
		children: (item, index) => {
			const entity = untrack(item);
			const key = by(entity);
			createEffect(item, (current) => {
				if (current !== entity) throw new Error(`ForEntity key ${String(key)} replaced its entity object; keep the object stable and update its signals/store instead`);
			});
			return props.children(entity, index);
		}
	});
}
//#endregion
//#region src/glue/event-effect.ts
/**
* Consume every new event from a retained feed exactly once and in sequence
* order. This avoids losing events when several feed updates are batched into
* one reactive notification.
*/
function createEventEffect(options) {
	const initial = untrack(options.source);
	let cursor = options.consumeInitial ? Number.NEGATIVE_INFINITY : latestSequence(initial, options.sequence);
	createEffect(options.source, (events) => {
		const pending = events.map((event) => ({
			event,
			sequence: options.sequence(event)
		})).filter((candidate) => candidate.sequence > cursor).sort((left, right) => left.sequence - right.sequence);
		for (const candidate of pending) {
			if (candidate.sequence <= cursor) continue;
			cursor = candidate.sequence;
			try {
				const result = options.onEvent(candidate.event);
				if (isPromiseLike(result)) Promise.resolve(result).catch((error) => reportError(options, error, candidate.event, candidate.sequence));
			} catch (error) {
				reportError(options, error, candidate.event, candidate.sequence);
			}
		}
	});
}
function isPromiseLike(value) {
	return (typeof value === "object" || typeof value === "function") && value !== null && typeof value.then === "function";
}
function reportError(options, error, event, sequence) {
	if (options.onError) try {
		options.onError(error, event);
		return;
	} catch (reportingError) {
		console.error(`[wabou-event-effect] onError failed for sequence ${sequence}`, reportingError);
	}
	console.error(`[wabou-event-effect] handler failed for sequence ${sequence}`, error);
}
function latestSequence(events, sequence) {
	let latest = Number.NEGATIVE_INFINITY;
	for (const event of events) latest = Math.max(latest, sequence(event));
	return latest;
}
//#endregion
//#region src/glue/host-resource.ts
var RevisionedHostWaitError = class extends Error {
	reason;
	constructor(reason, message) {
		super(message);
		this.name = "RevisionedHostWaitError";
		this.reason = reason;
	}
};
/**
* Keep a Solid value synchronized with a host-owned revisioned snapshot.
*
* A revision identifies the exact snapshot contents. After the first host
* value, producers must increase it whenever those contents can change;
* another payload with the same revision is treated as a duplicate.
*
* The initial RPC closes the subscription race by ignoring results older than
* an already received host push. A patch whose base revision no longer
* matches automatically falls back to one coalesced full refresh.
*/
function createRevisionedHostResource(options) {
	const [value, setValue] = createSignal(options.initial, { equals: false });
	const [loading, setLoading] = createSignal(false);
	const [error, setError] = createSignal();
	let disposed = false;
	let refreshPromise;
	let hostGeneration = 0;
	let hasAcceptedHostValue = false;
	const waiters = /* @__PURE__ */ new Set();
	const removeWaiter = (waiter) => {
		waiters.delete(waiter);
		if (waiter.timer !== void 0) clearTimeout(waiter.timer);
		if (waiter.signal && waiter.onAbort) waiter.signal.removeEventListener("abort", waiter.onAbort);
	};
	const rejectWaiter = (waiter, error) => {
		removeWaiter(waiter);
		waiter.reject(error);
	};
	const notifyWaiters = (next) => {
		for (const waiter of [...waiters]) {
			let matches;
			try {
				matches = waiter.predicate(next);
			} catch (cause) {
				rejectWaiter(waiter, cause instanceof Error ? cause : new Error(String(cause)));
				continue;
			}
			if (matches) {
				removeWaiter(waiter);
				waiter.resolve(next);
			}
		}
	};
	const reportError = (next, source) => {
		if (disposed) return;
		setError(next);
		options.onError?.(next, source);
	};
	const accept = (next, source) => {
		if (disposed || next.revision < value().revision) return void 0;
		if (hasAcceptedHostValue && next.revision === value().revision) {
			setError(void 0);
			return value();
		}
		hasAcceptedHostValue = true;
		setError(void 0);
		setValue(next);
		options.onValue?.(next, source);
		notifyWaiters(next);
		return next;
	};
	const waitForPush = (predicate, waitOptions = {}) => {
		if (disposed) return Promise.reject(new RevisionedHostWaitError("disposed", "revisioned host resource is disposed"));
		let matches;
		try {
			matches = predicate(value());
		} catch (cause) {
			return Promise.reject(cause instanceof Error ? cause : new Error(String(cause)));
		}
		if (matches) return Promise.resolve(value());
		if (waitOptions.signal?.aborted) return Promise.reject(new RevisionedHostWaitError("aborted", "revisioned host resource wait aborted"));
		return new Promise((resolve, reject) => {
			const waiter = {
				predicate,
				resolve,
				reject,
				signal: waitOptions.signal
			};
			if (waitOptions.timeout !== void 0) waiter.timer = setTimeout(() => rejectWaiter(waiter, new RevisionedHostWaitError("timeout", `revisioned host resource wait timed out after ${waitOptions.timeout}ms`)), Math.max(0, waitOptions.timeout));
			if (waitOptions.signal) {
				waiter.onAbort = () => rejectWaiter(waiter, new RevisionedHostWaitError("aborted", "revisioned host resource wait aborted"));
				waitOptions.signal.addEventListener("abort", waiter.onAbort, { once: true });
			}
			waiters.add(waiter);
		});
	};
	const refresh = () => {
		if (disposed) return Promise.resolve(void 0);
		if (refreshPromise) return refreshPromise;
		const generationAtStart = hostGeneration;
		setLoading(true);
		refreshPromise = options.load().then((next) => {
			if (hostGeneration !== generationAtStart && next.revision <= value().revision) return void 0;
			return accept(next, "load");
		}).catch((cause) => {
			reportError(cause, "load");
			throw cause;
		}).finally(() => {
			refreshPromise = void 0;
			if (!disposed) setLoading(false);
		});
		return refreshPromise;
	};
	const waitFor = async (predicate, waitOptions = {}) => {
		try {
			return await waitForPush(predicate, waitOptions);
		} catch (cause) {
			if (!(cause instanceof RevisionedHostWaitError) || cause.reason !== "timeout" || !waitOptions.refreshOnTimeout) throw cause;
			if (waitOptions.signal?.aborted) throw new RevisionedHostWaitError("aborted", "revisioned host resource wait aborted");
			const existingRefresh = refreshPromise;
			if (existingRefresh) {
				try {
					await existingRefresh;
				} catch {}
				if (predicate(value())) return value();
				if (waitOptions.signal?.aborted) throw new RevisionedHostWaitError("aborted", "revisioned host resource wait aborted");
			}
			const current = await refresh() ?? value();
			if (predicate(current)) return current;
			throw cause;
		}
	};
	const unsubscribers = [subscribeJson(options.snapshotTopic, (next) => {
		hostGeneration++;
		accept(next, "snapshot");
	}, {
		decode: options.decodeSnapshot,
		onError: (cause) => reportError(cause, "snapshot")
	})];
	const applyPatch = options.applyPatch;
	if (options.patchTopic && applyPatch) unsubscribers.push(subscribeJson(options.patchTopic, (patch) => {
		hostGeneration++;
		if (patch.baseRevision !== value().revision) {
			refresh().catch(() => void 0);
			return;
		}
		try {
			const next = applyPatch(value(), patch);
			if (next) accept(next, "patch");
			else refresh().catch(() => void 0);
		} catch (cause) {
			reportError(cause, "patch");
			refresh().catch(() => void 0);
		}
	}, {
		decode: options.decodePatch,
		onError: (cause) => reportError(cause, "patch")
	}));
	const dispose = () => {
		if (disposed) return;
		disposed = true;
		for (const unsubscribe of unsubscribers) unsubscribe();
		for (const waiter of [...waiters]) rejectWaiter(waiter, new RevisionedHostWaitError("disposed", "revisioned host resource is disposed"));
	};
	if (getOwner()) onCleanup(dispose);
	if (options.autoLoad !== false) queueMicrotask(() => void refresh().catch(() => void 0));
	return {
		value,
		loading,
		error,
		refresh,
		waitFor,
		dispose
	};
}
//#endregion
//#region src/glue/native-capability.ts
var CapabilityError = class extends Error {
	code;
	constructor(message, code = "capability_unavailable") {
		super(message);
		this.name = "CapabilityError";
		this.code = code;
	}
};
/** Validate and expose one versioned native capability namespace. */
function bindCapability(capability, options) {
	if (capability?.__wabouCapabilityVersion !== options.version) throw new CapabilityError(`The native ${options.name} capability version ${options.version} is unavailable`);
	return capability;
}
//#endregion
//#region src/glue/json-capability.ts
/** Bind Wabou's versioned JSON capability transport to a typed app wrapper. */
function bindJsonCapability(capability, options) {
	return async (method, request) => {
		if (capability?.__wabouCapabilityVersion !== options.version) throw new CapabilityError(`The native ${options.name} capability version ${options.version} is unavailable`, "capability_unavailable");
		const functionValue = capability[method];
		if (typeof functionValue !== "function") throw new CapabilityError(`The native ${options.name}.${method} method is unavailable`, "method_unavailable");
		const raw = await (request === void 0 ? functionValue.call(capability) : functionValue.call(capability, JSON.stringify(request)));
		if (typeof raw !== "string") throw new CapabilityError(`The native ${options.name}.${method} method returned a non-string response`, "invalid_response");
		let envelope;
		try {
			envelope = JSON.parse(raw);
		} catch {
			throw new CapabilityError(`The native ${options.name}.${method} method returned invalid JSON`, "invalid_response");
		}
		if (typeof envelope !== "object" || envelope === null || !("ok" in envelope)) throw new CapabilityError(`The native ${options.name}.${method} method returned an invalid response envelope`, "invalid_response");
		if (envelope.ok === true) {
			if (!("value" in envelope)) throw new CapabilityError(`The native ${options.name}.${method} method returned a success envelope without a value`, "invalid_response");
			return envelope.value;
		}
		const error = envelope.error;
		const code = typeof error?.code === "string" ? error.code : void 0;
		throw new CapabilityError(typeof error?.message === "string" ? error.message : `${options.name}.${method} failed`, code ?? "handlerFailure");
	};
}
//#endregion
//#region src/glue/kv.ts
/** Fluent optimistic transaction committed as one SQLite transaction. */
var KvAtomicOperation = class {
	#prefix;
	#native;
	#checks = [];
	#mutations = [];
	constructor(prefix, native) {
		this.#prefix = prefix;
		this.#native = native;
	}
	check(check) {
		this.#checks.push({
			key: encodeKey(scopedKey(this.#prefix, check.key)),
			versionstamp: check.versionstamp
		});
		return this;
	}
	set(key, value, options = {}) {
		this.#mutations.push({
			type: "set",
			key: encodeKey(scopedKey(this.#prefix, key)),
			value,
			...encodeExpiry(options)
		});
		return this;
	}
	delete(key) {
		this.#mutations.push({
			type: "delete",
			key: encodeKey(scopedKey(this.#prefix, key))
		});
		return this;
	}
	async commit() {
		const result = await this.#native.atomic({
			checks: this.#checks,
			mutations: this.#mutations
		});
		return {
			committed: result.committed,
			...result.versionstamp === null ? {} : { versionstamp: result.versionstamp }
		};
	}
};
/**
* Bind one explicit KV key to Solid state.
*
* The key is deliberately required: source location, signal creation order,
* and variable names are not stable persistence identities across HMR or
* refactors.
*/
function createKvSignal(options) {
	const [value, setValue] = createSignal(options.initial, { ownedWrite: true });
	const [ready, setReady] = createSignal(false);
	const [error, setError] = createSignal();
	let generation = 0;
	let pending;
	let timer;
	let writer;
	const reload = async () => {
		const startedAt = generation;
		try {
			const entry = await options.kv.get(options.key);
			if (generation === startedAt && entry !== null) setValue(() => entry.value);
			setError(void 0);
		} catch (cause) {
			setError(cause);
		} finally {
			setReady(true);
		}
	};
	const drain = async () => {
		while (pending !== void 0) {
			const next = pending;
			pending = void 0;
			try {
				await options.kv.set(options.key, next);
				setError(void 0);
			} catch (cause) {
				if (pending === void 0) pending = next;
				setError(cause);
				throw cause;
			}
		}
	};
	const flush = () => {
		if (timer !== void 0) clearTimeout(timer);
		timer = void 0;
		if (writer) return writer;
		writer = drain().finally(() => {
			writer = void 0;
		});
		return writer;
	};
	const set = (next) => {
		const resolved = typeof next === "function" ? next(value()) : next;
		generation += 1;
		setValue(() => resolved);
		pending = resolved;
		if (timer !== void 0) clearTimeout(timer);
		timer = setTimeout(() => void flush().catch(() => {}), options.saveDelayMs ?? 150);
	};
	reload();
	onCleanup(() => {
		flush().catch(() => {});
	});
	return {
		value,
		ready,
		error,
		set,
		reload,
		flush
	};
}
/**
* Open a namespaced view of the host's SQLite store.
*
* The host must opt in with `HostBuilder::kv()` and configure stable app
* directories. Prefixes are prepended by whole key parts, never string joined.
*/
function openKv(prefix = []) {
	const native = bindCapability(useHost().kv, {
		name: "kv",
		version: 1
	});
	const namespace = [...prefix];
	for (const part of namespace) encodePart(part);
	return {
		async get(key) {
			const entry = await native.get({ key: encodeKey(scopedKey(namespace, key)) });
			return entry === null ? null : decodeEntry(entry, namespace.length);
		},
		async set(key, value, options = {}) {
			return (await native.set({
				key: encodeKey(scopedKey(namespace, key)),
				value,
				...encodeExpiry(options)
			})).versionstamp;
		},
		async delete(key) {
			return (await native.delete({ key: encodeKey(scopedKey(namespace, key)) })).versionstamp;
		},
		async *list(options = {}) {
			const limit = options.limit ?? 100;
			if (!Number.isSafeInteger(limit) || limit < 0) throw new RangeError("KV list limit must be a non-negative safe integer");
			const entries = await native.list({
				prefix: encodeKey([...namespace, ...options.prefix ?? []], true),
				limit,
				reverse: options.reverse ?? false
			});
			for (const entry of entries) yield decodeEntry(entry, namespace.length);
		},
		atomic: () => new KvAtomicOperation(namespace, native)
	};
}
function scopedKey(prefix, key) {
	const scoped = [...prefix, ...key];
	if (scoped.length === 0) throw new TypeError("KV keys must contain at least one part");
	return scoped;
}
function encodeKey(key, allowEmpty = false) {
	if (!allowEmpty && key.length === 0) throw new TypeError("KV keys must contain at least one part");
	return key.map(encodePart);
}
function encodePart(part) {
	if (typeof part === "string") return {
		type: "string",
		value: part
	};
	if (typeof part === "boolean") return {
		type: "bool",
		value: part
	};
	if (typeof part === "number") {
		if (!Number.isSafeInteger(part)) throw new RangeError("numeric KV key parts must be safe integers");
		return {
			type: "i64",
			value: String(part)
		};
	}
	if (part instanceof Uint8Array) return {
		type: "bytes",
		value: Array.from(part)
	};
	throw new TypeError("unsupported KV key part");
}
function decodePart(part) {
	switch (part.type) {
		case "string":
		case "bool": return part.value;
		case "i64": {
			const value = Number(part.value);
			if (!Number.isSafeInteger(value)) throw new RangeError(`KV integer ${part.value} is not safe in JavaScript`);
			return value;
		}
		case "bytes": return Uint8Array.from(part.value);
	}
}
function decodeEntry(entry, prefixLength) {
	return {
		key: entry.key.slice(prefixLength).map(decodePart),
		value: entry.value,
		versionstamp: entry.versionstamp,
		...entry.expiresAt === null ? {} : { expiresAt: entry.expiresAt }
	};
}
function encodeExpiry(options) {
	if (options.expireIn === void 0) return {};
	if (!Number.isSafeInteger(options.expireIn) || options.expireIn < 0) throw new RangeError("KV expireIn must be a non-negative safe integer");
	return { expireIn: options.expireIn };
}
//#endregion
//#region src/glue/latest-async-resource.ts
/**
* Load the latest reactive key while exposing ordinary, non-suspending state.
* Older requests are aborted when possible and can never overwrite newer data.
*/
function createLatestAsyncResource(options) {
	const initialBox = Object.hasOwn(options, "initialValue") ? { value: options.initialValue } : void 0;
	const [valueBox, setValueBox] = createSignal(initialBox);
	const [loading, setLoading] = createSignal(false);
	const [error, setError] = createSignal();
	const [status, setStatus] = createSignal("idle");
	let currentKey;
	let generation = 0;
	let controller;
	let disposed = false;
	const refresh = async () => {
		const key = currentKey;
		if (disposed || key === void 0) return void 0;
		const request = ++generation;
		controller?.abort();
		controller = new AbortController();
		const signal = controller.signal;
		setLoading(true);
		setError(void 0);
		setStatus("pending");
		try {
			const loaded = options.load(key, { signal });
			const next = loaded !== null && (typeof loaded === "object" || typeof loaded === "function") && typeof loaded.then === "function" ? await loaded : loaded;
			if (disposed || request !== generation) return void 0;
			flush(() => {
				options.onCommit?.(next);
				setValueBox({ value: next });
				setStatus("ready");
			});
			return next;
		} catch (cause) {
			if (disposed || request !== generation || signal.aborted) return void 0;
			flush(() => {
				setError(cause);
				setStatus("error");
			});
			return;
		} finally {
			if (!disposed && request === generation) {
				controller = void 0;
				flush(() => setLoading(false));
			}
		}
	};
	createEffect(options.source, (key) => {
		if (Object.is(key, currentKey)) return;
		generation++;
		controller?.abort();
		controller = void 0;
		currentKey = key;
		setError(void 0);
		setLoading(false);
		setStatus("idle");
		if (!options.retainPrevious) setValueBox(initialBox);
		if (key !== void 0 && options.autoLoad !== false) refresh();
	});
	const mutate = (next) => {
		if (disposed) return;
		generation++;
		controller?.abort();
		controller = void 0;
		options.onCommit?.(next);
		setValueBox({ value: next });
		setError(void 0);
		setLoading(false);
		setStatus("ready");
	};
	const dispose = () => {
		if (disposed) return;
		disposed = true;
		generation++;
		controller?.abort();
		controller = void 0;
	};
	if (getOwner()) onCleanup(dispose);
	return {
		value: () => valueBox()?.value,
		loading,
		error,
		status,
		refresh,
		mutate,
		dispose
	};
}
//#endregion
//#region src/glue/native-menu.ts
/** Show a platform context menu and resolve with the selected item id. */
function showNativeMenu(options) {
	return dispatchEffect(effectOps.contextMenuShow, {
		windowId: options.windowId ?? currentWindow().id,
		position: options.position,
		items: options.items
	});
}
//#endregion
//#region src/keyed-list.ts
/**
* Reconcile a host-owned keyed list while validating its complete order.
* Returns `undefined` for duplicate, missing, or unaccounted-for keys so the
* caller can request a full snapshot instead of accepting divergent state.
*/
function reconcileKeyedList(current, patch, keyOf) {
	const values = new Map(current.map((value) => [keyOf(value), value]));
	for (const key of patch.removed) values.delete(key);
	for (const value of patch.upserted) values.set(keyOf(value), value);
	if (patch.order.length !== values.size) return void 0;
	const seen = /* @__PURE__ */ new Set();
	const ordered = [];
	for (const key of patch.order) {
		if (seen.has(key)) return void 0;
		const value = values.get(key);
		if (value === void 0) return void 0;
		seen.add(key);
		ordered.push(value);
	}
	return ordered;
}
//#endregion
export { AsyncActionConflictError, CapabilityError, ColorThemeProvider, Dynamic, ForEntity, HostProvider, INLINE_STYLE_CONTRACT, KvAtomicOperation, PathBuilder, PlatformProvider, Portal, RevisionedHostWaitError, STYLE_VALUE, StyleValueKind, VirtualList, appDirs, application, assertInlineStyleValue, auto, bindCapability, bindJsonCapability, bool, classes, clipboard, colorTheme, createAsyncAction, createAsyncQuery, createEventEffect, createFps, createKeyedAsyncAction, createKvSignal, createLatestAsyncResource, createRevisionedHostResource, createWindow, createWindowMatch, currentWindow, currentWindowOptions, defaultHost, dialog, hostMessages, intl, isDirectEvent, isTypedStyleValue, isVectorPath, mergeClasses, mount, notification, number, observeGlobalPointerEvent, openKv, percent, px, reconcileKeyedList, rgba, rotate2d, scale2d, setTransform2D, shadow, showNativeMenu, subscribeAll as subscribeAllHostMessages, subscribeAppLifecycle, subscribeFileDrop, subscribeGesture, subscribe as subscribeHostMessages, subscribeJson as subscribeJsonHostMessages, subscribeKeyboardModifiers, translate2d, useAppLifecycle, useClipboard, useColorTheme, useDialog, useFileDrop, useGesture, useHost, useKeyboardModifierChanges, useKeyboardModifiers, useNotification, useWindow, utilityConflictProperties, validateEntityKeys };

//# sourceMappingURL=index.mjs.map