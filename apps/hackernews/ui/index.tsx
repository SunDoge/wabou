// Hacker News UI entry.
import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import { AppShell } from "./AppShell";
import { ThemeProvider } from "./contexts/ThemeContext";
import { StoryDetail } from "./pages/StoryDetail";
import { StoryList } from "./pages/StoryList";

// Theme chrome wraps the whole tree. A provider may also wrap `{props.children}`
// in this root route — RouteMatch creates the outlet on that read.
function Root(props: { children?: import("solid-js").JSX.Element }) {
  return <AppShell>{props.children}</AppShell>;
}

const root = new BaseRootRoute({ component: Root });
const stories = new BaseRoute({
  getParentRoute: () => root,
  path: "/",
  component: StoryList,
});
const detail = new BaseRoute({
  getParentRoute: () => root,
  path: "story/$id",
  component: StoryDetail,
});
const router = createDataRouter({
  routeTree: root.addChildren([stories, detail]),
  context: {},
});

mount(() => (
  <ThemeProvider>
    <RouterProvider router={router} />
  </ThemeProvider>
));
