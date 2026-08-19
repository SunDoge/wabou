import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import "virtual:wabou-stylesheet";
import { App } from "./app";

const root = new BaseRootRoute({ component: App });
const index = new BaseRoute({ getParentRoute: () => root, path: "/" });
const task = new BaseRoute({ getParentRoute: () => root, path: "$task" });
const router = createDataRouter({
  routeTree: root.addChildren([index, task]),
  context: {},
});

mount(() => <RouterProvider router={router} />);
