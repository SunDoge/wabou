//#region src/index.ts
const OP = {
	CreateElement: 1,
	CreateText: 2,
	CreateComment: 3,
	AppendChild: 4,
	InsertBefore: 5,
	RemoveChild: 6,
	ReplaceNode: 7,
	SetText: 8,
	SetAttribute: 9,
	RemoveAttribute: 10,
	SetStyle: 11,
	RemoveStyle: 12,
	AddEventListener: 13,
	RemoveEventListener: 14,
	SetClassName: 15,
	FrameEnd: 16,
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
	RemoveWidgetConfig: 27
};
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
	Version: 1,
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
	createElement(id, tag, attrs = null) {
		if (attrs && attrs.length > 65535) throw new RangeError("element cannot contain more than 65535 attributes");
		this.emit(OP.CreateElement);
		this.u32(id);
		this.atom(tag);
		this.u16(attrs ? attrs.length : 0);
		if (attrs) for (const [n, v] of attrs) {
			this.atom(n);
			this.str(v);
		}
	}
	createText(id, text) {
		this.emit(OP.CreateText);
		this.u32(id);
		this.str(text);
	}
	createComment(id, text) {
		this.emit(OP.CreateComment);
		this.u32(id);
		this.str(text);
	}
	appendChild(parent, child) {
		this.emit(OP.AppendChild);
		this.u32(parent);
		this.u32(child);
	}
	insertBefore(parent, child, ref) {
		this.emit(OP.InsertBefore);
		this.u32(parent);
		this.u32(child);
		this.u32(ref);
	}
	removeChild(parent, child) {
		this.emit(OP.RemoveChild);
		this.u32(parent);
		this.u32(child);
	}
	replaceNode(parent, oldId, newId) {
		this.emit(OP.ReplaceNode);
		this.u32(parent);
		this.u32(oldId);
		this.u32(newId);
	}
	setText(id, text) {
		this.emit(OP.SetText);
		this.u32(id);
		this.str(text);
	}
	setAttribute(id, name, value) {
		this.emit(OP.SetAttribute);
		this.u32(id);
		this.atom(name);
		this.str(value);
	}
	removeAttribute(id, name) {
		this.emit(OP.RemoveAttribute);
		this.u32(id);
		this.atom(name);
	}
	setWidgetConfig(id, json) {
		this.emit(OP.SetWidgetConfig);
		this.u32(id);
		this.str(json);
	}
	removeWidgetConfig(id) {
		this.emit(OP.RemoveWidgetConfig);
		this.u32(id);
	}
	setStyle(id, prop, value) {
		this.emit(OP.SetStyle);
		this.u32(id);
		this.atom(prop);
		this.str(value);
	}
	setStyleValue(id, prop, kind, value) {
		this.emit(OP.SetStyleValue);
		this.u32(id);
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
		this.u32(id);
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
		this.u32(id);
		for (const part of matrix) this.f32(part);
	}
	setOverlayPlane(id, plane) {
		this.emit(OP.SetOverlayPlane);
		this.u32(id);
		this.u8(plane);
	}
	setScrollbarStyle(id, style) {
		this.emit(OP.SetScrollbarStyle);
		this.u32(id);
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
		this.u32(id);
		this.atom(prop);
	}
	addEventListener(id, eventCode) {
		this.emit(OP.AddEventListener);
		this.u32(id);
		this.u8(eventCode);
	}
	removeEventListener(id, eventCode) {
		this.emit(OP.RemoveEventListener);
		this.u32(id);
		this.u8(eventCode);
	}
	setClassName(id, value) {
		this.emit(OP.SetClassName);
		this.u32(id);
		const classes = value.split(/\s+/).filter(Boolean);
		if (classes.length > 65535) throw new RangeError("class list cannot contain more than 65535 tokens");
		this.u16(classes.length);
		for (const className of classes) this.atom(className);
	}
	frameEnd() {
		this.emit(OP.FrameEnd);
	}
	dropNode(id) {
		this.emit(OP.DropNode);
		this.u32(id);
	}
	focusNode(id) {
		this.emit(OP.FocusNode);
		this.u32(id);
	}
	scrollTo(id, x, y) {
		this.emit(OP.ScrollTo);
		this.u32(id);
		this.f32(x);
		this.f32(y);
	}
	scrollBy(id, x, y) {
		this.emit(OP.ScrollBy);
		this.u32(id);
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
export { EVENT_CODE, EVENT_DATA_LEN, EVENT_DATA_SLOT, HOST_FRAME, HOST_NODE_PAYLOAD, HOST_RECORD_KIND, OP, Writer };

//# sourceMappingURL=index.mjs.map