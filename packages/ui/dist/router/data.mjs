import { Show, createComponent, createContext, createMemo, createSignal, flush, getOwner, onCleanup, untrack, useContext } from "solid-js";
import { createMemoryHistory, createMemoryHistory as createMemoryHistory$1 } from "@tanstack/history";
import { BaseRootRoute, BaseRoute, RouterCore, notFound, redirect } from "@tanstack/router-core";
//#region src/router/data.tsx
globalThis.scrollTo ??= () => {};
function createMutableStore(initial) {
	const [read, write] = createSignal(() => initial, { ownedWrite: true });
	return {
		get: read,
		set(next) {
			const value = typeof next === "function" ? next(untrack(read)) : next;
			write(() => value);
		}
	};
}
function createReadonlyStore(read) {
	return { get: read };
}
const solidStores = () => ({
	createMutableStore,
	createReadonlyStore,
	batch: flush
});
const routerPersistence = /* @__PURE__ */ new WeakMap();
function isPersistedRouterLocation(value) {
	if (value === null || Array.isArray(value) || typeof value !== "object") return false;
	const record = value;
	return typeof record.version === "number" && Number.isSafeInteger(record.version) && typeof record.href === "string" && record.href.startsWith("/");
}
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
function createDataRouter(options) {
	const { persistence, ...routerOptions } = options;
	const history = routerOptions.history ?? createMemoryHistory({ initialEntries: ["/"] });
	const router = new RouterCore({
		isServer: false,
		origin: "wabou://app",
		...routerOptions,
		history
	}, solidStores);
	router.startTransition = async (commit) => {
		flush(commit);
		return true;
	};
	if (persistence) routerPersistence.set(router, persistence);
	return router;
}
const DataRouterContext = createContext();
function requireDataRouter() {
	if (!getOwner()) throw new Error("Wabou data-router hooks must be used inside <RouterProvider>");
	const router = useContext(DataRouterContext);
	if (!router) throw new Error("Wabou data-router hooks must be used inside <RouterProvider>");
	return router;
}
function matchView(router, index) {
	const match = router.state.matches[index];
	if (!match) return void 0;
	const options = router.routesById[match.routeId].options;
	return match.status === "error" ? options.errorComponent : match.status === "notFound" ? options.notFoundComponent : match.status === "pending" ? options.pendingComponent : options.component;
}
/** Preserve a matched component while its route and selected view are stable. */
function RouteMatch(props) {
	const match = () => props.router.state.matches[props.index];
	let outlet;
	return createComponent(Show, {
		get when() {
			return matchView(props.router, props.index);
		},
		keyed: true,
		children: (view) => createComponent(view, {
			get error() {
				return match()?.error;
			},
			get children() {
				if (outlet === void 0) outlet = createComponent(RouteOutlet, {
					router: props.router,
					index: props.index + 1
				});
				return outlet;
			}
		})
	});
}
/** Key only the route level that changed instead of rebuilding all matches. */
function RouteOutlet(props) {
	return createComponent(Show, {
		get when() {
			return props.router.state.matches[props.index]?.routeId;
		},
		keyed: true,
		get fallback() {
			return props.fallback ?? null;
		},
		children: (_routeId) => createComponent(RouteMatch, {
			router: props.router,
			index: props.index
		})
	});
}
/** Own router lifecycle and render its current native component branch. */
function RouterProvider(props) {
	const router = untrack(() => props.router);
	let disposed = false;
	let loadScheduled = false;
	let initialized = false;
	let pendingHref;
	let writer;
	const persistence = routerPersistence.get(router);
	const persistenceVersion = persistence?.version ?? 1;
	const persistLocation = (href) => {
		if (!persistence) return;
		pendingHref = href;
		if (writer) return;
		writer = Promise.resolve().then(async () => {
			while (!disposed && pendingHref !== void 0) {
				const next = pendingHref;
				pendingHref = void 0;
				await persistence.kv.set(persistence.key, {
					version: persistenceVersion,
					href: next
				});
			}
		}).catch((error) => {
			console.warn(`[wabou-router] could not persist location: ${String(error)}`);
		}).finally(() => {
			writer = void 0;
			if (!disposed && pendingHref !== void 0) persistLocation(pendingHref);
		});
	};
	const scheduleLoad = () => {
		if (disposed || !initialized || loadScheduled) return;
		loadScheduled = true;
		Promise.resolve().then(async () => {
			loadScheduled = false;
			if (!disposed) await router.load();
		}).catch((error) => {
			console.error(`[wabou-router] route load failed: ${String(error)}`);
		});
	};
	const unsubscribe = router.history.subscribe(() => {
		if (!initialized) return;
		scheduleLoad();
		persistLocation(router.history.location.href);
	});
	onCleanup(() => {
		disposed = true;
		unsubscribe();
	});
	Promise.resolve().then(async () => {
		if (persistence) {
			const entry = await persistence.kv.get(persistence.key);
			if (entry && isPersistedRouterLocation(entry.value) && entry.value.version === persistenceVersion) router.history.replace(entry.value.href);
		}
	}).catch((error) => {
		console.warn(`[wabou-router] could not restore persisted location: ${String(error)}`);
	}).finally(() => {
		initialized = true;
		scheduleLoad();
	});
	return createComponent(DataRouterContext, {
		value: router,
		get children() {
			return createComponent(RouteOutlet, {
				router,
				index: 0,
				get fallback() {
					return props.fallback;
				}
			});
		}
	});
}
function useRouter() {
	return requireDataRouter();
}
function useRouterState(selector) {
	const router = requireDataRouter();
	return createMemo(() => selector(router));
}
function useNavigate() {
	return requireDataRouter().navigate;
}
function useLocation() {
	const router = requireDataRouter();
	return createMemo(() => router.state.location);
}
/**
* Reactively report whether a native router destination is active.
*
* This delegates path, base-path, parameter, and trailing-slash behavior to
* Router Core instead of duplicating pathname comparisons in navigation UI.
*/
function useRouteActive(to, options = {}) {
	const router = requireDataRouter();
	const exact = options.exact ?? to === "/";
	return createMemo(() => router.matchRoute({ to }, {
		fuzzy: !exact,
		includeSearch: options.includeSearch ?? false,
		pending: options.pending
	}) !== false);
}
function useParams() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.params ?? {});
}
function useLoaderData() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.loaderData);
}
//#endregion
export { BaseRootRoute, BaseRoute, RouterProvider, createDataRouter, createMemoryHistory$1 as createMemoryHistory, notFound, redirect, useLoaderData, useLocation, useNavigate, useParams, useRouteActive, useRouter, useRouterState };

//# sourceMappingURL=data.mjs.map