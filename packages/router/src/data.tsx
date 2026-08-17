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
  useContext,
} from "solid-js";

// Router Core resets browser scroll after navigation. Native scroll areas own
// that policy explicitly, so the compatibility global is intentionally inert.
globalThis.scrollTo ??= () => {};

function createMutableStore<T>(initial: T): RouterWritableStore<T> {
  const [read, write] = createSignal<T>(() => initial);
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

function renderMatches(router: AnyRouter, index = 0): JSX.Element {
  const match = router.state.matches[index];
  if (!match) return null;
  const route = router.routesById[match.routeId];
  const options = route.options as typeof route.options & {
    component?: RouteView;
    pendingComponent?: RouteView;
    errorComponent?: RouteView;
    notFoundComponent?: RouteView;
  };
  const view =
    match.status === "error"
      ? options.errorComponent
      : match.status === "notFound"
        ? options.notFoundComponent
        : match.status === "pending"
          ? options.pendingComponent
          : options.component;
  const outlet = () => renderMatches(router, index + 1);
  if (!view) return outlet();
  return createComponent(view, {
    get error() {
      return match.error;
    },
    get children() {
      return outlet();
    },
  });
}

export interface RouterProviderProps {
  router: AnyRouter;
  fallback?: JSX.Element;
}

/** Own router lifecycle and render its current native component branch. */
export function RouterProvider(props: RouterProviderProps): JSX.Element {
  const router = props.router;
  const unsubscribe = router.history.subscribe(() => {
    void router.load().catch(console.error);
  });
  onCleanup(unsubscribe);
  void router.load().catch(console.error);
  const content = () =>
    router.state.matches.length > 0
      ? renderMatches(router)
      : (props.fallback ?? null);
  return createComponent(DataRouterContext, {
    value: router,
    get children() {
      return content as unknown as JSX.Element;
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
