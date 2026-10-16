import { o as isNodeKey } from "./protocol-B9cErdtk.mjs";
import { createComponent, createContext, getOwner, useContext } from "solid-js";
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
	systemCalendarDate: () => JSON.parse(__wabou_system_calendar_date()),
	firstWeekday: (locale) => typeof __wabou_first_weekday === "function" ? __wabou_first_weekday(locale) : new Intl.Locale(locale).getWeekInfo().firstDay % 7,
	formatNumber: (locale, value, options) => typeof __wabou_format_number === "function" ? __wabou_format_number(locale, value, options) : new Intl.NumberFormat(locale, JSON.parse(options)).format(value),
	parseNumber: (locale, input, options) => {
		if (typeof __wabou_parse_number === "function") return __wabou_parse_number(locale, input, options);
		const parts = new Intl.NumberFormat(locale, JSON.parse(options)).formatToParts(12345.6);
		const group = parts.find((part) => part.type === "group")?.value ?? ",";
		const decimal = parts.find((part) => part.type === "decimal")?.value ?? ".";
		const normalized = input.replaceAll(group, "").replace(decimal, ".").replace(/[^0-9+.-]/g, "");
		return Number(normalized);
	},
	pluralCategory: (locale, value, options) => typeof __wabou_plural_category === "function" ? __wabou_plural_category(locale, value, options) : new Intl.PluralRules(locale, JSON.parse(options)).select(value),
	formatDate: (locale, epochMillis, options) => typeof __wabou_format_date === "function" ? __wabou_format_date(locale, epochMillis, options) : new Intl.DateTimeFormat(locale, JSON.parse(options)).format(epochMillis),
	formatRelativeTime: (locale, value, unit, options) => typeof __wabou_format_relative_time === "function" ? __wabou_format_relative_time(locale, value, unit, options) : new Intl.RelativeTimeFormat(locale, JSON.parse(options)).format(value, unit)
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
		today: nativeHost.systemCalendarDate,
		firstWeekday: nativeHost.firstWeekday,
		formatNumber: nativeHost.formatNumber,
		parseNumber: nativeHost.parseNumber,
		pluralCategory: nativeHost.pluralCategory,
		formatDate: nativeHost.formatDate,
		formatRelativeTime: nativeHost.formatRelativeTime
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
const defaultHost = Object.assign(builtinHost, typeof __wabou_host_modules === "undefined" ? {} : __wabou_host_modules);
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
export { defaultHost as n, useHost as r, HostProvider as t };

//# sourceMappingURL=host-B5CKGzCs.mjs.map