import { createMemoryHistory } from "@tanstack/history";
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

if (!("AbortController" in globalThis)) {
  class ExperimentAbortSignal {
    aborted = false;
    reason: unknown;
    readonly listeners = new Set<() => void>();

    addEventListener(type: string, listener: () => void) {
      if (type === "abort") this.listeners.add(listener);
    }

    removeEventListener(type: string, listener: () => void) {
      if (type === "abort") this.listeners.delete(listener);
    }
  }

  class ExperimentAbortController {
    readonly signal = new ExperimentAbortSignal();

    abort(reason?: unknown) {
      if (this.signal.aborted) return;
      this.signal.aborted = true;
      this.signal.reason = reason;
      for (const listener of this.signal.listeners) listener();
    }
  }

  Object.defineProperty(globalThis, "AbortController", {
    value: ExperimentAbortController,
  });
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
      // Native Wabou owns presentation, scroll restoration and document
      // lifecycle, so Router Core runs without its browser coordinator.
      isServer: true,
      origin: "wabou://app",
      defaultPendingMs: 0,
    },
    solidStores,
  );

  await router.load();
  flush();
  const initialMatch = router.state.matches.at(-1);
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
  const navigatedMatch = router.state.matches.at(-1);
  const navigated = {
    pathname: router.state.location.pathname,
    loaderData: navigatedMatch?.loaderData,
  };

  history.back();
  await router.load();
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
