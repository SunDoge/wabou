# Wabou router implementation

This is the routing layer inside the public `@wabou/ui` package.
Applications import router APIs from `@wabou/ui`.

Solid 2 routing for Wabou applications, backed by TanStack Router Core. Wabou
owns the reactive store adapter, memory history, component rendering, and
native navigation lifecycle; no browser history or DOM router is involved.

```tsx
import {
  BaseRootRoute,
  BaseRoute,
  createMemoryHistory,
  createDataRouter,
  RouterProvider,
  useLoaderData,
} from "@wabou/ui";

const root = new BaseRootRoute();
const project = new BaseRoute({
  getParentRoute: () => root,
  path: "projects/$projectId",
  loader: ({ params }) => loadProject(params.projectId),
  component: Project,
});
const router = createDataRouter({
  routeTree: root.addChildren([project]),
  history: createMemoryHistory({ initialEntries: ["/projects/alpha"] }),
  context: {},
});

function Project() {
  const project = useLoaderData<Project>();
  return <Text>{project()?.name}</Text>;
}

mount(() => <RouterProvider router={router} />);
```

The router supports typed search and parameters, asynchronous loaders,
preloading, caching, guards, pending/error states, redirects, blockers, and
back/forward memory navigation. Native links, blocker presentation, and scroll
restoration remain explicit Wabou UI concerns.
