//#region src/layout.ts
const layoutRectRight = (rect) => rect.x + rect.width;
const layoutRectBottom = (rect) => rect.y + rect.height;
/** Assert that a completed native layout rect stays inside another rect. */
function assertLayoutRectContains(outer, inner, options = {}) {
	const tolerance = options.tolerance ?? 1;
	if (inner.x < outer.x - tolerance || inner.y < outer.y - tolerance || layoutRectRight(inner) > layoutRectRight(outer) + tolerance || layoutRectBottom(inner) > layoutRectBottom(outer) + tolerance) throw new Error(`${options.label ?? "layout rect"} (${rectText(inner)}) is outside (${rectText(outer)})`);
}
/**
* Assert typography after class resolution, Style IR application and native
* layout. This deliberately checks the completed layout node instead of source
* class names, so token and font-resolution regressions are visible to tests.
*/
function assertLayoutTextStyle(node, options) {
	const tolerance = options.tolerance ?? .01;
	const label = options.label ?? (layoutName(node) || node.text || "text node");
	const checks = [[
		"font size",
		node.computed.fontSize,
		options.fontSize
	], [
		"font weight",
		node.computed.fontWeight,
		options.fontWeight
	]];
	for (const [property, actual, expected] of checks) {
		if (expected === void 0) continue;
		if (actual == null) throw new Error(`${label} omitted its resolved ${property}`);
		if (Math.abs(actual - expected) > tolerance) throw new Error(`${label} ${property}: expected ${expected}, received ${actual}`);
	}
}
function record(value, path) {
	if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`invalid layout snapshot: ${path} must be an object`);
	return value;
}
function finiteNumber(value, path) {
	if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`invalid layout snapshot: ${path} must be a finite number`);
	return value;
}
function parseKey(value, path) {
	const item = record(value, path);
	return {
		lo: finiteNumber(item.lo, `${path}.lo`),
		hi: finiteNumber(item.hi, `${path}.hi`)
	};
}
function parseRect(value, path) {
	const item = record(value, path);
	return {
		x: finiteNumber(item.x, `${path}.x`),
		y: finiteNumber(item.y, `${path}.y`),
		width: finiteNumber(item.width, `${path}.width`),
		height: finiteNumber(item.height, `${path}.height`)
	};
}
/** Validate the Rust snapshot boundary before layout assertions consume it. */
function parseLayoutSnapshot(value) {
	const root = record(value, "root");
	const status = record(root.status, "status");
	if (!Array.isArray(root.nodes)) throw new Error("invalid layout snapshot: nodes must be an array");
	const nodes = root.nodes.map((value, index) => {
		const path = `nodes[${index}]`;
		const item = record(value, path);
		if (typeof item.tag !== "string") throw new Error(`invalid layout snapshot: ${path}.tag must be a string`);
		if (!Array.isArray(item.classes) || !item.classes.every((v) => typeof v === "string")) throw new Error(`invalid layout snapshot: ${path}.classes must be a string array`);
		if (!Array.isArray(item.attrs)) throw new Error(`invalid layout snapshot: ${path}.attrs must be an array`);
		if (!Array.isArray(item.styleDiagnostics) || !item.styleDiagnostics.every((v) => typeof v === "string")) throw new Error(`invalid layout snapshot: ${path}.styleDiagnostics must be a string array`);
		return {
			...item,
			id: parseKey(item.id, `${path}.id`),
			parentId: item.parentId === null || item.parentId === void 0 ? null : parseKey(item.parentId, `${path}.parentId`),
			tag: item.tag,
			textMetrics: item.textMetrics == null ? null : (() => {
				const metrics = record(item.textMetrics, `${path}.textMetrics`);
				if (metrics.source !== "node" && metrics.source !== "widget") throw new Error(`invalid layout snapshot: ${path}.textMetrics.source must be node or widget`);
				return {
					source: metrics.source,
					lineBox: parseRect(metrics.lineBox, `${path}.textMetrics.lineBox`),
					baseline: finiteNumber(metrics.baseline, `${path}.textMetrics.baseline`)
				};
			})(),
			classes: item.classes,
			attrs: item.attrs,
			rect: parseRect(item.rect, `${path}.rect`),
			contentRect: parseRect(item.contentRect, `${path}.contentRect`),
			styleDiagnostics: item.styleDiagnostics,
			computed: record(item.computed, `${path}.computed`)
		};
	});
	return {
		status: {
			viewportWidth: finiteNumber(status.viewportWidth, "status.viewportWidth"),
			viewportHeight: finiteNumber(status.viewportHeight, "status.viewportHeight"),
			deviceScale: finiteNumber(status.deviceScale, "status.deviceScale"),
			nodeCount: finiteNumber(status.nodeCount, "status.nodeCount")
		},
		nodes
	};
}
const key = (id) => `${id.lo}:${id.hi}`;
function attrs(node) {
	return new Map(node.attrs);
}
function opaqueRgb(value) {
	if (!value?.startsWith("#")) return null;
	const hex = value.slice(1).toLowerCase();
	if (hex.length === 3 || hex.length === 4) {
		if (hex.length === 4 && hex[3] !== "f") return null;
		return [...hex].map((part) => Number.parseInt(`${part}${part}`, 16));
	}
	if (hex.length === 8 && hex.slice(6) !== "ff") return null;
	if (hex.length !== 6 && hex.length !== 8) return null;
	return [
		0,
		2,
		4
	].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16));
}
function relativeLuminance(rgb) {
	const linear = rgb.map((channel) => {
		const value = channel / 255;
		return value <= .04045 ? value / 12.92 : ((value + .055) / 1.055) ** 2.4;
	});
	return .2126 * linear[0] + .7152 * linear[1] + .0722 * linear[2];
}
/** Return the visual contrast ratio for two opaque hexadecimal colors. */
function layoutColorContrast(foreground, background) {
	const foregroundRgb = opaqueRgb(foreground);
	const backgroundRgb = opaqueRgb(background);
	if (!foregroundRgb || !backgroundRgb) return void 0;
	const first = relativeLuminance(foregroundRgb);
	const second = relativeLuminance(backgroundRgb);
	return (Math.max(first, second) + .05) / (Math.min(first, second) + .05);
}
function layoutRole(node) {
	return node.semantic?.role ?? attrs(node).get("role") ?? "";
}
function layoutName(node) {
	return node.semantic?.label ?? attrs(node).get("aria-label") ?? "";
}
function childrenByParent(snapshot) {
	const children = /* @__PURE__ */ new Map();
	for (const node of snapshot.nodes) {
		const parent = node.parentId ? key(node.parentId) : "<root>";
		const items = children.get(parent) ?? [];
		items.push(node);
		children.set(parent, items);
	}
	return children;
}
function scopedNodes(snapshot, within) {
	if (!within) return snapshot.nodes;
	const children = childrenByParent(snapshot);
	const result = [];
	const visit = (node) => {
		result.push(node);
		for (const child of children.get(key(node.id)) ?? []) visit(child);
	};
	visit(within);
	return result;
}
function isAggregatedProtocolTextLeaf(node, nodes) {
	const parent = node.parentId ? nodes.get(key(node.parentId)) : void 0;
	return node.tag === "text" && node.rect.width === 0 && node.rect.height === 0 && !node.textMetrics && parent?.tag === "text";
}
function queryLayoutNodes(snapshot, query) {
	const nodes = new Map(snapshot.nodes.map((node) => [key(node.id), node]));
	return snapshot.nodes.filter((node) => {
		if (isAggregatedProtocolTextLeaf(node, nodes)) return false;
		if (query.tag !== void 0 && node.tag !== query.tag) return false;
		if (query.role !== void 0 && layoutRole(node) !== query.role) return false;
		if (query.name !== void 0 && layoutName(node) !== query.name) return false;
		if (query.text !== void 0 && node.text !== query.text) return false;
		if (query.className !== void 0 && !node.classes.includes(query.className)) return false;
		return true;
	});
}
function getLayoutNode(snapshot, query) {
	const matches = queryLayoutNodes(snapshot, query);
	if (matches.length !== 1) throw new Error(`expected one layout node for ${JSON.stringify(query)}, found ${matches.length}`);
	return matches[0];
}
function depthOf(node, nodes) {
	let depth = 0;
	let parent = node.parentId ? nodes.get(key(node.parentId)) : void 0;
	while (parent && depth < nodes.size) {
		depth += 1;
		parent = parent.parentId ? nodes.get(key(parent.parentId)) : void 0;
	}
	return depth;
}
const rectText = (rect) => `${rect.x.toFixed(1)},${rect.y.toFixed(1)} ${rect.width.toFixed(1)}x${rect.height.toFixed(1)}`;
/** Stable text projection of the exact Style IR + Taffy result. */
function formatLayoutTree(snapshot) {
	const nodes = new Map(snapshot.nodes.map((node) => [key(node.id), node]));
	const lines = [`viewport ${snapshot.status.viewportWidth}x${snapshot.status.viewportHeight} scale=${snapshot.status.deviceScale} nodes=${snapshot.nodes.length}`];
	for (const node of snapshot.nodes) {
		const parts = [`${"  ".repeat(depthOf(node, nodes))}${node.tag}#${key(node.id)}`];
		const role = layoutRole(node);
		const name = layoutName(node);
		if (role) parts.push(`role=${role}`);
		if (name) parts.push(`name=${JSON.stringify(name)}`);
		if (node.text) parts.push(`text=${JSON.stringify(node.text.slice(0, 80))}`);
		parts.push(`rect=(${rectText(node.rect)})`);
		parts.push(`content=(${rectText(node.contentRect)})`);
		const overflowX = node.computed.overflowX ?? "Visible";
		const overflowY = node.computed.overflowY ?? "Visible";
		if (overflowX !== "Visible" || overflowY !== "Visible") parts.push(`overflow=${overflowX}/${overflowY}`);
		if (node.classes.length > 0) parts.push(`class=${JSON.stringify(node.classes.join(" "))}`);
		lines.push(parts.join(" "));
	}
	return `${lines.join("\n")}\n`;
}
function overflowAmount(outer, inner) {
	return Math.max(0, outer.x - inner.x, outer.y - inner.y, inner.x + inner.width - (outer.x + outer.width), inner.y + inner.height - (outer.y + outer.height));
}
function compactText(value, limit = 80) {
	const normalized = value.replace(/\s+/g, " ").trim();
	return normalized.length <= limit ? normalized : `${normalized.slice(0, limit - 1)}…`;
}
function diagnosticNodeText(node) {
	const parts = [`${node.tag}#${key(node.id)}`];
	const role = layoutRole(node);
	const name = layoutName(node);
	if (role) parts.push(`role=${role}`);
	if (name) parts.push(`name=${JSON.stringify(compactText(name))}`);
	if (node.text) parts.push(`text=${JSON.stringify(compactText(node.text))}`);
	if (node.classes.length > 0) parts.push(`class=${JSON.stringify(compactText(node.classes.join(" "), 120))}`);
	parts.push(`rect=(${rectText(node.rect)})`);
	const overflowX = node.computed.overflowX ?? "Visible";
	const overflowY = node.computed.overflowY ?? "Visible";
	if (overflowX !== "Visible" || overflowY !== "Visible") parts.push(`overflow=${overflowX}/${overflowY}`);
	return parts.join(" ");
}
function diagnosticAncestorPath(node, nodes) {
	const path = [];
	let current = node;
	while (current) {
		const name = layoutName(current);
		path.push(`${current.tag}#${key(current.id)}${name ? `[${JSON.stringify(compactText(name, 40))}]` : ""}`);
		current = current.parentId ? nodes.get(key(current.parentId)) : void 0;
	}
	return path.reverse().join(" > ");
}
function visibleOverflowDiagnostics(snapshot, options = {}) {
	const tolerance = options.tolerance ?? 1;
	const nodes = new Map(snapshot.nodes.map((node) => [key(node.id), node]));
	const diagnostics = [];
	for (const node of scopedNodes(snapshot, options.within)) {
		let parent = node.parentId ? nodes.get(key(node.parentId)) : void 0;
		if (isAggregatedProtocolTextLeaf(node, nodes)) continue;
		while (parent) {
			if ((parent.computed.overflowX ?? "Visible") !== "Visible" || (parent.computed.overflowY ?? "Visible") !== "Visible") break;
			const amount = overflowAmount(parent.rect, node.rect);
			if (amount > tolerance) {
				diagnostics.push({
					code: "visible-overflow",
					node,
					related: parent,
					amount,
					message: [
						`${node.tag} ${key(node.id)} extends ${amount.toFixed(1)}px outside ${parent.tag} ${key(parent.id)}`,
						`node: ${diagnosticNodeText(node)}`,
						`boundary: ${diagnosticNodeText(parent)}`,
						`path: ${diagnosticAncestorPath(node, nodes)}`
					].join("\n    ")
				});
				break;
			}
			parent = parent.parentId ? nodes.get(key(parent.parentId)) : void 0;
		}
	}
	return diagnostics;
}
function overlaps(first, second, tolerance) {
	return first.width > 0 && first.height > 0 && second.width > 0 && second.height > 0 && first.x + first.width > second.x + tolerance && second.x + second.width > first.x + tolerance && first.y + first.height > second.y + tolerance && second.y + second.height > first.y + tolerance;
}
function intersectLayoutRects(first, second) {
	const x = Math.max(first.x, second.x);
	const y = Math.max(first.y, second.y);
	const right = Math.min(layoutRectRight(first), layoutRectRight(second));
	const bottom = Math.min(layoutRectBottom(first), layoutRectBottom(second));
	if (right <= x || bottom <= y) return void 0;
	return {
		x,
		y,
		width: right - x,
		height: bottom - y
	};
}
/** Visible border-box bounds after axis-aligned native clipping. */
function visibleLayoutRect(node) {
	const clip = node.clip;
	if (!clip) return node.rect;
	const effective = clip.effective;
	if (effective && (effective.coordinateSpace === "window-logical" || effective.coordinateSpace === "layout-window-logical")) return intersectLayoutRects(node.rect, effective.rect);
	let visible = node.rect;
	for (const ancestor of clip.chain) {
		if (ancestor.coordinateSpace !== "window-logical" && ancestor.coordinateSpace !== "layout-window-logical") continue;
		visible = visible ? intersectLayoutRects(visible, ancestor.rect) : void 0;
		if (!visible) return void 0;
	}
	return visible;
}
/** Opt-in collision check for normal-flow siblings. */
function siblingCollisionDiagnostics(snapshot, options = {}) {
	const tolerance = options.tolerance ?? 1;
	const scope = new Set(scopedNodes(snapshot, options.within).map((node) => key(node.id)));
	const children = childrenByParent(snapshot);
	const diagnostics = [];
	for (const siblings of children.values()) {
		const flow = siblings.filter((node) => scope.has(key(node.id)) && node.computed.position !== "Absolute");
		for (let index = 0; index < flow.length; index += 1) for (const second of flow.slice(index + 1)) {
			const first = flow[index];
			if (first.computed.overlayPlane !== second.computed.overlayPlane || !overlaps(first.rect, second.rect, tolerance)) continue;
			diagnostics.push({
				code: "flow-sibling-overlap",
				node: second,
				related: first,
				message: `${first.tag} ${key(first.id)} overlaps sibling ${second.tag} ${key(second.id)}`
			});
		}
	}
	return diagnostics;
}
/**
* Opt-in collision check for visible text leaves across component subtrees.
* This catches content collisions that a direct-sibling layout check cannot
* see, such as a transformed reaction inside a bubble covering its footer.
*/
function textCollisionDiagnostics(snapshot, options = {}) {
	const tolerance = options.tolerance ?? 1;
	const textNodes = scopedNodes(snapshot, options.within).filter((node) => node.tag === "text" && Boolean(node.text));
	const diagnostics = [];
	for (let index = 0; index < textNodes.length; index += 1) for (const second of textNodes.slice(index + 1)) {
		const first = textNodes[index];
		const firstVisible = visibleLayoutRect(first);
		const secondVisible = visibleLayoutRect(second);
		if (first.computed.overlayPlane !== second.computed.overlayPlane || !firstVisible || !secondVisible || !overlaps(firstVisible, secondVisible, tolerance)) continue;
		diagnostics.push({
			code: "text-overlap",
			node: second,
			related: first,
			message: `text ${key(first.id)} ${JSON.stringify(first.text)} overlaps text ${key(second.id)} ${JSON.stringify(second.text)}`
		});
	}
	return diagnostics;
}
function styleDiagnostics(snapshot, options = {}) {
	return scopedNodes(snapshot, options.within).flatMap((node) => node.styleDiagnostics.map((message) => ({
		code: "style-diagnostic",
		node,
		message
	})));
}
/**
* Opt-in visual legibility checks over the resolved native scene contract.
* These intentionally consume computed colors and geometry rather than source
* class names, so theme changes and component composition are covered too.
*/
function visualQualityDiagnostics(snapshot, options = {}) {
	const minimumContrast = options.minimumTextContrast ?? 4.5;
	const minimumTarget = options.minimumInteractiveTarget ?? 28;
	const nodes = new Map(snapshot.nodes.map((node) => [key(node.id), node]));
	const diagnostics = [];
	for (const node of scopedNodes(snapshot, options.within)) {
		const nodeAttrs = attrs(node);
		if (nodeAttrs.get("aria-hidden") === "true") continue;
		if (node.tag === "text" && node.text?.trim()) {
			const foreground = node.computed.textColor;
			let background = node.computed.background;
			let ancestor = node.parentId ? nodes.get(key(node.parentId)) : void 0;
			while (!background && ancestor) {
				background = ancestor.computed.background;
				ancestor = ancestor.parentId ? nodes.get(key(ancestor.parentId)) : void 0;
			}
			const contrast = foreground && background ? layoutColorContrast(foreground, background) : void 0;
			if (contrast !== void 0 && contrast + .01 < minimumContrast) diagnostics.push({
				code: "low-text-contrast",
				node,
				amount: contrast,
				message: `${diagnosticNodeText(node)} has ${contrast.toFixed(2)}:1 text contrast (${foreground} on ${background}); expected at least ${minimumContrast.toFixed(2)}:1`
			});
		}
		const role = layoutRole(node);
		const minimumWidth = options.minimumInteractiveTarget === void 0 && role === "switch" ? 40 : minimumTarget;
		const minimumHeight = options.minimumInteractiveTarget === void 0 && role === "switch" ? 24 : minimumTarget;
		if ((role === "button" || role === "checkbox" || role === "combobox" || role === "radio" || role === "switch") && nodeAttrs.get("disabled") !== "true" && (node.rect.width + .01 < minimumWidth || node.rect.height + .01 < minimumHeight)) diagnostics.push({
			code: "interactive-target-too-small",
			node,
			amount: Math.min(node.rect.width, node.rect.height),
			message: `${diagnosticNodeText(node)} role=${role} is ${node.rect.width.toFixed(1)}x${node.rect.height.toFixed(1)}; expected at least ${minimumWidth.toFixed(1)}x${minimumHeight.toFixed(1)}px`
		});
	}
	return diagnostics;
}
function assertNoLayoutDiagnostics(diagnostics) {
	if (diagnostics.length === 0) return;
	throw new Error(`layout diagnostics:\n${diagnostics.map((item) => `  - [${item.code}] ${item.message}`).join("\n")}`);
}
//#endregion
export { assertLayoutRectContains, assertLayoutTextStyle, assertNoLayoutDiagnostics, formatLayoutTree, getLayoutNode, layoutColorContrast, layoutName, layoutRectBottom, layoutRectRight, layoutRole, parseLayoutSnapshot, queryLayoutNodes, siblingCollisionDiagnostics, styleDiagnostics, textCollisionDiagnostics, visibleOverflowDiagnostics, visualQualityDiagnostics };

//# sourceMappingURL=layout.mjs.map