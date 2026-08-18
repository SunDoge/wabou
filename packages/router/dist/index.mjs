import { createMemoryHistory, createMemoryHistory as createMemoryHistory$1 } from "@tanstack/history";
import { BaseRootRoute, BaseRoute, RouterCore, notFound, redirect } from "@tanstack/router-core";
import { Show, createComponent, createContext, createMemo, createSignal, flush, getOwner, onCleanup, useContext } from "solid-js";
//#region src/data.tsx
globalThis.scrollTo ??= () => {};
function createMutableStore(initial) {
	const [read, write] = createSignal(() => initial);
	return {
		get: read,
		set(next) {
			const value = typeof next === "function" ? next(read()) : next;
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
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
function createDataRouter(options) {
	const history = options.history ?? createMemoryHistory$1({ initialEntries: ["/"] });
	const router = new RouterCore({
		isServer: false,
		origin: "wabou://app",
		...options,
		history
	}, solidStores);
	router.startTransition = async (commit) => {
		flush(commit);
		return true;
	};
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
	const outlet = createComponent(RouteOutlet, {
		router: props.router,
		index: props.index + 1
	});
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
	const router = props.router;
	const unsubscribe = router.history.subscribe(() => {
		router.load().catch(console.error);
	});
	onCleanup(unsubscribe);
	router.load().catch(console.error);
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
function useParams() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.params ?? {});
}
function useLoaderData() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.loaderData);
}
//#endregion
export { BaseRootRoute, BaseRoute, RouterProvider, createDataRouter, createMemoryHistory, notFound, redirect, useLoaderData, useLocation, useNavigate, useParams, useRouter, useRouterState };

//# sourceMappingURL=index.mjs.map