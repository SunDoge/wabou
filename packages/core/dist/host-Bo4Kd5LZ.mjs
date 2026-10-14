import { _ as isNodeKey } from "./protocol-C0xTI-jG.mjs";
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
export { defaultHost as n, useHost as r, HostProvider as t };

//# sourceMappingURL=host-Bo4Kd5LZ.mjs.map