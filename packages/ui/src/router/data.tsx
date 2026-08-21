import { createMemoryHistory, type RouterHistory } from "@tanstack/history";
import {
  type AnyRoute,
  type AnyRouter,
  BaseRootRoute,
  BaseRoute,
  type GetStoreConfig,
  type RouterConstructorOptions,
  RouterCore,
  type RouterReadableStore,
  type RouterWritableStore,
  type TrailingSlashOption,
} from "@tanstack/router-core";
import {
  type Accessor,
  type Component,
  createComponent,
  createContext,
  createMemo,
  createSignal,
  flush,
  getOwner,
  type JSX,
  onCleanup,
  Show,
  untrack,
  useContext,
} from "solid-js";

// Router Core resets browser scroll after navigation. Native scroll areas own
// that policy explicitly, so the compatibility global is intentionally inert.
globalThis.scrollTo ??= () => {};

function createMutableStore<T>(initial: T): RouterWritableStore<T> {
  // Router Core owns this store and may publish a navigation while a Solid
  // component/event owner is current. Solid 2 treats that as an accidental
  // cross-owner write unless the adapter declares the write intentional.
  const [read, write] = createSignal<T>(() => initial, { ownedWrite: true });
  return {
    get: read,
    set(next: T | ((previous: T) => T)) {
      const value =
        typeof next === "function"
          ? (next as (previous: T) => T)(read())
          : next;
      write(() => value);
    },
  };
}

function createReadonlyStore<T>(read: () => T): RouterReadableStore<T> {
  return { get: read };
}

const solidStores: GetStoreConfig = () => ({
  createMutableStore,
  createReadonlyStore,
  batch: flush,
});

export type WabouDataRouter<TRouteTree extends AnyRoute = AnyRoute> =
  RouterCore<TRouteTree, TrailingSlashOption, boolean, RouterHistory>;

/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
export function createDataRouter<
  TRouteTree extends AnyRoute,
  TTrailingSlash extends TrailingSlashOption = "never",
  TStructuralSharing extends boolean = false,
  THistory extends RouterHistory = RouterHistory,
  // biome-ignore lint/suspicious/noExplicitAny: mirrors Router Core's serialized-data constraint
  TDehydrated extends Record<string, any> = Record<string, any>,
>(
  options: RouterConstructorOptions<
    TRouteTree,
    TTrailingSlash,
    TStructuralSharing,
    THistory,
    TDehydrated
  >,
): RouterCore<
  TRouteTree,
  TTrailingSlash,
  TStructuralSharing,
  THistory,
  TDehydrated
> {
  const history =
    options.history ?? createMemoryHistory({ initialEntries: ["/"] });
  const router = new RouterCore(
    {
      isServer: false,
      origin: "wabou://app",
      ...options,
      history,
    } as RouterConstructorOptions<
      TRouteTree,
      TTrailingSlash,
      TStructuralSharing,
      THistory,
      TDehydrated
    >,
    solidStores,
  );
  router.startTransition = async (commit) => {
    flush(commit);
    return true;
  };
  return router;
}

const DataRouterContext = createContext<AnyRouter>();

function requireDataRouter(): AnyRouter {
  if (!getOwner()) {
    throw new Error(
      "Wabou data-router hooks must be used inside <RouterProvider>",
    );
  }
  const router = useContext(DataRouterContext);
  if (!router) {
    throw new Error(
      "Wabou data-router hooks must be used inside <RouterProvider>",
    );
  }
  return router;
}

type RouteView = Component<{
  children?: JSX.Element;
  error?: unknown;
}>;

function matchView(router: AnyRouter, index: number): RouteView | undefined {
  const match = router.state.matches[index];
  if (!match) return undefined;
  const route = router.routesById[match.routeId];
  const options = route.options as typeof route.options & {
    component?: RouteView;
    pendingComponent?: RouteView;
    errorComponent?: RouteView;
    notFoundComponent?: RouteView;
  };
  return match.status === "error"
    ? options.errorComponent
    : match.status === "notFound"
      ? options.notFoundComponent
      : match.status === "pending"
        ? options.pendingComponent
        : options.component;
}

interface RouteMatchProps {
  router: AnyRouter;
  index: number;
}

/** Preserve a matched component while its route and selected view are stable. */
function RouteMatch(props: RouteMatchProps): JSX.Element {
  const match = () => props.router.state.matches[props.index];
  // Create the child outlet on first `props.children` read, not here. Eager
  // createComponent would own the nested route under RouteMatch, so a
  // provider in this view would not be visible to child routes. Caching the
  // instance still keeps the child tree stable across view remounts.
  let outlet: JSX.Element | undefined;
  return createComponent(
    Show as unknown as (props: {
      when: RouteView | undefined;
      keyed: true;
      children: (view: RouteView) => JSX.Element;
    }) => JSX.Element,
    {
      get when() {
        return matchView(props.router, props.index);
      },
      keyed: true,
      children: (view: RouteView) =>
        createComponent(view, {
          get error() {
            return match()?.error;
          },
          get children() {
            if (outlet === undefined) {
              outlet = createComponent(RouteOutlet, {
                router: props.router,
                index: props.index + 1,
              });
            }
            return outlet;
          },
        }),
    },
  );
}

interface RouteOutletProps {
  router: AnyRouter;
  index: number;
  fallback?: JSX.Element;
}

/** Key only the route level that changed instead of rebuilding all matches. */
function RouteOutlet(props: RouteOutletProps): JSX.Element {
  return createComponent(
    Show as unknown as (props: {
      when: string | undefined;
      keyed: true;
      fallback?: JSX.Element;
      children: (routeId: string) => JSX.Element;
    }) => JSX.Element,
    {
      get when() {
        return props.router.state.matches[props.index]?.routeId;
      },
      keyed: true,
      get fallback() {
        return props.fallback ?? null;
      },
      children: (_routeId: string) =>
        createComponent(RouteMatch, {
          router: props.router,
          index: props.index,
        }),
    },
  );
}

export interface RouterProviderProps {
  router: AnyRouter;
  fallback?: JSX.Element;
}

/** Own router lifecycle and render its current native component branch. */
export function RouterProvider(props: RouterProviderProps): JSX.Element {
  // The provider owns one router for its lifetime; reading that invariant prop
  // reactively would only produce Solid 2's untracked-read diagnostic.
  const router = untrack(() => props.router);
  let disposed = false;
  let loadScheduled = false;
  const scheduleLoad = () => {
    if (disposed || loadScheduled) return;
    loadScheduled = true;
    // Router Core reads and publishes its external store during load(). Run it
    // after component construction so Solid does not mistake those reads for
    // untracked component dependencies. Multiple synchronous history events
    // collapse into one load of the latest location.
    void Promise.resolve()
      .then(async () => {
        loadScheduled = false;
        if (!disposed) await router.load();
      })
      .catch((error) => {
        console.error(`[wabou-router] route load failed: ${String(error)}`);
      });
  };
  const unsubscribe = router.history.subscribe(scheduleLoad);
  onCleanup(() => {
    disposed = true;
    unsubscribe();
  });
  scheduleLoad();
  return createComponent(DataRouterContext, {
    value: router,
    get children() {
      return createComponent(RouteOutlet, {
        router,
        index: 0,
        get fallback() {
          return props.fallback;
        },
      });
    },
  });
}

export function useRouter(): AnyRouter {
  return requireDataRouter();
}

export function useRouterState<T>(
  selector: (router: AnyRouter) => T,
): Accessor<T> {
  const router = requireDataRouter();
  return createMemo(() => selector(router));
}

export function useNavigate(): AnyRouter["navigate"] {
  return requireDataRouter().navigate;
}

export function useLocation(): Accessor<AnyRouter["state"]["location"]> {
  const router = requireDataRouter();
  return createMemo(() => router.state.location);
}

export interface RouteActiveOptions {
  /** Match only this path instead of descendant routes. Defaults to true for `/`. */
  exact?: boolean;
  /** Include the target's search parameters in the match. Defaults to false. */
  includeSearch?: boolean;
  /** Match the pending destination while navigation is loading. */
  pending?: boolean;
}

/**
 * Reactively report whether a native router destination is active.
 *
 * This delegates path, base-path, parameter, and trailing-slash behavior to
 * Router Core instead of duplicating pathname comparisons in navigation UI.
 */
export function useRouteActive(
  to: string,
  options: RouteActiveOptions = {},
): Accessor<boolean> {
  const router = requireDataRouter();
  const exact = options.exact ?? to === "/";
  return createMemo(
    () =>
      router.matchRoute({ to } as never, {
        fuzzy: !exact,
        includeSearch: options.includeSearch ?? false,
        pending: options.pending,
      }) !== false,
  );
}

export function useParams<
  T extends Record<string, string> = Record<string, string>,
>(): Accessor<T> {
  const router = requireDataRouter();
  return createMemo(() => (router.state.matches.at(-1)?.params ?? {}) as T);
}

export function useLoaderData<T = unknown>(): Accessor<T | undefined> {
  const router = requireDataRouter();
  return createMemo(
    () => router.state.matches.at(-1)?.loaderData as T | undefined,
  );
}

export { createMemoryHistory } from "@tanstack/history";
export { notFound, redirect } from "@tanstack/router-core";
export { BaseRootRoute, BaseRoute };
