import {
  Badge,
  BaseRootRoute,
  BaseRoute,
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  createDataRouter,
  createMemoryHistory,
  createScrollReset,
  currentWindow,
  createWindowMatch,
  Fps,
  type Handle,
  mount,
  PrimitiveButton,
  RouterProvider,
  Separator,
  Text,
  useLocation,
  useNavigate,
  useParams,
  useWindow,
  View,
} from "@wabou/ui";
import { createSignal, Match, Show, Switch as ShowCase } from "solid-js";
import "virtual:wabou-stylesheet";

import { OverlayPage } from "./pages/overlay";
import { SystemPage } from "./pages/system";
import { GallerySidebar } from "./sidebar";

function classes(...values: Array<string | false | undefined>): string {
  return values.filter(Boolean).join(" ");
}

type ComponentId =
  | "button"
  | "badge"
  | "card"
  | "input"
  | "input-otp"
  | "number-field"
  | "checkbox"
  | "radio-group"
  | "switch"
  | "toggle"
  | "tabs"
  | "progress"
  | "slider"
  | "rating"
  | "fps"
  | "animation"
  | "platform"
  | "system"
  | "colors"
  | "shadows"
  | "utilities"
  | "scroll-area"
  | "resizable"
  | "aspect-ratio"
  | "overlay"
  | "alert"
  | "skeleton"
  | "spinner"
  | "kbd"
  | "separator"
  | "dialog"
  | "alert-dialog"
  | "sheet"
  | "drawer"
  | "toast"
  | "tooltip"
  | "popover"
  | "hover-card"
  | "breadcrumb"
  | "pagination"
  | "navigation-menu"
  | "carousel"
  | "dropdown-menu"
  | "context-menu"
  | "command"
  | "combobox"
  | "accordion"
  | "avatar"
  | "field"
  | "empty"
  | "item"
  | "attachment"
  | "message"
  | "message-scroller"
  | "button-group"
  | "toolbar"
  | "menubar"
  | "select"
  | "native-select"
  | "date-picker"
  | "data-table"
  | "tree-view"
  | "chart"
  | "direction"
  | "typography"
  | "i18n";

const groups: Array<{
  label: string;
  items: Array<{ id: ComponentId; name: string }>;
}> = [
  {
    label: "Actions",
    items: [
      { id: "button", name: "Button" },
      { id: "button-group", name: "Button group" },
      { id: "toolbar", name: "Toolbar" },
      { id: "menubar", name: "Menubar" },
      { id: "toggle", name: "Toggle" },
      { id: "switch", name: "Switch" },
      { id: "dropdown-menu", name: "Dropdown menu" },
      { id: "context-menu", name: "Context menu" },
      { id: "command", name: "Command" },
      { id: "sheet", name: "Sheet" },
      { id: "drawer", name: "Drawer" },
      { id: "toast", name: "Toast" },
      { id: "popover", name: "Popover" },
      { id: "hover-card", name: "Hover card" },
    ],
  },
  {
    label: "Forms",
    items: [
      { id: "input", name: "Input" },
      { id: "input-otp", name: "Input OTP" },
      { id: "number-field", name: "Number field" },
      { id: "select", name: "Select" },
      { id: "native-select", name: "Native select" },
      { id: "combobox", name: "Combobox" },
      { id: "date-picker", name: "Date picker" },
      { id: "checkbox", name: "Checkbox" },
      { id: "radio-group", name: "Radio group" },
      { id: "field", name: "Field & input group" },
    ],
  },
  {
    label: "Navigation",
    items: [
      { id: "breadcrumb", name: "Breadcrumb" },
      { id: "pagination", name: "Pagination" },
      { id: "navigation-menu", name: "Navigation menu" },
      { id: "carousel", name: "Carousel" },
    ],
  },
  {
    label: "Data display",
    items: [
      { id: "badge", name: "Badge" },
      { id: "card", name: "Card" },
      { id: "item", name: "Item" },
      { id: "attachment", name: "Attachment" },
      { id: "message", name: "Message" },
      { id: "message-scroller", name: "Message scroller" },
      { id: "chart", name: "Chart experiment" },
      { id: "typography", name: "Typography" },
      { id: "fps", name: "FPS" },
      { id: "progress", name: "Progress" },
      { id: "slider", name: "Slider" },
      { id: "rating", name: "Rating" },
      { id: "tabs", name: "Tabs" },
      { id: "kbd", name: "Kbd" },
      { id: "avatar", name: "Avatar" },
      { id: "data-table", name: "Data table" },
      { id: "tree-view", name: "Tree view" },
    ],
  },
  {
    label: "Foundations",
    items: [
      { id: "colors", name: "Colors" },
      { id: "shadows", name: "Shadows" },
      { id: "overlay", name: "Overlay" },
      { id: "i18n", name: "Internationalization" },
      { id: "direction", name: "Direction" },
    ],
  },
  {
    label: "Feedback",
    items: [
      { id: "alert", name: "Alert" },
      { id: "dialog", name: "Dialog" },
      { id: "alert-dialog", name: "Alert dialog" },
      { id: "tooltip", name: "Tooltip" },
      { id: "empty", name: "Empty" },
      { id: "accordion", name: "Accordion" },
      { id: "skeleton", name: "Skeleton" },
      { id: "spinner", name: "Spinner" },
      { id: "animation", name: "Animation" },
      { id: "separator", name: "Separator" },
    ],
  },
  {
    label: "Layout",
    items: [
      { id: "aspect-ratio", name: "Aspect ratio" },
      { id: "utilities", name: "Utilities" },
      { id: "scroll-area", name: "Scroll area" },
      { id: "resizable", name: "Resizable" },
    ],
  },
  {
    label: "Platform",
    items: [
      { id: "platform", name: "Native window" },
      { id: "system", name: "System APIs" },
    ],
  },
];

const descriptions: Record<ComponentId, string> = {
  button: "Displays a button or a component that looks like a button.",
  badge: "A compact label for statuses, categories and metadata.",
  card: "A flexible container for grouped content and actions.",
  input: "A native text input with consistent layout and visual treatment.",
  "input-otp":
    "Projects native keyboard and paste input into explicit one-time-code slots.",
  "number-field":
    "Edits locale-aware numeric values with explicit stepping and native spinbutton semantics.",
  checkbox:
    "A binary or indeterminate selection control for forms and settings.",
  "radio-group": "A mutually exclusive selection group with native semantics.",
  switch: "A control that lets users toggle a setting on or off.",
  toggle: "A two-state button for formatting, filters and compact toolbars.",
  tabs: "Organizes related panels with keyboard-operable native tab semantics.",
  progress: "Shows completion for a task or a long-running operation.",
  slider:
    "Selects a numeric value with pointer dragging and keyboard controls.",
  rating:
    "Selects a discrete rating with hover preview, roving focus and native radio semantics.",
  fps: "Measures native host frames and highlights performance regressions.",
  animation: "Pure JavaScript value animations rendered by the native host.",
  platform: "Native windows and Rust-powered custom widgets.",
  system: "Native file dialogs, message dialogs and desktop notifications.",
  colors: "Every color token exported by the native Wabou utility theme.",
  shadows:
    "Vello-native blurred rounded rectangles with explicit Gaussian parameters.",
  utilities: "Tailwind-style static classes parsed by the native Rust preset.",
  "scroll-area": "A native scrolling viewport with intrinsic flex content.",
  resizable:
    "Composes explicitly identified panels with pointer and keyboard resizing.",
  "aspect-ratio":
    "Maintains a native width-to-height layout constraint for media and previews.",
  overlay: "Explicit floating and modal planes shared by JavaScript portals.",
  alert: "Calls attention to information that needs user awareness.",
  skeleton: "A lightweight animated placeholder for content that is loading.",
  spinner: "An animated status indicator for indeterminate work.",
  kbd: "Displays keyboard input and shortcut chords.",
  separator: "Visually separates content in a list or layout.",
  dialog: "A modal surface with native focus isolation and dismissal behavior.",
  "alert-dialog":
    "Interrupts a workflow until the user explicitly confirms or cancels.",
  sheet: "A modal panel attached to a viewport edge.",
  drawer:
    "A focus-isolated edge panel that can be dismissed with captured pointer dragging.",
  toast: "Shows non-blocking feedback with queueing, actions and timeouts.",
  tooltip:
    "A delayed floating label for pointer hover and keyboard focus targets.",
  popover:
    "Displays composed interactive content in a collision-aware floating surface.",
  "hover-card":
    "Previews related content with delayed pointer and keyboard interaction.",
  breadcrumb:
    "Shows the current location while leaving navigation behavior to the application.",
  pagination: "Composes explicit controls for navigating a paged collection.",
  "navigation-menu":
    "Switches rich navigation content inside one shared native floating viewport.",
  carousel:
    "Presents snapping slides with captured pointer dragging, buttons and keyboard navigation.",
  "dropdown-menu":
    "Presents a compact list of actions with native focus and typeahead.",
  "context-menu": "Anchors actions to a native secondary-click coordinate.",
  command: "Filters and activates commands with pointer or keyboard input.",
  combobox: "Searches and selects one value from a larger option collection.",
  accordion:
    "Vertically stacked disclosure sections with controlled or uncontrolled state.",
  avatar: "A compact visual identity with initials, images and grouped counts.",
  field: "Composable labels, descriptions, errors and input adornments.",
  select: "A keyboard-operable native listbox for choosing one option.",
  "native-select":
    "A compact immediate select contract for ordinary desktop forms.",
  "date-picker":
    "Selects an internationalized calendar date from a native floating panel.",
  empty: "A centered placeholder for collections that do not contain data yet.",
  item: "A composable row for lists, settings and compact application summaries.",
  attachment:
    "Displays native file and transfer state using reusable media, content and action slots.",
  message:
    "Composes aligned avatars, metadata, bubbles, reactions and timeline markers.",
  "message-scroller":
    "Follows appended messages until the user explicitly scrolls away from the end.",
  "button-group":
    "Groups related actions into horizontal or vertical toolbars.",
  toolbar:
    "Composes commands and toggles with one tab stop and directional keyboard navigation.",
  menubar:
    "Provides persistent application menus with sibling switching by keyboard or pointer.",
  "data-table":
    "A framework-agnostic TanStack Table core rendered through native Wabou primitives.",
  "tree-view":
    "Navigates explicit hierarchical data with roving focus and native keyboard semantics.",
  chart:
    "D3 geometry and scales rendered through Wabou's typed native path pipeline.",
  direction:
    "Projects explicit logical direction into native row and text layout.",
  typography:
    "Composable heading, paragraph, list, quote and inline-code treatments.",
  i18n: "Tree-shakeable typed messages compiled by Paraglide and driven by Solid locale state.",
};

const history = createMemoryHistory();
const themeOrder = ["dark", "light", "violet"] as const;
type GalleryTheme = (typeof themeOrder)[number];

import { AnimationPage } from "./pages/animation";
import {
  AlertPage,
  BadgePage,
  ButtonPage,
  CardPage,
  CheckboxPage,
  ChildWindowPage,
  FpsPage,
  InputPage,
  KbdPage,
  NumberFieldPage,
  PlatformPage,
  ProgressPage,
  RadioGroupPage,
  RatingPage,
  ScrollAreaPage,
  SeparatorPage,
  SkeletonPage,
  SliderPage,
  SpinnerPage,
  SwitchPage,
  TabsPage,
  TogglePage,
  UtilitiesPage,
} from "./pages/basics";
import { ChartPage } from "./pages/chart";
import { DataTablePage } from "./pages/data-table";
import { ColorsPage, ShadowsPage } from "./pages/foundations";
import { I18nPage } from "./pages/i18n";
import { MenubarPage } from "./pages/menubar";
import { OverviewPage } from "./pages/overview";
import {
  DirectionPage,
  NativeSelectPage,
  TypographyPage,
} from "./pages/shadcn";
import { ToolbarPage } from "./pages/toolbar";
import { TreeViewPage } from "./pages/tree-view";
import {
  AccordionPage,
  AlertDialogPage,
  AspectRatioPage,
  AttachmentPage,
  AvatarPage,
  BreadcrumbPageDemo,
  ButtonGroupPage,
  CarouselPage,
  ComboboxPage,
  CommandPage,
  ContextMenuPage,
  DatePickerPage,
  DialogPage,
  DrawerPage,
  DropdownMenuPage,
  EmptyPage,
  FieldPage,
  HoverCardPage,
  InputOTPPage,
  ItemPage,
  MessagePage,
  MessageScrollerPage,
  NavigationMenuPage,
  PaginationPage,
  PopoverPage,
  ResizablePage,
  SelectPage,
  SheetPage,
  ToastPage,
  TooltipPage,
} from "./pages/widgets";

function App() {
  const window = useWindow();
  const compact = createWindowMatch({ maxWidth: 1099 }, window);
  const windowId = currentWindow().id;
  if (windowId.lo !== 1 || windowId.hi !== 1) return <ChildWindowPage />;
  const [theme, setTheme] = createSignal<GalleryTheme>("dark");
  const dark = () => theme() !== "light";
  const themeLabel = () =>
    `${theme().slice(0, 1).toUpperCase()}${theme().slice(1)}`;
  const cycleTheme = () => {
    const index = themeOrder.indexOf(theme());
    setTheme(themeOrder[(index + 1) % themeOrder.length]);
  };
  const params = useParams<{ component?: string }>();
  const location = useLocation();
  let contentViewport: Handle | undefined;
  createScrollReset({
    target: () => contentViewport,
    key: () => location().pathname,
  });
  const navigate = useNavigate();
  const selected = (): ComponentId | null =>
    groups.some((group) =>
      group.items.some((item) => item.id === params().component),
    )
      ? (params().component as ComponentId)
      : null;
  const selectedName = () =>
    selected() === null
      ? "Overview"
      : (groups
          .flatMap((group) => group.items)
          .find((item) => item.id === selected())?.name ?? "Component");
  const selectedDescription = () => {
    const component = selected();
    return component === null ? "" : descriptions[component];
  };

  return (
    <ColorThemeProvider
      theme={theme()}
      transition={{ duration: 0.32, easing: "ease-out", colorSpace: "oklab" }}
    >
      <ComponentsProvider theme={dark() ? "dark" : "light"}>
        <View class="w-full h-full flex overflow-hidden bg-canvas text-primary font-sans">
          <GallerySidebar
            groups={groups}
            descriptions={descriptions}
            selected={selected()}
            compact={compact()}
            onSelect={(id) =>
              void navigate({ to: id === null ? "/" : `/components/${id}` })
            }
          />

          <View class="flex-1 min-w-0 h-full flex flex-col">
            <View class="h-14 flex-none px-6 flex items-center justify-between border-b border-subtle bg-surface">
              <View class="flex items-center gap-3">
                <View class="flex items-center gap-1">
                  <PrimitiveButton
                    unstyled
                    class={(state) =>
                      classes(
                        "w-8 h-8 justify-center rounded-md text-secondary",
                        state.hovered && "bg-control-hover text-primary",
                      )
                    }
                    onClick={() => history.back()}
                  >
                    ‹
                  </PrimitiveButton>
                  <PrimitiveButton
                    unstyled
                    class={(state) =>
                      classes(
                        "w-8 h-8 justify-center rounded-md text-secondary",
                        state.hovered && "bg-control-hover text-primary",
                      )
                    }
                    onClick={() => history.forward()}
                  >
                    ›
                  </PrimitiveButton>
                </View>
                <Text class="text-sm text-muted">
                  {selected() === null
                    ? "Wabou / Overview"
                    : `Components / ${selectedName()}`}
                </Text>
              </View>
              <View class="flex items-center gap-2">
                <Fps />
                <Button size="sm" variant="ghost" onClick={cycleTheme}>
                  {`Theme: ${themeLabel()}`}
                </Button>
                <Show when={!compact()}>
                  <Badge variant="success">Native</Badge>
                  <Badge variant="outline">UnoCSS</Badge>
                </Show>
              </View>
            </View>
            <View
              ref={(node) => (contentViewport = node)}
              class="flex-1 min-w-0 min-h-0 overflow-x-hidden overflow-y-auto"
            >
              <View
                class={classes(
                  "w-full max-w-5xl mx-auto flex flex-col gap-6",
                  compact() ? "px-5 py-6" : "px-10 py-8",
                )}
              >
                <Show when={selected() !== null}>
                  <View class="flex flex-col gap-2">
                    <Text
                      role="heading"
                      class="text-2xl font-bold text-primary"
                    >
                      {selectedName()}
                    </Text>
                    <Text class="text-sm text-secondary">
                      {selectedDescription()}
                    </Text>
                  </View>
                  <Separator />
                </Show>
                <ShowCase>
                  <Match when={selected() === null}>
                    <OverviewPage
                      theme={themeLabel()}
                      onCycleTheme={cycleTheme}
                      onExplore={() =>
                        void navigate({ to: "/components/button" })
                      }
                    />
                  </Match>
                  <Match when={selected() === "button"}>
                    <ButtonPage />
                  </Match>
                  <Match when={selected() === "badge"}>
                    <BadgePage />
                  </Match>
                  <Match when={selected() === "card"}>
                    <CardPage />
                  </Match>
                  <Match when={selected() === "input"}>
                    <InputPage />
                  </Match>
                  <Match when={selected() === "input-otp"}>
                    <InputOTPPage />
                  </Match>
                  <Match when={selected() === "number-field"}>
                    <NumberFieldPage />
                  </Match>
                  <Match when={selected() === "checkbox"}>
                    <CheckboxPage />
                  </Match>
                  <Match when={selected() === "radio-group"}>
                    <RadioGroupPage />
                  </Match>
                  <Match when={selected() === "switch"}>
                    <SwitchPage />
                  </Match>
                  <Match when={selected() === "toggle"}>
                    <TogglePage />
                  </Match>
                  <Match when={selected() === "tabs"}>
                    <TabsPage />
                  </Match>
                  <Match when={selected() === "progress"}>
                    <ProgressPage />
                  </Match>
                  <Match when={selected() === "slider"}>
                    <SliderPage />
                  </Match>
                  <Match when={selected() === "rating"}>
                    <RatingPage />
                  </Match>
                  <Match when={selected() === "fps"}>
                    <FpsPage />
                  </Match>
                  <Match when={selected() === "scroll-area"}>
                    <ScrollAreaPage />
                  </Match>
                  <Match when={selected() === "overlay"}>
                    <OverlayPage />
                  </Match>
                  <Match when={selected() === "utilities"}>
                    <UtilitiesPage />
                  </Match>
                  <Match when={selected() === "colors"}>
                    <ColorsPage />
                  </Match>
                  <Match when={selected() === "shadows"}>
                    <ShadowsPage />
                  </Match>
                  <Match when={selected() === "alert"}>
                    <AlertPage />
                  </Match>
                  <Match when={selected() === "skeleton"}>
                    <SkeletonPage />
                  </Match>
                  <Match when={selected() === "spinner"}>
                    <SpinnerPage />
                  </Match>
                  <Match when={selected() === "kbd"}>
                    <KbdPage />
                  </Match>
                  <Match when={selected() === "animation"}>
                    <AnimationPage />
                  </Match>
                  <Match when={selected() === "platform"}>
                    <PlatformPage />
                  </Match>
                  <Match when={selected() === "system"}>
                    <SystemPage />
                  </Match>
                  <Match when={selected() === "separator"}>
                    <SeparatorPage />
                  </Match>
                  <Match when={selected() === "dialog"}>
                    <DialogPage />
                  </Match>
                  <Match when={selected() === "alert-dialog"}>
                    <AlertDialogPage />
                  </Match>
                  <Match when={selected() === "sheet"}>
                    <SheetPage />
                  </Match>
                  <Match when={selected() === "drawer"}>
                    <DrawerPage />
                  </Match>
                  <Match when={selected() === "toast"}>
                    <ToastPage />
                  </Match>
                  <Match when={selected() === "tooltip"}>
                    <TooltipPage />
                  </Match>
                  <Match when={selected() === "popover"}>
                    <PopoverPage />
                  </Match>
                  <Match when={selected() === "hover-card"}>
                    <HoverCardPage />
                  </Match>
                  <Match when={selected() === "resizable"}>
                    <ResizablePage />
                  </Match>
                  <Match when={selected() === "aspect-ratio"}>
                    <AspectRatioPage />
                  </Match>
                  <Match when={selected() === "breadcrumb"}>
                    <BreadcrumbPageDemo />
                  </Match>
                  <Match when={selected() === "pagination"}>
                    <PaginationPage />
                  </Match>
                  <Match when={selected() === "navigation-menu"}>
                    <NavigationMenuPage />
                  </Match>
                  <Match when={selected() === "carousel"}>
                    <CarouselPage />
                  </Match>
                  <Match when={selected() === "dropdown-menu"}>
                    <DropdownMenuPage />
                  </Match>
                  <Match when={selected() === "context-menu"}>
                    <ContextMenuPage />
                  </Match>
                  <Match when={selected() === "command"}>
                    <CommandPage />
                  </Match>
                  <Match when={selected() === "accordion"}>
                    <AccordionPage />
                  </Match>
                  <Match when={selected() === "avatar"}>
                    <AvatarPage />
                  </Match>
                  <Match when={selected() === "field"}>
                    <FieldPage />
                  </Match>
                  <Match when={selected() === "empty"}>
                    <EmptyPage />
                  </Match>
                  <Match when={selected() === "item"}>
                    <ItemPage />
                  </Match>
                  <Match when={selected() === "attachment"}>
                    <AttachmentPage />
                  </Match>
                  <Match when={selected() === "message"}>
                    <MessagePage />
                  </Match>
                  <Match when={selected() === "message-scroller"}>
                    <MessageScrollerPage />
                  </Match>
                  <Match when={selected() === "button-group"}>
                    <ButtonGroupPage />
                  </Match>
                  <Match when={selected() === "toolbar"}>
                    <ToolbarPage />
                  </Match>
                  <Match when={selected() === "menubar"}>
                    <MenubarPage />
                  </Match>
                  <Match when={selected() === "select"}>
                    <SelectPage />
                  </Match>
                  <Match when={selected() === "native-select"}>
                    <NativeSelectPage />
                  </Match>
                  <Match when={selected() === "combobox"}>
                    <ComboboxPage />
                  </Match>
                  <Match when={selected() === "date-picker"}>
                    <DatePickerPage />
                  </Match>
                  <Match when={selected() === "data-table"}>
                    <DataTablePage />
                  </Match>
                  <Match when={selected() === "tree-view"}>
                    <TreeViewPage />
                  </Match>
                  <Match when={selected() === "chart"}>
                    <ChartPage />
                  </Match>
                  <Match when={selected() === "direction"}>
                    <DirectionPage />
                  </Match>
                  <Match when={selected() === "typography"}>
                    <TypographyPage />
                  </Match>
                  <Match when={selected() === "i18n"}>
                    <I18nPage />
                  </Match>
                </ShowCase>
              </View>
            </View>
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

const root = new BaseRootRoute({ component: App });
const index = new BaseRoute({ getParentRoute: () => root, path: "/" });
const component = new BaseRoute({
  getParentRoute: () => root,
  path: "components/$component",
});
const router = createDataRouter({
  routeTree: root.addChildren([index, component]),
  history,
  context: {},
});

mount(() => <RouterProvider router={router} />);
