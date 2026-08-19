//#region src/protocol/resource-key.ts
const U32_MAX = 4294967295;
const resourceKeyFamily = Symbol("wabou.resource-key-family");
function u32(value, field) {
	if (!Number.isInteger(value) || value < 0 || value > U32_MAX) throw new RangeError(`${field} must be an unsigned 32-bit integer`);
	return value;
}
/** Validate the common two-u32 SlotMap wire representation. */
function validateResourceKeyParts(value, label = "ResourceKey") {
	const lo = u32(value.lo, `${label}.lo`);
	const hi = u32(value.hi, `${label}.hi`);
	if (lo === 0) throw new RangeError(`${label} slot zero is reserved`);
	if ((hi & 1) === 0) throw new RangeError(`${label} generation must be a non-zero odd u32`);
	return {
		lo,
		hi
	};
}
/** Structural check for a key arriving through JSON or another untyped edge. */
function isResourceKeyParts(value) {
	if (value === null || typeof value !== "object") return false;
	const candidate = value;
	return typeof candidate.lo === "number" && Number.isInteger(candidate.lo) && candidate.lo > 0 && candidate.lo <= U32_MAX && typeof candidate.hi === "number" && Number.isInteger(candidate.hi) && candidate.hi > 0 && candidate.hi <= U32_MAX && (candidate.hi & 1) === 1;
}
/** Stable diagnostic form; binary paths continue to write two u32 fields. */
function formatResourceKeyParts(value) {
	return `${value.lo}v${value.hi}`;
}
/** Slot-indexed storage that validates both the family and generation. */
var ResourceKeyTable = class {
	#family;
	#entries = [];
	constructor(family) {
		this.#family = family;
	}
	set(key, value) {
		this.#family.assert(key);
		this.#entries[key.lo] = {
			hi: key.hi,
			value
		};
		return this;
	}
	get(key) {
		if (!this.#family.is(key)) return void 0;
		const entry = this.#entries[key.lo];
		return entry?.hi === key.hi ? entry.value : void 0;
	}
	has(key) {
		return this.#family.is(key) && this.#entries[key.lo]?.hi === key.hi;
	}
	delete(key) {
		if (!this.#family.is(key)) return false;
		if (this.#entries[key.lo]?.hi !== key.hi) return false;
		this.#entries[key.lo] = void 0;
		return true;
	}
	clear() {
		this.#entries.length = 0;
	}
};
/**
* Define one opaque handle family. The private symbol token also catches
* accidental cross-family casts at runtime; it is not serialized on the wire.
*/
function createResourceKeyFamily(name, options = {}) {
	const token = Symbol(`wabou.resource-key.${name}`);
	const runtimeBrand = options.runtimeBrand ?? true;
	const fromParts = (lo, hi) => {
		const parts = validateResourceKeyParts({
			lo,
			hi
		}, `${name} key`);
		if (runtimeBrand) Object.defineProperty(parts, resourceKeyFamily, { value: token });
		return parts;
	};
	const is = (value) => isResourceKeyParts(value) && (!runtimeBrand || value[resourceKeyFamily] === token);
	const assert = (value) => {
		if (!is(value)) throw new TypeError(`expected a ${name} resource key`);
	};
	const family = {
		name,
		fromParts,
		fromJSON(value) {
			if (!isResourceKeyParts(value)) throw new TypeError(`expected { lo, hi } for a ${name} resource key`);
			return fromParts(value.lo, value.hi);
		},
		is,
		assert,
		equals(left, right) {
			return left === right || !!left && !!right && is(left) && is(right) && left.lo === right.lo && left.hi === right.hi;
		},
		format(value) {
			return `${name}:${formatResourceKeyParts(value)}`;
		},
		table() {
			return new ResourceKeyTable(family);
		}
	};
	return Object.freeze(family);
}
//#endregion
//#region src/protocol/node-key.ts
const nodeKeyFamily = createResourceKeyFamily("node", { runtimeBrand: false });
const ROOT_NODE_KEY = nodeKey(1, 1);
/** Construct a node key received from a trusted binary boundary. */
function nodeKey(lo, hi) {
	return nodeKeyFamily.fromParts(lo, hi);
}
function isNodeKey(value) {
	return isResourceKeyParts(value);
}
function nodeKeyEquals(left, right) {
	return nodeKeyFamily.equals(left, right);
}
/** Stable diagnostic form; do not use it on the binary hot path. */
function formatNodeKey(key) {
	return `${key.lo}v${key.hi}`;
}
/**
* Splits SlotMap's `KeyData::as_ffi()` representation without converting the
* full value to an imprecise JavaScript number.
*/
function nodeKeyFromSlotMapFfi(lo, hi) {
	return nodeKey(lo, hi);
}
/**
* Allocates full-width generational node keys. Exhausted generations retire a
* slot instead of wrapping and making a stale key valid again.
*/
var NodeKeyAllocator = class {
	#generations = [];
	#live = [];
	#free = [];
	#nextSlot;
	constructor(firstSlot = 2) {
		if (!Number.isInteger(firstSlot) || firstSlot < 0 || firstSlot > 4294967295) throw new RangeError("firstSlot must be an unsigned 32-bit integer");
		this.#nextSlot = firstSlot;
		if (firstSlot === 0) throw new RangeError("slot zero is reserved");
	}
	allocate() {
		const lo = this.#free.pop() ?? this.#allocateSlot();
		const hi = this.#generations[lo] ?? 1;
		this.#generations[lo] = hi;
		this.#live[lo] = true;
		return nodeKey(lo, hi);
	}
	release(key) {
		if (!this.isLive(key)) return false;
		this.#live[key.lo] = false;
		const next = key.hi + 2;
		if (next <= 4294967295) {
			this.#generations[key.lo] = next;
			this.#free.push(key.lo);
		}
		return true;
	}
	isLive(key) {
		return this.#live[key.lo] === true && this.#generations[key.lo] === key.hi;
	}
	#allocateSlot() {
		if (this.#nextSlot > 4294967295) throw new RangeError("NodeKey slot space exhausted");
		return this.#nextSlot++;
	}
};
/**
* Slot-indexed storage which always validates the complete generational key.
* This keeps array lookup speed without allowing stale-key aliasing.
*/
var NodeKeyTable = class extends ResourceKeyTable {
	constructor() {
		super(nodeKeyFamily);
	}
};
//#endregion
//#region src/protocol/index.ts
const OP = {
	CreateElement: 1,
	CreateText: 2,
	AppendChild: 4,
	InsertBefore: 5,
	RemoveChild: 6,
	SetText: 8,
	SetAttribute: 9,
	RemoveAttribute: 10,
	SetStyle: 11,
	RemoveStyle: 12,
	AddEventListener: 13,
	RemoveEventListener: 14,
	SetClassName: 15,
	DropNode: 17,
	SetTransform2D: 18,
	FocusNode: 19,
	ScrollTo: 20,
	ScrollBy: 21,
	SetStyleValue: 22,
	SetShadows: 23,
	SetOverlayPlane: 24,
	SetScrollbarStyle: 25,
	SetWidgetConfig: 26,
	RemoveWidgetConfig: 27,
	SetTextBehavior: 28,
	SetInteractionPolicy: 29,
	SetGraphicSource: 30,
	ClearGraphicSource: 31
};
const TEXT_BEHAVIOR = {
	AggregateDirectText: 1,
	SingleLine: 2
};
const TEXT_BEHAVIOR_MASK = TEXT_BEHAVIOR.AggregateDirectText | TEXT_BEHAVIOR.SingleLine;
const INTERACTION_POLICY = {
	Focusable: 1,
	BlockSubtree: 2,
	ContainFocus: 4
};
const INTERACTION_POLICY_MASK = INTERACTION_POLICY.Focusable | INTERACTION_POLICY.BlockSubtree | INTERACTION_POLICY.ContainFocus;
const GRAPHIC_SOURCE = {
	Svg: 1,
	NetworkRaster: 2
};
function validGraphicSourceKind(kind) {
	return kind === GRAPHIC_SOURCE.Svg || kind === GRAPHIC_SOURCE.NetworkRaster;
}
const EVENT_CODE = {
	click: 1,
	input: 2,
	submit: 3,
	keydown: 4,
	keyup: 5,
	change: 6,
	pointerdown: 7,
	pointermove: 8,
	pointerup: 9,
	pointerenter: 10,
	pointerleave: 11,
	wheel: 12,
	focus: 13,
	blur: 14,
	imecommit: 15,
	pointercancel: 16,
	pointerover: 17,
	pointerout: 18,
	contextmenu: 19,
	dblclick: 20,
	focusin: 21,
	focusout: 22,
	scroll: 23,
	terminalexit: 24,
	terminalprogress: 25,
	terminalnotification: 26,
	terminaltitlechange: 27,
	terminalcwdchange: 28,
	terminalselectionchange: 29,
	textselectionchange: 30,
	terminalbell: 31,
	resourceready: 32,
	resourceerror: 33
};
const EVENT_DATA_SLOT = {
	clientX: 0,
	clientY: 1,
	offsetX: 2,
	offsetY: 3,
	button: 4,
	buttons: 5,
	mods: 6,
	deltaX: 7,
	deltaY: 8,
	scrollX: 9,
	scrollY: 10
};
const EVENT_DATA_LEN = Object.keys(EVENT_DATA_SLOT).length;
/** Versioned Host → JS frame envelope. Keep in sync through `bun run gen`. */
const HOST_FRAME = {
	Magic: 826689623,
	Version: 2,
	HeaderLen: 32
};
const HOST_RECORD_KIND = {
	NodeEvent: 1,
	Resize: 2,
	ApplicationMessage: 3,
	Window: 4,
	Widget: 5
};
const HOST_NODE_PAYLOAD = {
	None: 0,
	Numeric: 1,
	Json: 2
};
const fallbackAtoms = /* @__PURE__ */ new Map();
function fallbackIntern(value) {
	let id = fallbackAtoms.get(value);
	if (id === void 0) {
		id = fallbackAtoms.size + 1;
		fallbackAtoms.set(value, id);
	}
	return id;
}
let encoder;
const FLOAT_VIEW = /* @__PURE__ */ new DataView(/* @__PURE__ */ new ArrayBuffer(4));
function utf8Encode(s) {
	if (encoder === void 0) encoder = typeof TextEncoder !== "undefined" ? new TextEncoder() : null;
	if (encoder) return encoder.encode(s);
	const out = [];
	for (let i = 0; i < s.length; i++) {
		const c = s.charCodeAt(i);
		if (c < 128) out.push(c);
		else if (c < 2048) out.push(192 | c >> 6, 128 | c & 63);
		else if (c >= 55296 && c <= 56319) {
			const c2 = s.charCodeAt(++i);
			const cp = 65536 + ((c & 1023) << 10) + (c2 & 63);
			out.push(240 | cp >> 18, 128 | cp >> 12 & 63, 128 | cp >> 6 & 63, 128 | cp & 63);
		} else out.push(224 | c >> 12, 128 | c >> 6 & 63, 128 | c & 63);
	}
	return new Uint8Array(out);
}
/**
* Per-tick binary frame writer. Emits ops into an internal buffer; `flush()`
* returns the complete frame (header + ops) or null if nothing was emitted.
* The caller owns how the bytes cross the host bridge.
*/
var Writer = class {
	buf = /* @__PURE__ */ new Uint8Array(4096);
	cursor = 8;
	count = 0;
	seq = 0;
	atoms = /* @__PURE__ */ new Map();
	/** Strings already emitted in this frame (only values large enough to win). */
	frameStrings = /* @__PURE__ */ new Map();
	internHost;
	constructor(internHost) {
		this.internHost = internHost ?? ((value) => {
			const hostIntern = globalThis.__wabou_intern;
			return hostIntern ? hostIntern(value) : fallbackIntern(value);
		});
	}
	ensure(n) {
		if (this.cursor + n <= this.buf.length) return;
		let cap = this.buf.length;
		while (cap < this.cursor + n) cap *= 2;
		const next = new Uint8Array(cap);
		next.set(this.buf);
		this.buf = next;
	}
	u8(v) {
		this.ensure(1);
		this.buf[this.cursor++] = v & 255;
	}
	u16(v) {
		this.ensure(2);
		const c = this.cursor;
		this.buf[c] = v & 255;
		this.buf[c + 1] = v >> 8 & 255;
		this.cursor += 2;
	}
	u32(v) {
		this.ensure(4);
		const c = this.cursor;
		this.buf[c] = v & 255;
		this.buf[c + 1] = v >> 8 & 255;
		this.buf[c + 2] = v >> 16 & 255;
		this.buf[c + 3] = v >> 24 & 255;
		this.cursor += 4;
	}
	key(value) {
		this.u32(value.lo);
		this.u32(value.hi);
	}
	f32(v) {
		FLOAT_VIEW.setFloat32(0, v, true);
		this.u32(FLOAT_VIEW.getUint32(0, true));
	}
	str(s) {
		const existing = this.frameStrings.get(s);
		if (existing !== void 0) {
			this.u16(65535);
			this.u16(existing);
			return;
		}
		const bytes = utf8Encode(s);
		if (bytes.length >= 65535) throw new RangeError(`protocol string is ${bytes.length} bytes; maximum is 65534`);
		this.u16(bytes.length);
		this.ensure(bytes.length);
		this.buf.set(bytes, this.cursor);
		this.cursor += bytes.length;
		if (bytes.length >= 4 && this.frameStrings.size <= 65535) this.frameStrings.set(s, this.frameStrings.size);
	}
	atom(value) {
		let id = this.atoms.get(value);
		if (id === void 0) {
			id = this.internHost(value);
			if (!Number.isInteger(id) || id <= 0 || id > 4294967295) throw new RangeError(`invalid Atom ID ${id}`);
			this.atoms.set(value, id);
		}
		this.u32(id);
	}
	emit(op) {
		if (this.count === 4294967295) throw new RangeError("protocol frame cannot contain more than 2^32-1 ops");
		this.u8(op);
		this.count++;
	}
	createElement(id, tag) {
		this.emit(OP.CreateElement);
		this.key(id);
		this.atom(tag);
	}
	createText(id, text) {
		this.emit(OP.CreateText);
		this.key(id);
		this.str(text);
	}
	appendChild(parent, child) {
		this.emit(OP.AppendChild);
		this.key(parent);
		this.key(child);
	}
	insertBefore(parent, child, ref) {
		this.emit(OP.InsertBefore);
		this.key(parent);
		this.key(child);
		this.key(ref);
	}
	removeChild(parent, child) {
		this.emit(OP.RemoveChild);
		this.key(parent);
		this.key(child);
	}
	setText(id, text) {
		this.emit(OP.SetText);
		this.key(id);
		this.str(text);
	}
	setAttribute(id, name, value) {
		this.emit(OP.SetAttribute);
		this.key(id);
		this.atom(name);
		this.str(value);
	}
	removeAttribute(id, name) {
		this.emit(OP.RemoveAttribute);
		this.key(id);
		this.atom(name);
	}
	setWidgetConfig(id, json) {
		this.emit(OP.SetWidgetConfig);
		this.key(id);
		this.str(json);
	}
	setTextBehavior(id, flags) {
		if (!Number.isInteger(flags) || flags < 0 || (flags & ~TEXT_BEHAVIOR_MASK) !== 0) throw new RangeError(`invalid text behavior flags ${flags}`);
		this.emit(OP.SetTextBehavior);
		this.key(id);
		this.u8(flags);
	}
	setInteractionPolicy(id, flags, focusOrder) {
		if (!Number.isInteger(flags) || flags < 0 || (flags & ~INTERACTION_POLICY_MASK) !== 0) throw new RangeError(`invalid interaction policy flags ${flags}`);
		if (!Number.isInteger(focusOrder) || focusOrder < -2147483648 || focusOrder > 2147483647) throw new RangeError(`invalid focus order ${focusOrder}`);
		if ((flags & INTERACTION_POLICY.Focusable) === 0 && focusOrder !== 0) throw new RangeError("a non-focusable policy must encode focus order 0");
		this.emit(OP.SetInteractionPolicy);
		this.key(id);
		this.u8(flags);
		this.u32(focusOrder >>> 0);
	}
	setGraphicSource(id, kind, source) {
		if (!validGraphicSourceKind(kind)) throw new RangeError(`invalid graphic source kind ${kind}`);
		this.emit(OP.SetGraphicSource);
		this.key(id);
		this.u8(kind);
		this.str(source);
	}
	clearGraphicSource(id, kind) {
		if (!validGraphicSourceKind(kind)) throw new RangeError(`invalid graphic source kind ${kind}`);
		this.emit(OP.ClearGraphicSource);
		this.key(id);
		this.u8(kind);
	}
	removeWidgetConfig(id) {
		this.emit(OP.RemoveWidgetConfig);
		this.key(id);
	}
	setStyle(id, prop, value) {
		this.emit(OP.SetStyle);
		this.key(id);
		this.atom(prop);
		this.str(value);
	}
	setStyleValue(id, prop, kind, value) {
		this.emit(OP.SetStyleValue);
		this.key(id);
		this.atom(prop);
		this.u8(kind);
		if (kind !== 6) {
			if (kind === 5) this.u32(value >>> 0);
			else if (kind === 4) this.u8(value ? 1 : 0);
			else this.f32(value);
		}
	}
	setShadows(id, shadows) {
		if (shadows.length > 65535) throw new RangeError("a node cannot have more than 65535 shadow layers");
		this.emit(OP.SetShadows);
		this.key(id);
		this.u16(shadows.length);
		for (const shadow of shadows) {
			this.f32(shadow.offsetX);
			this.f32(shadow.offsetY);
			this.f32(shadow.spread);
			this.f32(shadow.stdDev);
			this.u32(shadow.color >>> 0);
			this.f32(shadow.radius ?? NaN);
		}
	}
	setTransform2D(id, matrix) {
		this.emit(OP.SetTransform2D);
		this.key(id);
		for (const part of matrix) this.f32(part);
	}
	setOverlayPlane(id, plane) {
		this.emit(OP.SetOverlayPlane);
		this.key(id);
		this.u8(plane);
	}
	setScrollbarStyle(id, style) {
		this.emit(OP.SetScrollbarStyle);
		this.key(id);
		this.u8(style.visibility);
		this.f32(style.hideDelay);
		this.f32(style.fadeDuration);
		this.f32(style.thickness);
		this.f32(style.margin);
		this.f32(style.minThumbLength);
		this.f32(style.radius);
		this.u32(style.trackColor >>> 0);
		this.u32(style.thumbColor >>> 0);
		this.u32(style.hoverColor >>> 0);
		this.u32(style.activeColor >>> 0);
	}
	removeStyle(id, prop) {
		this.emit(OP.RemoveStyle);
		this.key(id);
		this.atom(prop);
	}
	addEventListener(id, eventCode) {
		this.emit(OP.AddEventListener);
		this.key(id);
		this.u8(eventCode);
	}
	removeEventListener(id, eventCode) {
		this.emit(OP.RemoveEventListener);
		this.key(id);
		this.u8(eventCode);
	}
	setClassName(id, value) {
		this.emit(OP.SetClassName);
		this.key(id);
		const classes = value.split(/\s+/).filter(Boolean);
		if (classes.length > 65535) throw new RangeError("class list cannot contain more than 65535 tokens");
		this.u16(classes.length);
		for (const className of classes) this.atom(className);
	}
	dropNode(id) {
		this.emit(OP.DropNode);
		this.key(id);
	}
	focusNode(id) {
		this.emit(OP.FocusNode);
		this.key(id);
	}
	scrollTo(id, x, y) {
		this.emit(OP.ScrollTo);
		this.key(id);
		this.f32(x);
		this.f32(y);
	}
	scrollBy(id, x, y) {
		this.emit(OP.ScrollBy);
		this.key(id);
		this.f32(x);
		this.f32(y);
	}
	/** Drain the buffer into a frame, or null if no ops were emitted this tick. */
	flush() {
		if (this.count === 0) return null;
		this.seq++;
		const s = this.seq;
		this.buf[0] = s & 255;
		this.buf[1] = s >> 8 & 255;
		this.buf[2] = s >> 16 & 255;
		this.buf[3] = s >> 24 & 255;
		this.buf[4] = this.count & 255;
		this.buf[5] = this.count >> 8 & 255;
		this.buf[6] = this.count >> 16 & 255;
		this.buf[7] = this.count >> 24 & 255;
		const out = this.buf.subarray(0, this.cursor);
		this.cursor = 8;
		this.count = 0;
		this.frameStrings.clear();
		return out;
	}
};
//#endregion
export { isResourceKeyParts as C, formatResourceKeyParts as S, nodeKey as _, HOST_FRAME as a, ResourceKeyTable as b, INTERACTION_POLICY as c, Writer as d, NodeKeyAllocator as f, isNodeKey as g, formatNodeKey as h, GRAPHIC_SOURCE as i, OP as l, ROOT_NODE_KEY as m, EVENT_DATA_LEN as n, HOST_NODE_PAYLOAD as o, NodeKeyTable as p, EVENT_DATA_SLOT as r, HOST_RECORD_KIND as s, EVENT_CODE as t, TEXT_BEHAVIOR as u, nodeKeyEquals as v, validateResourceKeyParts as w, createResourceKeyFamily as x, nodeKeyFromSlotMapFfi as y };

//# sourceMappingURL=protocol-DfLpXnPC.mjs.map