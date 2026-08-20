import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import { DownloadsProvider } from "./downloads";
import { DashboardPage } from "./pages/dashboard";
import { DownloadsPage } from "./pages/downloads";
import { NotificationsPage } from "./pages/notifications";
import { SettingsPage } from "./pages/settings";
import { AppShell } from "./shell";

const root = new BaseRootRoute({ component: AppShell });
const routes = [
  new BaseRoute({
    getParentRoute: () => root,
    path: "/",
    component: DashboardPage,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "downloads",
    component: DownloadsPage,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "settings",
    component: SettingsPage,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "notifications",
    component: NotificationsPage,
  }),
];
const router = createDataRouter({
  routeTree: root.addChildren(routes),
  context: {},
});

mount(() => (
  <DownloadsProvider>
    <RouterProvider router={router} />
  </DownloadsProvider>
));
