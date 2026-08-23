import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
  View,
} from "@wabou/ui";
import "virtual:wabou-stylesheet";
import type { JSX } from "solid-js";
import { About } from "./about";
import { Reader } from "./reader";

function App(props: { children?: JSX.Element }) {
  return (
    <View class="w-full h-full min-w-0 min-h-0 bg-canvas text-primary">
      {props.children}
    </View>
  );
}

const root = new BaseRootRoute({ component: App });
const reader = new BaseRoute({
  getParentRoute: () => root,
  path: "/",
  component: Reader,
});
const about = new BaseRoute({
  getParentRoute: () => root,
  path: "about",
  component: About,
});
const router = createDataRouter({
  routeTree: root.addChildren([reader, about]),
  context: {},
});

mount(() => <RouterProvider router={router} />);
