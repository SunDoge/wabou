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
  useWindow,
} from "@wabou/ui";
import { App } from "./app";
import { AppErrorBoundary } from "./app-error-boundary";

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

function ThemedApp() {
  const window = useWindow();
  const theme = () => window.colorScheme();
  return (
    <ColorThemeProvider theme={theme()} transition={false}>
      <ComponentsProvider theme={theme()}>
        <AppErrorBoundary>
          <RouterProvider router={router} />
        </AppErrorBoundary>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

mount(() => <ThemedApp />);
