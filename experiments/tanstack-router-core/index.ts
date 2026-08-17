import { createMemoryHistory } from "@tanstack/history";
import "../../packages/core/src/prelude/bootstrap";
import "../../packages/core/src/polyfills/abort-controller";
import "../../packages/core/src/polyfills/fetch";
import "../../packages/core/src/glue/timers";
import {
  BaseRootRoute,
  BaseRoute,
  type GetStoreConfig,
  RouterCore,
  type RouterReadableStore,
  type RouterWritableStore,
} from "@tanstack/router-core";
import { createSignal, flush } from "solid-js";

// Browser-targeted Router Core records a debug handle on `self`. Wabou has a
// browser-like global object but deliberately does not expose a DOM/Worker
// global alias, so the adapter owns this one compatibility shim.
if (!("self" in globalThis)) {
  Object.defineProperty(globalThis, "self", { value: globalThis });
}

function mutableStore<T>(initial: T): RouterWritableStore<T> {
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

function readonlyStore<T>(read: () => T): RouterReadableStore<T> {
  return { get: read };
}

const solidStores: GetStoreConfig = () => ({
  createMutableStore: mutableStore,
  createReadonlyStore: readonlyStore,
  // Solid 2 renamed the synchronous batching boundary from `batch` to `flush`.
  // Router Core only requires the callback to run as one store transaction.
  batch: flush,
});

const rootRoute = new BaseRootRoute({
  validateSearch: (search: Record<string, unknown>) => ({
    page: Math.max(1, Number(search.page) || 1),
  }),
});

const projectRoute = new BaseRoute({
  getParentRoute: () => rootRoute,
  path: "projects/$projectId",
  loaderDeps: ({ search }) => ({ page: search.page }),
  beforeLoad: ({ params }) => ({ projectKey: `project:${params.projectId}` }),
  loader: async ({ context, deps, params }) => ({
    id: params.projectId,
    page: deps.page,
    key: context.projectKey,
  }),
});

const settingsRoute = new BaseRoute({
  getParentRoute: () => rootRoute,
  path: "settings",
  loader: () => ({ section: "general" }),
});

const routeTree = rootRoute.addChildren([projectRoute, settingsRoute]);

async function waitForPublishedMatch(
  router: RouterCore<typeof routeTree>,
  predicate: (match: (typeof router.state.matches)[number]) => boolean,
) {
  for (let attempt = 0; attempt < 100; attempt++) {
    flush();
    const match = router.state.matches.at(-1);
    if (match && predicate(match)) return match;
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
  }
  throw new Error("Router Core did not publish a matching route in time");
}

export interface ExperimentResult {
  compatible: boolean;
  diagnostics: unknown;
  initial: {
    pathname: string;
    search: unknown;
    params: unknown;
    loaderData: unknown;
  };
  navigated: {
    pathname: string;
    loaderData: unknown;
  };
  restored: {
    pathname: string;
    canGoBack: boolean;
  };
}

export async function runExperiment(): Promise<ExperimentResult> {
  const history = createMemoryHistory({
    initialEntries: ["/projects/alpha?page=2"],
  });
  const router = new RouterCore(
    {
      routeTree,
      history,
      context: {},
      isServer: false,
      origin: "wabou://app",
      defaultPendingMs: 0,
    },
    solidStores,
  );
  router.startTransition = async (commit) => {
    flush(commit);
    return true;
  };

  await router.load();
  const initialMatch = await waitForPublishedMatch(
    router,
    (match) =>
      (match.loaderData as { key?: string } | undefined)?.key ===
      "project:alpha",
  );
  const diagnostics = router.state.matches.map((match) => ({
    routeId: match.routeId,
    status: match.status,
    params: match.params,
    loaderData: match.loaderData,
  }));
  const initial = {
    pathname: router.state.location.pathname,
    search: router.state.location.search,
    params: initialMatch?.params,
    loaderData: initialMatch?.loaderData,
  };

  await router.navigate({ to: "/settings" });
  const navigatedMatch = await waitForPublishedMatch(
    router,
    (match) =>
      (match.loaderData as { section?: string } | undefined)?.section ===
      "general",
  );
  const navigated = {
    pathname: router.state.location.pathname,
    loaderData: navigatedMatch?.loaderData,
  };

  history.back();
  await router.load();
  await waitForPublishedMatch(
    router,
    (match) =>
      (match.loaderData as { key?: string } | undefined)?.key ===
      "project:alpha",
  );
  const restored = {
    pathname: router.state.location.pathname,
    canGoBack: history.canGoBack(),
  };

  return {
    compatible:
      diagnostics.length === 2 &&
      (initial.loaderData as { key?: string } | undefined)?.key ===
        "project:alpha" &&
      (navigated.loaderData as { section?: string } | undefined)?.section ===
        "general",
    diagnostics,
    initial,
    navigated,
    restored,
  };
}

declare global {
  var __wabouRouterCoreExperiment:
    | ExperimentResult
    | { error: string }
    | undefined;
}

void runExperiment().then(
  (result) => {
    globalThis.__wabouRouterCoreExperiment = result;
  },
  (error) => {
    globalThis.__wabouRouterCoreExperiment = {
      error:
        error instanceof Error
          ? `${error.message}\n${error.stack ?? ""}`
          : String(error),
    };
  },
);
