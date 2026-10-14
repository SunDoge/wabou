import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  BaseRootRoute,
  BaseRoute,
  createDataRouter,
  mount,
  RouterProvider,
} from "@wabou/ui";
import type { JSX } from "solid-js";
import { BenchmarkIndex } from "./benchmark-index";
import { DiskTree } from "./disktree";
import { FolioDesk } from "./folio-desk";
import { OceanView } from "./ocean-view";
import { Omanta } from "./omanta";
import { Signed } from "./signed";
import { SkillsManager } from "./skills-manager";
import { Tendra } from "./tendra";

function Root(props: { children?: JSX.Element }) {
  return props.children;
}

const root = new BaseRootRoute({ component: Root });
const routes = [
  new BaseRoute({
    getParentRoute: () => root,
    path: "/",
    component: BenchmarkIndex,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "skills",
    component: SkillsManager,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "folio",
    component: FolioDesk,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "tendra",
    component: Tendra,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "ocean",
    component: OceanView,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "signed",
    component: Signed,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "omanta",
    component: Omanta,
  }),
  new BaseRoute({
    getParentRoute: () => root,
    path: "disktree",
    component: DiskTree,
  }),
];
const router = createDataRouter({
  routeTree: root.addChildren(routes),
  context: {},
});

mount(() => <RouterProvider router={router} />);
