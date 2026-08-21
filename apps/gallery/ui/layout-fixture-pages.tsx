import type { LayoutFixtureRegistry } from "@wabou/test/layout/fixtures";
import { ComponentsProvider, View } from "@wabou/ui";
import { type Component, createComponent } from "solid-js";
import * as chart from "./pages/chart";
import * as dataTable from "./pages/data-table";
import * as i18n from "./pages/i18n";
import * as menubar from "./pages/menubar";
import * as overlay from "./pages/overlay";
import * as system from "./pages/system";
import * as toolbar from "./pages/toolbar";
import * as treeView from "./pages/tree-view";
import * as widgets from "./pages/widgets";

type PageModule = Readonly<Record<string, unknown>>;

function pageFixtures(
  prefix: string,
  pages: PageModule,
): Record<string, () => ReturnType<Component>> {
  return Object.fromEntries(
    Object.entries(pages)
      .filter(
        (entry): entry is [string, Component] =>
          entry[0].endsWith("Page") && typeof entry[1] === "function",
      )
      .map(([name, Page]) => [
        `${prefix}/${name.replace(/Page$/, "")}`,
        () => (
          <ComponentsProvider theme="light">
            <View class="w-full min-h-full p-6">
              {createComponent(Page, {})}
            </View>
          </ComponentsProvider>
        ),
      ]),
  );
}

export const galleryLayoutFixtures: LayoutFixtureRegistry = {
  ...pageFixtures("widgets", widgets),
  ...pageFixtures("chart", chart),
  ...pageFixtures("data-table", dataTable),
  ...pageFixtures("i18n", i18n),
  ...pageFixtures("menubar", menubar),
  ...pageFixtures("overlay", overlay),
  ...pageFixtures("system", system),
  ...pageFixtures("toolbar", toolbar),
  ...pageFixtures("tree-view", treeView),
};
