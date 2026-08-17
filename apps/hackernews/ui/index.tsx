// Hacker News UI entry.
import "@wabou/core";
import "virtual:wabou-stylesheet";
import { mount } from "@wabou/core";
import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  RouterProvider,
} from "@wabou/router";
import { AppShell } from "./AppShell";
import { ThemeProvider } from "./contexts/ThemeContext";
import { StoryDetail } from "./pages/StoryDetail";
import { StoryList } from "./pages/StoryList";

function Root(props: { children?: import("solid-js").JSX.Element }) {
  return (
    <ThemeProvider>
      <AppShell>{props.children}</AppShell>
    </ThemeProvider>
  );
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

mount(() => <RouterProvider router={router} />);
