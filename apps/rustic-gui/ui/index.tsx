import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import "virtual:wabou-stylesheet";
import { RusticSessionProvider } from "./session";
import { SetupPage } from "./setup";
import { AppShell } from "./shell";
import { SnapshotsPage } from "./snapshots";

const root = new BaseRootRoute({ component: AppShell });
const setup = new BaseRoute({
  getParentRoute: () => root,
  path: "/",
  component: SetupPage,
});
const snapshots = new BaseRoute({
  getParentRoute: () => root,
  path: "snapshots",
  component: SnapshotsPage,
});
const router = createDataRouter({
  routeTree: root.addChildren([setup, snapshots]),
  context: {},
});

mount(() => (
  <RusticSessionProvider>
    <RouterProvider router={router} />
  </RusticSessionProvider>
));
