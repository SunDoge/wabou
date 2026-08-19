import { expect, test } from "bun:test";
import { createMemoryHistory } from "@tanstack/history";
import { isServer, mount } from "@wabou/core/renderer";
import { createComponent, createEffect, createRoot, flush } from "solid-js";
import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  RouterProvider,
  useLoaderData,
  useLocation,
  useNavigate,
  useParams,
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

    function Layout(props: { children?: unknown }) {
      rootMounts++;
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
    expect(rootMounts).toBe(1);

    await navigate?.({ to: "/settings" });
    await settle();
    expect(seen).toContain("/settings");
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
