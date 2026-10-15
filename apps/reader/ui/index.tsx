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
import { AppShell } from "./app-shell";
import { LibraryPage } from "./library-page";
import { ReaderPage } from "./reader-page";

function Root(props: { children?: JSX.Element }) {
  return <AppShell>{props.children}</AppShell>;
}

const root = new BaseRootRoute({ component: Root });
const reader = new BaseRoute({
  getParentRoute: () => root,
  path: "/",
  component: ReaderPage,
});
const library = new BaseRoute({
  getParentRoute: () => root,
  path: "library",
  component: LibraryPage,
});
const router = createDataRouter({
  routeTree: root.addChildren([reader, library]),
  context: {},
});

mount(() => (
  <ColorThemeProvider theme="light" transition={false}>
    <ComponentsProvider theme="light">
      <RouterProvider router={router} />
    </ComponentsProvider>
  </ColorThemeProvider>
));
