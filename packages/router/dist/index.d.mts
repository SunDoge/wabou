import { RouterHistory, createMemoryHistory } from "@tanstack/history";
import { AnyRoute, AnyRouter, BaseRootRoute, BaseRoute, RouterConstructorOptions, RouterCore, TrailingSlashOption, notFound, redirect } from "@tanstack/router-core";
import { Accessor, JSX } from "solid-js";
//#region src/data.d.ts
type WabouDataRouter<TRouteTree extends AnyRoute = AnyRoute> = RouterCore<TRouteTree, TrailingSlashOption, boolean, RouterHistory>;
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
declare function createDataRouter<TRouteTree extends AnyRoute, TTrailingSlash extends TrailingSlashOption = "never", TStructuralSharing extends boolean = false, THistory extends RouterHistory = RouterHistory, TDehydrated extends Record<string, any> = Record<string, any>>(options: RouterConstructorOptions<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>): RouterCore<TRouteTree, TTrailingSlash, TStructuralSharing, THistory, TDehydrated>;
interface RouterProviderProps {
  router: AnyRouter;
  fallback?: JSX.Element;
}
/** Own router lifecycle and render its current native component branch. */
declare function RouterProvider(props: RouterProviderProps): JSX.Element;
declare function useRouter(): AnyRouter;
declare function useRouterState<T>(selector: (router: AnyRouter) => T): Accessor<T>;
declare function useNavigate(): AnyRouter["navigate"];
declare function useLocation(): Accessor<AnyRouter["state"]["location"]>;
declare function useParams<T extends Record<string, string> = Record<string, string>>(): Accessor<T>;
declare function useLoaderData<T = unknown>(): Accessor<T | undefined>;
//#endregion
export { BaseRootRoute, BaseRoute, RouterProvider, RouterProviderProps, WabouDataRouter, createDataRouter, createMemoryHistory, notFound, redirect, useLoaderData, useLocation, useNavigate, useParams, useRouter, useRouterState };
//# sourceMappingURL=index.d.mts.map