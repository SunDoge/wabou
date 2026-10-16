import { Kv, KvKey } from "@wabou/core";
import { Accessor, JSX } from "solid-js";
import { RouterHistory, createMemoryHistory as createMemoryHistory$1 } from "@tanstack/history";
import { AnyRoute, AnyRouter, BaseRootRoute, BaseRoute, RouterConstructorOptions, RouterCore, TrailingSlashOption, notFound, redirect } from "@tanstack/router-core";
//#region src/router/data.d.ts
export type WabouDataRouter<TRouteTree extends AnyRoute = AnyRoute> = RouterCore<TRouteTree, TrailingSlashOption, boolean, RouterHistory>;
export interface RouterPersistenceOptions {
  /** Application-scoped KV handle. The host must opt in with `HostBuilder::kv()`. */
  kv: Kv;
  /** Explicit hierarchical identity for this router. */
  key: KvKey;
  /** Ignore locations written by another schema version. Defaults to 1. */
  version?: number;
}
export type WabouRouterConstructorOptions<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption, TStructuralSharing extends boolean, THistory extends RouterHistory, TDehydrated extends Record<string, any>> = RouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated> & {
  /** Optionally restore and save the last committed native location. */
  persistence?: RouterPersistenceOptions;
};
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
export declare function createDataRouter<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption = "never", TStructuralSharing extends boolean = false, THistory extends RouterHistory = RouterHistory, TDehydrated extends Record<string, any> = Record<string, any>>(options: WabouRouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>): RouterCore<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>;
export interface RouterProviderProps {
  router: AnyRouter;
  fallback?: JSX.Element;
}
/** Own router lifecycle and render its current native component branch. */
export declare function RouterProvider(props: RouterProviderProps): JSX.Element;
export declare function useRouter(): AnyRouter;
export declare function useRouterState<T>(selector: (router: AnyRouter) => T): Accessor<T>;
export declare function useNavigate(): AnyRouter["navigate"];
export declare function useLocation(): Accessor<AnyRouter["state"]["location"]>;
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
export declare function useRouteActive(to: string, options?: RouteActiveOptions): Accessor<boolean>;
export declare function useParams<T extends Record<string, string> = Record<string, string>>(): Accessor<T>;
export declare function useLoaderData<T = unknown>(): Accessor<T | undefined>;
//#endregion
export { BaseRootRoute, BaseRoute, createMemoryHistory$1 as createMemoryHistory, notFound, redirect };
//# sourceMappingURL=data.d.mts.map