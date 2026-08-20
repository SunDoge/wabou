import { expect, test } from "bun:test";
import { createMemoryHistory } from "@tanstack/history";
import { isServer, mount } from "@wabou/core/renderer";
import {
  createComponent,
  createContext,
  createEffect,
  createRoot,
  flush,
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

    function Layout(props: { children?: unknown }) {
      rootMounts++;
      projectsActive = useRouteActive("/projects");
      settingsActive = useRouteActive("/settings", { exact: true });
      return props.children as never;
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
  "context outside RouterProvider reaches nested route components",
  async () => {
    // Regression for app-shell provider crashes (e.g. a theme provider): route
    // components arrive as `props.children` that the data router instantiates
    // inside RouteMatch, so their Solid owner chain ends at RouteMatch — not at
    // the root route component. A provider placed inside the root route is
    // therefore invisible to nested routes; app-level providers must wrap
    // <RouterProvider> instead.
    const Ctx = createContext<string>("none");
    const seen: string[] = [];
    function Root(props: { children?: unknown }) {
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
        // Use a getter so RouterProvider is created while the outer Ctx is
        // the current owner — a plain `children: createComponent(...)` would
        // create it eagerly under the mount root, bypassing the provider
        // (the same owner-chain trap that outlet creation hits in RouteMatch).
        get children() {
          return createComponent(RouterProvider, { router });
        },
      }),
    );
    await settle();

    // The nested route reads the OUTER provider. The one inside Root never
    // reaches it, because Leaf's owner chain does not pass through Root.
    expect(seen).toContain("outer");
    expect(seen).not.toContain("inside-root");
    dispose();
  },
);
