import {
  defineComponentFixtures,
  type LayoutFixtureRegistry,
} from "@wabou/test/layout/fixtures";
import { ComponentsProvider, View } from "@wabou/ui";
import { type Component, createComponent } from "solid-js";
import {
  AdaptiveSplitPaneLayoutFixture,
  DialogLayoutFixture,
  ImageListLayoutFixture,
  ImageViewportLayoutFixture,
  ScrollAreaLayoutFixture,
  SelectLayoutFixture,
  SidebarLayoutFixture,
} from "./layout-fixture-components";
import * as animation from "./pages/animation";
import * as basics from "./pages/basics";
import * as chart from "./pages/chart";
import * as dataTable from "./pages/data-table";
import * as foundations from "./pages/foundations";
import * as i18n from "./pages/i18n";
import * as imageViewport from "./pages/image-viewport";
import * as menubar from "./pages/menubar";
import * as overlay from "./pages/overlay";
import * as shadcn from "./pages/shadcn";
import * as system from "./pages/system";
import * as toolbar from "./pages/toolbar";
import * as treeView from "./pages/tree-view";
import * as widgets from "./pages/widgets";

type PageModule = Readonly<Record<string, unknown>>;

function pageFixtures(
  prefix: string,
  pages: PageModule,
): LayoutFixtureRegistry {
  const fixtures = Object.fromEntries(
    Object.entries(pages)
      .filter(
        (entry): entry is [string, Component] =>
          entry[0].endsWith("Page") && typeof entry[1] === "function",
      )
      .map(([name, Page]) => [
        `${prefix}/${name.replace(/Page$/, "")}`,
        () => createComponent(Page, {}),
      ]),
  );
  return defineComponentFixtures(fixtures, {
    wrap: (content) => (
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-h-0 p-6 overflow-x-hidden overflow-y-auto">
          {content}
        </View>
      </ComponentsProvider>
    ),
  });
}

export const galleryLayoutFixtures: LayoutFixtureRegistry = {
  ...defineComponentFixtures(
    {
      "component/Sidebar": {
        width: 360,
        height: 420,
        render: SidebarLayoutFixture,
      },
      "component/ScrollArea": {
        width: 360,
        height: 240,
        render: ScrollAreaLayoutFixture,
      },
      "component/Select": {
        width: 480,
        height: 420,
        waitMs: 20,
        render: SelectLayoutFixture,
      },
      "component/Dialog": {
        width: 640,
        height: 480,
        render: DialogLayoutFixture,
      },
      "component/AdaptiveSplitPane": {
        width: 720,
        height: 360,
        render: AdaptiveSplitPaneLayoutFixture,
      },
      "component/ImageViewport": {
        width: 720,
        height: 520,
        render: ImageViewportLayoutFixture,
      },
      "component/ImageList": {
        width: 360,
        height: 360,
        render: ImageListLayoutFixture,
      },
    },
    {
      wrap: (content) => (
        <ComponentsProvider theme="light">{content}</ComponentsProvider>
      ),
    },
  ),
  ...pageFixtures("animation", animation),
  ...pageFixtures("basics", basics),
  ...pageFixtures("widgets", widgets),
  ...pageFixtures("foundations", foundations),
  ...pageFixtures("chart", chart),
  ...pageFixtures("data-table", dataTable),
  ...pageFixtures("i18n", i18n),
  ...pageFixtures("image-viewport", imageViewport),
  ...pageFixtures("menubar", menubar),
  ...pageFixtures("overlay", overlay),
  ...pageFixtures("shadcn", shadcn),
  ...pageFixtures("system", system),
  ...pageFixtures("toolbar", toolbar),
  ...pageFixtures("tree-view", treeView),
};
