import { expect, test } from "bun:test";
import { createMemoryHistory } from "@tanstack/history";
import type { Kv, KvValue } from "@wabou/core";
import { isServer, mount } from "@wabou/core/renderer";
import {
  createComponent,
  createContext,
  createEffect,
  createRoot,
  flush,
  type JSX,
  onCleanup,
  useContext,
} from "solid-js";
import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  RouterProvider,
  useLoaderData,
  useLocation,
  useNavigate,
  useParams,
  useRouteActive,
  useRouter,
} from "./data";

async function settle(): Promise<void> {
  for (let attempt = 0; attempt < 100; attempt++) {
    flush();
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
  }
}

test.skipIf(isServer)(
  "data router publishes params, loader data, and navigation through Solid",
  async () => {
    const seen: string[] = [];
    let rootMounts = 0;
    let navigate: ReturnType<typeof useNavigate> | undefined;
    let projectsActive: (() => boolean) | undefined;
    let settingsActive: (() => boolean) | undefined;

    function Layout(props: { children?: JSX.Element }) {
      rootMounts++;
      projectsActive = useRouteActive("/projects");
      settingsActive = useRouteActive("/settings", { exact: true });
      return props.children;
    }

    function Project() {
      const location = useLocation();
      const params = useParams<{ projectId: string }>();
      const data = useLoaderData<{ id: string; page: number }>();
      navigate = useNavigate();
      createEffect(
        () => `${location().pathname}:${params().projectId}:${data()?.page}`,
        (value) => {
          seen.push(value);
        },
      );
      return null;
    }

    function Settings() {
      const location = useLocation();
      createEffect(
        () => location().pathname,
        (value) => {
          seen.push(value);
        },
      );
      return null;
    }

    const root = new BaseRootRoute({
      component: Layout,
      validateSearch: (search: Record<string, unknown>) => ({
        page: Number(search.page) || 1,
      }),
    });
    const project = new BaseRoute({
      getParentRoute: () => root,
      path: "projects/$projectId",
      loaderDeps: ({ search }) => ({ page: search.page }),
      loader: async ({ deps, params }) => ({
        id: params.projectId,
        page: deps.page,
      }),
      component: Project,
    });
    const settings = new BaseRoute({
      getParentRoute: () => root,
      path: "settings",
      component: Settings,
    });
    const routeTree = root.addChildren([project, settings]);
    const router = createDataRouter({
      routeTree,
      history: createMemoryHistory({
        initialEntries: ["/projects/alpha?page=2"],
      }),
      context: {},
      defaultPendingMs: 0,
    });

    const dispose = mount(() => createComponent(RouterProvider, { router }));
    await settle();
    expect(seen).toContain("/projects/alpha:alpha:2");
    expect(projectsActive?.()).toBe(true);
    expect(settingsActive?.()).toBe(false);
    expect(rootMounts).toBe(1);

    await navigate?.({ to: "/settings" });
    await settle();
    expect(seen).toContain("/settings");
    expect(projectsActive?.()).toBe(false);
    expect(settingsActive?.()).toBe(true);
    expect(rootMounts).toBe(1);
    dispose();
  },
);

test.skipIf(isServer)(
  "keys route lifetime by route id while params and loader data stay reactive",
  async () => {
    let rootMounts = 0;
    let projectMounts = 0;
    let projectCleanups = 0;
    let settingsMounts = 0;
    let navigate: ReturnType<typeof useNavigate> | undefined;
    const seen: string[] = [];

    function Root(props: { children?: JSX.Element }) {
      rootMounts++;
      return props.children;
    }
    function Project() {
      projectMounts++;
      onCleanup(() => projectCleanups++);
      const params = useParams<{ projectId: string }>();
      const data = useLoaderData<{ revision: string }>();
      navigate = useNavigate();
      createEffect(
        () => `${params().projectId}:${data()?.revision}`,
        (value) => {
          seen.push(value);
        },
      );
      return null;
    }
    function Settings() {
      settingsMounts++;
      navigate = useNavigate();
      return null;
    }

    const root = new BaseRootRoute({ component: Root });
    const project = new BaseRoute({
      getParentRoute: () => root,
      path: "projects/$projectId",
      validateSearch: (search: Record<string, unknown>) => ({
        revision: String(search.revision ?? "initial"),
      }),
      loaderDeps: ({ search }) => ({ revision: search.revision }),
      loader: ({ deps }) => ({ revision: deps.revision }),
      component: Project,
    });
    const settings = new BaseRoute({
      getParentRoute: () => root,
      path: "settings",
      component: Settings,
    });
    const router = createDataRouter({
      routeTree: root.addChildren([project, settings]),
      history: createMemoryHistory({
        initialEntries: ["/projects/alpha?revision=1"],
      }),
      context: {},
      defaultPendingMs: 0,
    });

    const dispose = mount(() => createComponent(RouterProvider, { router }));
    await settle();
    expect(seen).toContain("alpha:1");
    expect(projectMounts).toBe(1);

    await navigate?.({
      to: "/projects/$projectId",
      params: { projectId: "beta" },
      search: { revision: "2" },
    });
    await settle();
    expect(seen).toContain("beta:2");
    expect(projectMounts).toBe(1);
    expect(projectCleanups).toBe(0);

    await navigate?.({ to: "/settings" });
    await settle();
    expect(projectCleanups).toBe(1);
    expect(settingsMounts).toBe(1);

    await navigate?.({
      to: "/projects/$projectId",
      params: { projectId: "gamma" },
      search: { revision: "3" },
    });
    await settle();
    expect(seen).toContain("gamma:3");
    expect(projectMounts).toBe(2);
    expect(rootMounts).toBe(1);
    dispose();
  },
);

test.skipIf(isServer)(
  "commits only the latest route when navigations overlap",
  async () => {
    let releaseSlow: (() => void) | undefined;
    let navigate: ReturnType<typeof useNavigate> | undefined;
    const mounted: string[] = [];

    function Root(props: { children?: JSX.Element }) {
      navigate = useNavigate();
      return props.children;
    }
    const root = new BaseRootRoute({ component: Root });
    const slow = new BaseRoute({
      getParentRoute: () => root,
      path: "slow",
      loader: () =>
        new Promise<string>((resolve) => {
          releaseSlow = () => resolve("slow");
        }),
      component: () => {
        mounted.push("slow");
        return null;
      },
    });
    const fast = new BaseRoute({
      getParentRoute: () => root,
      path: "fast",
      component: () => {
        mounted.push("fast");
        return null;
      },
    });
    const router = createDataRouter({
      routeTree: root.addChildren([slow, fast]),
      history: createMemoryHistory({ initialEntries: ["/fast"] }),
      context: {},
      defaultPendingMs: 0,
    });
    const dispose = mount(() => createComponent(RouterProvider, { router }));
    await settle();

    const slowNavigation = navigate?.({ to: "/slow" });
    await Promise.resolve();
    await navigate?.({ to: "/fast" });
    releaseSlow?.();
    await slowNavigation;
    await settle();

    expect(router.state.location.pathname).toBe("/fast");
    expect(mounted.at(-1)).toBe("fast");
    dispose();
  },
);

test("data-router hooks reject calls outside RouterProvider", () => {
  expect(() => useRouter()).toThrow(
    "Wabou data-router hooks must be used inside <RouterProvider>",
  );
});

test("router-owned stores can publish while a Solid owner is current", async () => {
  const root = new BaseRootRoute({ component: () => null });
  const router = createDataRouter({
    routeTree: root,
    history: createMemoryHistory({ initialEntries: ["/"] }),
    context: {},
  });

  await createRoot(() => router.load());
  expect(router.state.status).toBe("idle");
});

test.skipIf(isServer)(
  "restores and persists the last location through application KV",
  async () => {
    const values = new Map<string, KvValue>([
      ["router/main", { version: 2, href: "/settings?tab=theme" }],
    ]);
    const writes: KvValue[] = [];
    const kv = {
      async get(key: readonly (string | number | boolean | Uint8Array)[]) {
        const value = values.get(key.join("/"));
        return value === undefined ? null : { key, value, versionstamp: "1" };
      },
      async set(
        key: readonly (string | number | boolean | Uint8Array)[],
        value: KvValue,
      ) {
        values.set(key.join("/"), value);
        writes.push(value);
        return String(writes.length + 1);
      },
    } as Kv;

    let navigate: ReturnType<typeof useNavigate> | undefined;
    const seen: string[] = [];
    const root = new BaseRootRoute({
      component: (props: { children?: JSX.Element }) => {
        navigate = useNavigate();
        return props.children;
      },
    });
    const home = new BaseRoute({
      getParentRoute: () => root,
      path: "/",
      component: () => null,
    });
    const settings = new BaseRoute({
      getParentRoute: () => root,
      path: "settings",
      component: () => {
        const location = useLocation();
        createEffect(
          () => location().href,
          (href) => {
            seen.push(href);
          },
        );
        return null;
      },
    });
    const router = createDataRouter({
      routeTree: root.addChildren([home, settings]),
      history: createMemoryHistory({ initialEntries: ["/"] }),
      context: {},
      persistence: { kv, key: ["router", "main"], version: 2 },
    });

    const dispose = mount(() => createComponent(RouterProvider, { router }));
    await settle();
    expect(seen).toContain("/settings?tab=theme");
    expect(writes).toHaveLength(0);

    await navigate?.({ to: "/" });
    await settle();
    expect(writes.at(-1)).toEqual({ version: 2, href: "/" });
    dispose();
  },
);

test.skipIf(isServer)(
  "context from the root route reaches nested route components",
  async () => {
    // RouteMatch creates the child outlet on first `props.children` access so
    // the nested route is owned under the parent view — a provider wrapping
    // `{props.children}` is visible to leaf routes.
    const Ctx = createContext<string>("none");
    const seen: string[] = [];
    function Root(props: { children?: JSX.Element }) {
      return createComponent(Ctx, {
        value: "inside-root",
        get children() {
          return props.children;
        },
      });
    }
    function Leaf() {
      seen.push(useContext(Ctx));
      return null;
    }
    const root = new BaseRootRoute({ component: Root });
    const leaf = new BaseRoute({
      getParentRoute: () => root,
      path: "/",
      component: Leaf,
    });
    const router = createDataRouter({
      routeTree: root.addChildren([leaf]),
      history: createMemoryHistory({ initialEntries: ["/"] }),
      context: {},
    });

    const dispose = mount(() =>
      createComponent(Ctx, {
        value: "outer",
        get children() {
          return createComponent(RouterProvider, { router });
        },
      }),
    );
    await settle();

    expect(seen).toContain("inside-root");
    expect(seen).not.toContain("outer");
    dispose();
  },
);

test.skipIf(isServer)(
  "context outside RouterProvider reaches nested route components",
  async () => {
    const Ctx = createContext<string>("none");
    const seen: string[] = [];
    function Root(props: { children?: JSX.Element }) {
      return props.children;
    }
    function Leaf() {
      seen.push(useContext(Ctx));
      return null;
    }
    const root = new BaseRootRoute({ component: Root });
    const leaf = new BaseRoute({
      getParentRoute: () => root,
      path: "/",
      component: Leaf,
    });
    const router = createDataRouter({
      routeTree: root.addChildren([leaf]),
      history: createMemoryHistory({ initialEntries: ["/"] }),
      context: {},
    });

    const dispose = mount(() =>
      createComponent(Ctx, {
        value: "outer",
        // Use a getter so RouterProvider is created while the outer Ctx is
        // the current owner — a plain `children: createComponent(...)` would
        // create it eagerly under the mount root, bypassing the provider.
        get children() {
          return createComponent(RouterProvider, { router });
        },
      }),
    );
    await settle();

    expect(seen).toContain("outer");
    dispose();
  },
);
