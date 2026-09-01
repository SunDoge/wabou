import {
  defineComponentFixtures,
  type LayoutFixtureRegistry,
} from "@wabou/test/layout/fixtures";
import { ComponentsProvider, View } from "@wabou/ui";
import { type Component, createComponent } from "solid-js";
import {
  ActivityStatusLayoutFixture,
  AdaptiveSplitPaneLayoutFixture,
  AlertLayoutFixture,
  CardSurfaceLayoutFixture,
  CompactSurfaceLayoutFixture,
  ContentStateLayoutFixture,
  ControlBaselineLayoutFixture,
  DarkSurfaceLayoutFixture,
  DialogLayoutFixture,
  EditorLayoutFixture,
  EmptyLayoutFixture,
  IconFrameLayoutFixture,
  ImageListLayoutFixture,
  ImageViewportLayoutFixture,
  InputGroupLayoutFixture,
  LabeledSeparatorLayoutFixture,
  ListboxLayoutFixture,
  MarkdownConversationLayoutFixture,
  MarkdownInlineLayoutFixture,
  MessageLayoutFixture,
  OnboardingLayoutFixture,
  PiAgentHeaderLayoutFixture,
  PiComposerContextLayoutFixture,
  PiComposerDeliveryLayoutFixture,
  PiComposerImagesLayoutFixture,
  PiConversationLayoutFixture,
  PiExtensionUiLayoutFixture,
  PiModelControlsLayoutFixture,
  PiSessionBehaviorLayoutFixture,
  PiTranscriptSearchLayoutFixture,
  PromptSuggestionLayoutFixture,
  QRCodeLayoutFixture,
  ReasoningLayoutFixture,
  ResourceBoundaryLayoutFixture,
  ScrollAreaLayoutFixture,
  SelectionControlsLayoutFixture,
  SelectLayoutFixture,
  SidebarLayoutFixture,
  SliderLayoutFixture,
  SpinnerLayoutFixture,
  TabsLayoutFixture,
  VerticalTabsLayoutFixture,
  ToastLayoutFixture,
  ToolLayoutFixture,
} from "./layout-fixture-components";
import * as animation from "./pages/animation";
import * as basics from "./pages/basics";
import * as chart from "./pages/chart";
import * as dataTable from "./pages/data-table";
import * as foundations from "./pages/foundations";
import * as i18n from "./pages/i18n";
import * as imageViewport from "./pages/image-viewport";
import * as markdown from "./pages/markdown";
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
    wrap: (render) => (
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-h-0 p-6 overflow-x-hidden overflow-y-auto">
          {render()}
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
      "component/ControlBaseline": {
        width: 520,
        height: 280,
        render: ControlBaselineLayoutFixture,
      },
      "component/SelectionControls": {
        width: 420,
        height: 360,
        render: SelectionControlsLayoutFixture,
      },
      "component/Tabs": {
        width: 560,
        height: 280,
        render: TabsLayoutFixture,
      },
      "component/VerticalTabs": {
        width: 560,
        height: 280,
        render: VerticalTabsLayoutFixture,
      },
      "component/Alert": {
        width: 420,
        height: 240,
        render: AlertLayoutFixture,
      },
      "component/Toast": {
        width: 480,
        height: 240,
        render: ToastLayoutFixture,
      },
      "component/CardSurface": {
        width: 480,
        height: 320,
        render: CardSurfaceLayoutFixture,
      },
      "component/DarkSurface": {
        width: 420,
        height: 340,
        render: DarkSurfaceLayoutFixture,
      },
      "component/CompactSurface": {
        width: 300,
        height: 360,
        render: CompactSurfaceLayoutFixture,
      },
      "component/Listbox": {
        width: 420,
        height: 240,
        render: ListboxLayoutFixture,
      },
      "component/Spinner": {
        width: 240,
        height: 120,
        render: SpinnerLayoutFixture,
      },
      "component/ActivityStatus": {
        width: 280,
        height: 72,
        render: ActivityStatusLayoutFixture,
      },
      "component/ContentState": {
        width: 320,
        height: 240,
        render: ContentStateLayoutFixture,
      },
      "component/ResourceBoundary": {
        width: 320,
        height: 240,
        render: ResourceBoundaryLayoutFixture,
      },
      "component/Onboarding": {
        width: 360,
        height: 420,
        render: OnboardingLayoutFixture,
      },
      "component/Slider": {
        width: 480,
        height: 120,
        render: SliderLayoutFixture,
      },
      "component/Dialog": {
        width: 640,
        height: 480,
        render: DialogLayoutFixture,
      },
      "component/Empty": {
        width: 320,
        height: 420,
        render: EmptyLayoutFixture,
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
      "component/QRCode": {
        width: 320,
        height: 320,
        render: QRCodeLayoutFixture,
      },
      "component/IconFrame": {
        width: 160,
        height: 160,
        render: IconFrameLayoutFixture,
      },
      "component/InputGroup": {
        width: 440,
        height: 160,
        render: InputGroupLayoutFixture,
      },
      "component/MarkdownInline": {
        width: 360,
        height: 120,
        render: MarkdownInlineLayoutFixture,
      },
      "component/LabeledSeparator": {
        width: 560,
        height: 96,
        render: LabeledSeparatorLayoutFixture,
      },
      "component/MarkdownConversation": {
        width: 720,
        height: 520,
        render: MarkdownConversationLayoutFixture,
      },
      "component/PiConversation": {
        width: 720,
        height: 520,
        render: PiConversationLayoutFixture,
        waitMs: 220,
      },
      "component/Tool": {
        width: 360,
        height: 420,
        render: ToolLayoutFixture,
      },
      "component/Reasoning": {
        width: 600,
        height: 240,
        render: ReasoningLayoutFixture,
      },
      "component/PromptSuggestion": {
        width: 420,
        height: 260,
        render: PromptSuggestionLayoutFixture,
      },
      "component/PiAgentHeader": {
        width: 1080,
        height: 56,
        render: PiAgentHeaderLayoutFixture,
      },
      "component/PiComposerImages": {
        width: 420,
        height: 120,
        render: PiComposerImagesLayoutFixture,
      },
      "component/PiComposerContext": {
        width: 420,
        height: 120,
        render: PiComposerContextLayoutFixture,
      },
      "component/PiComposerDelivery": {
        width: 240,
        height: 80,
        render: PiComposerDeliveryLayoutFixture,
      },
      "component/PiExtensionUi": {
        width: 420,
        height: 180,
        render: PiExtensionUiLayoutFixture,
      },
      "component/PiModelControls": {
        width: 480,
        height: 100,
        render: PiModelControlsLayoutFixture,
      },
      "component/PiSessionBehavior": {
        width: 520,
        height: 360,
        render: PiSessionBehaviorLayoutFixture,
      },
      "component/PiTranscriptSearch": {
        width: 480,
        height: 180,
        render: PiTranscriptSearchLayoutFixture,
      },
      "component/Editor": {
        width: 520,
        height: 320,
        render: EditorLayoutFixture,
      },
      "component/Message": {
        width: 360,
        height: 180,
        render: MessageLayoutFixture,
      },
      "component/ImageList": {
        width: 360,
        height: 360,
        render: ImageListLayoutFixture,
      },
    },
    {
      wrap: (render) => (
        <ComponentsProvider theme="light">{render()}</ComponentsProvider>
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
  ...pageFixtures("markdown", markdown),
  ...pageFixtures("overlay", overlay),
  ...pageFixtures("shadcn", shadcn),
  ...pageFixtures("system", system),
  ...pageFixtures("toolbar", toolbar),
  ...pageFixtures("tree-view", treeView),
};
