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

// ThemeProvider lives outside RouterProvider (in `mount`) rather than here.
// Route components arrive as `props.children` that the data router instantiated
// inside RouteMatch; their Solid owner chain ends at RouteMatch, so a provider
// placed here would be invisible to them — useTheme in StoryList would throw.
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
