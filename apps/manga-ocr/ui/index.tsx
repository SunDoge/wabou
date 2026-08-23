import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import "virtual:wabou-stylesheet";
import { About } from "./about";
import { Reader } from "./reader";
import { MangaSessionProvider } from "./session";
import { Settings } from "./settings";
import { AppShell } from "./shell";
import { Starter } from "./starter";

const root = new BaseRootRoute({ component: AppShell });
const starter = new BaseRoute({
  getParentRoute: () => root,
  path: "/",
  component: Starter,
});
const reader = new BaseRoute({
  getParentRoute: () => root,
  path: "reader",
  component: Reader,
});
const settings = new BaseRoute({
  getParentRoute: () => root,
  path: "settings",
  component: Settings,
});
const about = new BaseRoute({
  getParentRoute: () => root,
  path: "about",
  component: About,
});
const router = createDataRouter({
  routeTree: root.addChildren([starter, reader, settings, about]),
  context: {},
});

mount(() => (
  <MangaSessionProvider>
    <RouterProvider router={router} />
  </MangaSessionProvider>
));
