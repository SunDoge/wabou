import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  BaseRootRoute,
  BaseRoute,
  ColorThemeProvider,
  ComponentsProvider,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import type { JSX } from "solid-js";
import { AppShell, OverviewPage, PlaceholderPage } from "./app";
import bot from "lucide-static/icons/bot.svg?raw";
import database from "lucide-static/icons/database.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import list from "lucide-static/icons/list.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";

function Root(props: { children?: JSX.Element }): JSX.Element {
  return <AppShell>{props.children}</AppShell>;
}

const root = new BaseRootRoute({ component: Root });
const overview = new BaseRoute({
  getParentRoute: () => root,
  path: "/",
  component: OverviewPage,
});
const tasks = new BaseRoute({
  getParentRoute: () => root,
  path: "tasks",
  component: () => (
    <PlaceholderPage
      title="Tasks"
      description="A compact example of application state and native persistence."
      icon={list}
    />
  ),
});
const data = new BaseRoute({
  getParentRoute: () => root,
  path: "database",
  component: () => (
    <PlaceholderPage
      title="Database"
      description="Inspect local data without exposing storage details to the interface."
      icon={database}
    />
  ),
});
const agent = new BaseRoute({
  getParentRoute: () => root,
  path: "agent",
  component: () => (
    <PlaceholderPage
      title="Agent"
      description="Keep generated changes visible and approval-first."
      icon={bot}
    />
  ),
});
const preferences = new BaseRoute({
  getParentRoute: () => root,
  path: "settings",
  component: () => (
    <PlaceholderPage
      title="Settings"
      description="Global application preferences belong in one predictable place."
      icon={settings}
    />
  ),
});
const about = new BaseRoute({
  getParentRoute: () => root,
  path: "about",
  component: () => (
    <PlaceholderPage
      title="About"
      description="A native retained UI experiment driven by Solid and rendered by Vello Hybrid."
      icon={info}
    />
  ),
});

const router = createDataRouter({
  routeTree: root.addChildren([
    overview,
    tasks,
    data,
    agent,
    preferences,
    about,
  ]),
  context: {},
});

mount(() => (
  <ColorThemeProvider theme="light" transition={false}>
    <ComponentsProvider theme="light">
      <RouterProvider router={router} />
    </ComponentsProvider>
  </ColorThemeProvider>
));
