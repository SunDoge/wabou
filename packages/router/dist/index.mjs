import { children, createComponent, createContext, createMemo, createSignal, getOwner, onCleanup, useContext } from "solid-js";
//#region src/history.ts
function asEntry(entry) {
	return typeof entry === "string" ? { value: entry } : {
		value: entry.value,
		state: entry.state
	};
}
/** A deterministic, window-independent navigation stack. */
function createMemoryHistory(options = {}) {
	const entries = options.initialEntries?.length ? options.initialEntries.map(asEntry) : [{ value: "/" }];
	let index = Math.max(0, Math.min(options.initialIndex ?? entries.length - 1, entries.length - 1));
	const listeners = /* @__PURE__ */ new Set();
	const current = () => entries[index];
	const publish = () => {
		const entry = current();
		for (const listener of listeners) listener(entry);
	};
	const go = (delta) => {
		if (!Number.isFinite(delta)) return;
		const next = Math.max(0, Math.min(index + Math.trunc(delta), entries.length - 1));
		if (next === index) return;
		index = next;
		publish();
	};
	return {
		get: () => ({ ...current() }),
		set(update) {
			const next = asEntry(update);
			if (update.replace) entries[index] = next;
			else {
				entries.splice(index + 1, entries.length - index - 1, next);
				index += 1;
			}
			publish();
		},
		back: () => go(-1),
		forward: () => go(1),
		go,
		canGoBack: () => index > 0,
		canGoForward: () => index < entries.length - 1,
		listen(listener) {
			listeners.add(listener);
			return () => listeners.delete(listener);
		}
	};
}
//#endregion
//#region src/match.ts
function safeDecode(value) {
	try {
		return decodeURIComponent(value);
	} catch {
		return value;
	}
}
function normalizePathname(pathname) {
	const collapsed = (pathname.startsWith("/") ? pathname : `/${pathname}`).replace(/\/{2,}/g, "/");
	return collapsed.length > 1 ? collapsed.replace(/\/+$/, "") : collapsed;
}
function parsePath(value) {
	const hashAt = value.indexOf("#");
	const hash = hashAt >= 0 ? value.slice(hashAt) : "";
	const withoutHash = hashAt >= 0 ? value.slice(0, hashAt) : value;
	const searchAt = withoutHash.indexOf("?");
	const search = searchAt >= 0 ? withoutHash.slice(searchAt) : "";
	return {
		pathname: normalizePathname((searchAt >= 0 ? withoutHash.slice(0, searchAt) : withoutHash) || "/"),
		search,
		hash
	};
}
function resolvePath(to, from) {
	if (to.startsWith("/")) return to;
	if (to.startsWith("?") || to.startsWith("#")) {
		const current = parsePath(from);
		if (to.startsWith("?")) return `${current.pathname}${to}`;
		return `${current.pathname}${current.search}${to}`;
	}
	const segments = parsePath(from).pathname.split("/").filter(Boolean);
	for (const segment of to.split("/")) {
		if (!segment || segment === ".") continue;
		if (segment === "..") segments.pop();
		else segments.push(segment);
	}
	return `/${segments.join("/")}`;
}
/** Match a complete pathname. `:name`, `:name?`, and `*rest` are supported. */
function matchPath(pattern, pathname) {
	const patternSegments = normalizePathname(pattern).split("/").filter(Boolean);
	const pathSegments = normalizePathname(pathname).split("/").filter(Boolean);
	const params = {};
	let pathIndex = 0;
	for (const patternSegment of patternSegments) {
		if (patternSegment.startsWith("*")) {
			const name = patternSegment.slice(1) || "rest";
			params[name] = safeDecode(pathSegments.slice(pathIndex).join("/"));
			pathIndex = pathSegments.length;
			break;
		}
		const optional = patternSegment.startsWith(":") && patternSegment.endsWith("?");
		if (patternSegment.startsWith(":")) {
			const name = patternSegment.slice(1, optional ? -1 : void 0);
			const value = pathSegments[pathIndex];
			if (value === void 0) {
				if (optional) continue;
				return null;
			}
			params[name] = safeDecode(value);
			pathIndex += 1;
			continue;
		}
		if (pathSegments[pathIndex] !== patternSegment) return null;
		pathIndex += 1;
	}
	if (pathIndex !== pathSegments.length) return null;
	return {
		params,
		path: normalizePathname(pathname)
	};
}
function routeScore(pattern) {
	return normalizePathname(pattern).split("/").filter(Boolean).reduce((score, segment) => {
		if (segment.startsWith("*")) return score + 1;
		if (segment.startsWith(":")) return score + (segment.endsWith("?") ? 2 : 3);
		return score + 4;
	}, 0);
}
function joinPaths(parent, child) {
	if (child.startsWith("/")) return normalizePathname(child);
	if (!child) return normalizePathname(parent || "/");
	return normalizePathname(`${parent}/${child}`);
}
//#endregion
//#region src/index.tsx
const RouterContext = createContext();
function requireRouter() {
	if (!getOwner()) throw new Error("Wabou router hooks must be used inside <MemoryRouter>");
	let router;
	try {
		router = useContext(RouterContext);
	} catch {
		throw new Error("Wabou router hooks must be used inside <MemoryRouter>");
	}
	if (!router) throw new Error("Wabou router hooks must be used inside <MemoryRouter>");
	return router;
}
function routeChildren(value) {
	const values = [];
	const append = (candidate) => {
		if (Array.isArray(candidate)) for (const child of candidate) append(child);
		else values.push(candidate);
	};
	append(value);
	return values.filter((candidate) => typeof candidate === "object" && candidate !== null && "__wabouRoute" in candidate);
}
function Route(props) {
	const resolved = children(() => props.children);
	return {
		__wabouRoute: true,
		path: props.path,
		component: props.component,
		get children() {
			return routeChildren(resolved());
		}
	};
}
function pathVariants(path) {
	return typeof path === "string" || path === void 0 ? [path ?? ""] : path;
}
function collectMatches(definitions, pathname, parentPath = "", parents = []) {
	const matches = [];
	for (const definition of definitions) for (const variant of pathVariants(definition.path)) {
		const pattern = joinPaths(parentPath, variant);
		const chain = [...parents, definition];
		if (definition.children.length) matches.push(...collectMatches(definition.children, pathname, pattern, chain));
		const matched = matchPath(pattern, pathname);
		if (matched) matches.push({
			definitions: chain,
			params: matched.params,
			score: routeScore(pattern)
		});
	}
	return matches;
}
function renderDefinitions(definitions, index = 0) {
	const definition = definitions[index];
	if (!definition) return null;
	const outlet = () => renderDefinitions(definitions, index + 1);
	return definition.component ? createComponent(definition.component, { get children() {
		return outlet();
	} }) : outlet();
}
function sameDefinitions(left, right) {
	return left === right || left !== void 0 && right !== void 0 && left.length === right.length && left.every((definition, index) => definition === right[index]);
}
function MemoryRouter(props) {
	const history = props.history ?? createMemoryHistory({
		initialEntries: props.initialEntries,
		initialIndex: props.initialIndex
	});
	const [entry, setEntry] = createSignal(history.get());
	onCleanup(history.listen(setEntry));
	const resolvedRoutes = children(() => props.children);
	const definitions = createMemo(() => routeChildren(resolvedRoutes()));
	const path = createMemo(() => parsePath(entry().value));
	const match = createMemo(() => {
		return collectMatches(definitions(), path().pathname).sort((a, b) => b.score - a.score)[0];
	});
	const branch = createMemo((previous) => {
		const next = match()?.definitions;
		return sameDefinitions(previous, next) ? previous : next;
	});
	const params = new Proxy({}, {
		get: (_target, property) => typeof property === "string" ? match()?.params[property] : void 0,
		ownKeys: () => Reflect.ownKeys(match()?.params ?? {}),
		getOwnPropertyDescriptor: () => ({
			enumerable: true,
			configurable: true
		})
	});
	const location = {
		get pathname() {
			return path().pathname;
		},
		get search() {
			return path().search;
		},
		get hash() {
			return path().hash;
		},
		get state() {
			return entry().state;
		}
	};
	const navigate = (destination, options = {}) => {
		if (typeof destination === "number") {
			history.go(destination);
			return;
		}
		const value = options.resolve === false ? destination : resolvePath(destination, entry().value);
		history.set({
			value,
			state: options.state,
			replace: options.replace
		});
	};
	const context = {
		history,
		location,
		navigate,
		params
	};
	const routed = () => {
		const outlet = () => {
			const current = branch();
			return current ? renderDefinitions(current) : null;
		};
		return props.root ? createComponent(props.root, { get children() {
			return outlet;
		} }) : outlet;
	};
	return createComponent(RouterContext, {
		value: context,
		get children() {
			return routed();
		}
	});
}
function useNavigate() {
	return requireRouter().navigate;
}
function useLocation() {
	return requireRouter().location;
}
function useParams() {
	return requireRouter().params;
}
function useHistory() {
	return requireRouter().history;
}
//#endregion
export { MemoryRouter, Route, createMemoryHistory, matchPath, normalizePathname, parsePath, resolvePath, useHistory, useLocation, useNavigate, useParams };

//# sourceMappingURL=index.mjs.map