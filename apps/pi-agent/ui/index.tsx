import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import { App } from "./app";

const root = new BaseRootRoute({ component: App });
const index = new BaseRoute({ getParentRoute: () => root, path: "/" });
const agent = new BaseRoute({
  getParentRoute: () => root,
  path: "agents/$agentId",
});
const session = new BaseRoute({
  getParentRoute: () => root,
  path: "agents/$agentId/sessions/$sessionId",
});
const settings = new BaseRoute({
  getParentRoute: () => root,
  path: "settings",
});
const router = createDataRouter({
  routeTree: root.addChildren([index, agent, session, settings]),
  context: {},
});

mount(() => <RouterProvider router={router} />);
