import { n as installMissingGlobals } from "./abort-controller-rU77nrtL.mjs";
import { ByteLengthQueuingStrategy, CountQueuingStrategy, ReadableByteStreamController, ReadableStream as ReadableStream$1, ReadableStreamBYOBReader, ReadableStreamBYOBRequest, ReadableStreamDefaultController, ReadableStreamDefaultReader, TransformStream as TransformStream$1, TransformStreamDefaultController, WritableStream, WritableStreamDefaultController, WritableStreamDefaultWriter } from "web-streams-polyfill";
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
	TransformStream: TransformStream$1,
	TransformStreamDefaultController,
	WritableStream,
	WritableStreamDefaultController,
	WritableStreamDefaultWriter
};
/** Install the WHATWG Streams constructors missing from the current runtime. */
function installStreamsPolyfill() {
	installMissingGlobals(streamGlobals);
}
installStreamsPolyfill();
//#endregion
//#region src/polyfills/encoding-streams.ts
/** Stream wrapper over the runtime's stateful native TextDecoder. */
var WabouTextDecoderStream = class {
	decoder;
	transform;
	constructor(label = "utf-8", options = {}) {
		this.decoder = new TextDecoder(label, options);
		this.transform = new TransformStream({
			transform: (chunk, controller) => {
				const text = this.decoder.decode(chunk, { stream: true });
				if (text) controller.enqueue(text);
			},
			flush: (controller) => {
				const text = this.decoder.decode();
				if (text) controller.enqueue(text);
			}
		});
	}
	get encoding() {
		return this.decoder.encoding;
	}
	get fatal() {
		return this.decoder.fatal;
	}
	get ignoreBOM() {
		return this.decoder.ignoreBOM;
	}
	get readable() {
		return this.transform.readable;
	}
	get writable() {
		return this.transform.writable;
	}
	get [Symbol.toStringTag]() {
		return "TextDecoderStream";
	}
};
/** Keep a trailing high surrogate until the next chunk or stream flush. */
var WabouTextEncoderStream = class {
	transform;
	constructor() {
		const encoder = new TextEncoder();
		let pending = "";
		this.transform = new TransformStream({
			transform(chunk, controller) {
				if (typeof chunk === "symbol") throw new TypeError("Cannot encode a Symbol");
				let text = pending + String(chunk);
				pending = "";
				const last = text.charCodeAt(text.length - 1);
				if (last >= 55296 && last <= 56319) {
					pending = text.slice(-1);
					text = text.slice(0, -1);
				}
				if (text) controller.enqueue(encoder.encode(text));
			},
			flush(controller) {
				if (pending) controller.enqueue(encoder.encode(pending));
				pending = "";
			}
		});
	}
	get encoding() {
		return "utf-8";
	}
	get readable() {
		return this.transform.readable;
	}
	get writable() {
		return this.transform.writable;
	}
	get [Symbol.toStringTag]() {
		return "TextEncoderStream";
	}
};
/** Install the Encoding Standard transforms missing from QuickJS. */
function installEncodingStreamsPolyfill() {
	installMissingGlobals({
		TextDecoderStream: WabouTextDecoderStream,
		TextEncoderStream: WabouTextEncoderStream
	});
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
	redirected;
	bodyBytes;
	consumed = false;
	constructor(body = null, init = {}, url = "", copyBody = true, redirected = false) {
		this.bodyBytes = encodeResponseBody(body, copyBody);
		this.status = init.status ?? 200;
		this.statusText = init.statusText ?? "";
		this.headers = new WabouHeaders(init.headers);
		this.url = url;
		this.redirected = redirected;
		if (body === null) this.body = null;
		else {
			const bytes = this.bodyBytes;
			const stream = new ReadableStream({ start: (controller) => {
				controller.enqueue(bytes);
				controller.close();
			} });
			const getReader = stream.getReader.bind(stream);
			stream.getReader = ((...args) => {
				const reader = getReader(...args);
				this.consumed = true;
				this.bodyBytes = null;
				return reader;
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
		if (chunks.length === 1) return chunks[0];
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
		const bytes = await this.consumeBody();
		if (bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength) return bytes.buffer;
		return bytes.slice().buffer;
	}
	clone() {
		if (this.bodyUsed) throw new TypeError("Cannot clone a consumed Response body");
		return new WabouResponse(this.body === null ? null : this.bodyBytes, {
			status: this.status,
			statusText: this.statusText,
			headers: this.headers
		}, this.url, true, this.redirected);
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
		}, data.url ?? url, false, data.redirected ?? false);
	}
};
let nextFetchId = 0;
/** Install the host-backed Fetch API surface. Safe to call again in tests. */
function installFetchPolyfill() {
	installMissingGlobals({
		Headers: WabouHeaders,
		Response: WabouResponse
	});
	if (!("__wabou_fetch" in globalThis)) return;
	globalThis.fetch = ((input, init) => {
		const signal = init?.signal;
		if (signal?.aborted) return Promise.reject(signal.reason);
		const runtime = globalThis;
		const id = ++nextFetchId;
		const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
		const serializedInit = init ? {
			method: init.method,
			headers: init.headers instanceof WabouHeaders ? init.headers.toRecord() : init.headers
		} : {};
		const body = init?.body;
		const bodyBytes = typeof body === "string" ? new TextEncoder().encode(body) : body instanceof Uint8Array ? body : body instanceof ArrayBuffer ? new Uint8Array(body) : void 0;
		const initJson = JSON.stringify(serializedInit);
		runtime.__wabou_fetch_start?.(id);
		return new Promise((resolve, reject) => {
			const abort = () => {
				runtime.__wabou_fetch_cancel?.(id);
				reject(signal.reason);
			};
			signal?.addEventListener("abort", abort, { once: true });
			runtime.__wabou_fetch(url, initJson, id, bodyBytes).then(resolve, reject).finally(() => signal?.removeEventListener("abort", abort));
		}).then((data) => {
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
export { WabouResponse };

//# sourceMappingURL=web.mjs.map