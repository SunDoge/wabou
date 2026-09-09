import {
  ActivityStatus,
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  createTransition,
  createWindowMatch,
  Icon,
  PageViewport,
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuIcon,
  SidebarMenuLabel,
  Text,
  TitleBar,
  TitleBarDragRegion,
  useLocation,
  useNavigate,
  useRouteActive,
  useWindow,
  View,
  WindowFrame,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import check from "lucide-static/icons/check.svg?raw";
import code from "lucide-static/icons/code-2.svg?raw";
import database from "lucide-static/icons/database.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import list from "lucide-static/icons/list.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import monitor from "lucide-static/icons/monitor.svg?raw";
import panelLeftClose from "lucide-static/icons/panel-left-close.svg?raw";
import panelLeftOpen from "lucide-static/icons/panel-left-open.svg?raw";
import refresh from "lucide-static/icons/refresh-cw.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createSignal, For as ForValue, type JSX, Show } from "solid-js";
import meowMark from "./meow-mark.svg?raw";

const navItems = [
  { path: "/", label: "Overview", icon: monitor },
  { path: "/tasks", label: "Tasks", icon: list },
  { path: "/database", label: "Database", icon: database },
  { path: "/agent", label: "Agent", icon: bot },
  { path: "/settings", label: "Settings", icon: settings },
  { path: "/about", label: "About", icon: info },
] as const;

const capabilities = [
  {
    title: "SQLite",
    detail: "Local-first storage",
    status: "Ready",
    icon: database,
  },
  {
    title: "Tray",
    detail: "Background lifecycle",
    status: "Enabled",
    icon: monitor,
  },
  {
    title: "Updater",
    detail: "Signed delivery",
    status: "Up to date",
    icon: refresh,
  },
  {
    title: "Agent Preview",
    detail: "Approval-first tools",
    status: "Preview",
    icon: bot,
  },
] as const;

function WindowControls(): JSX.Element {
  const window = useWindow();
  return (
    <View class="h-full flex-none flex flex-row items-center gap-1 pr-2">
      <Button
        aria-label="Minimize window"
        variant="ghost"
        size="icon"
        onClick={() => window.minimize()}
      >
        <Icon source={minus} size={15} />
      </Button>
      <Button
        aria-label={window.maximized() ? "Restore window" : "Maximize window"}
        variant="ghost"
        size="icon"
        onClick={() => window.setMaximized(!window.maximized())}
      >
        <Icon source={square} size={13} />
      </Button>
      <Button
        aria-label="Close window"
        variant="ghost"
        size="icon"
        onClick={() => window.close()}
      >
        <Icon source={x} size={16} />
      </Button>
    </View>
  );
}

function NavButton(props: {
  path: string;
  label: string;
  icon: string;
  expanded: boolean;
}): JSX.Element {
  const navigate = useNavigate();
  const active = useRouteActive(props.path);
  return (
    <SidebarMenuButton
      value={props.path}
      selected={active()}
      aria-current={active() ? "page" : undefined}
      aria-label={props.label}
      class="h-10 rounded-lg border-transparent text-sidebar-muted"
      classList={{
        "px-3 justify-start": props.expanded,
        "px-0 justify-center": !props.expanded,
        "bg-sidebar-raised text-sidebar-primary": active(),
      }}
      onClick={() => navigate({ to: props.path })}
    >
      <SidebarMenuIcon>
        <Icon source={props.icon} size={17} />
      </SidebarMenuIcon>
      <Show when={props.expanded}>
        <SidebarMenuLabel class="text-sidebar-primary">
          {props.label}
        </SidebarMenuLabel>
      </Show>
    </SidebarMenuButton>
  );
}

function AppSidebar(props: {
  expanded: boolean;
  width: number;
  onExpandedChange(value: boolean): void;
}): JSX.Element {
  return (
    <Sidebar
      aria-label="Primary navigation"
      class="flex-none border-0 bg-sidebar"
      style={{ width: `${props.width}px` }}
    >
      <View class="h-20 flex-none px-5 flex flex-row items-center gap-3">
        <View class="w-9 h-9 flex-none flex items-center justify-center">
          <Icon source={meowMark} size={34} />
        </View>
        <Show when={props.expanded}>
          <Text class="min-w-0 flex-1 truncate text-base font-semibold text-sidebar-primary">
            meow-starter
          </Text>
        </Show>
      </View>

      <SidebarContent contentClass="px-3 py-2">
        <SidebarMenu value="">
          <ForValue each={navItems}>
            {(item) => <NavButton {...item} expanded={props.expanded} />}
          </ForValue>
        </SidebarMenu>
      </SidebarContent>

      <SidebarFooter class="p-3 flex flex-col gap-3 border-0 bg-transparent">
        <View
          class="min-w-0 h-10 px-3 flex flex-row items-center gap-2 rounded-lg bg-sidebar-raised"
          aria-label="Application status"
        >
          <ActivityStatus
            label={props.expanded ? "Ready to build" : "Ready"}
            tone="success"
            animated
            textClass="text-sidebar-primary"
          />
        </View>
        <View class="h-8 flex flex-row items-center justify-between gap-2">
          <Show when={props.expanded}>
            <Text class="text-xs text-sidebar-muted">v0.1.0 · native</Text>
          </Show>
          <Button
            aria-label={props.expanded ? "Collapse sidebar" : "Expand sidebar"}
            variant="ghost"
            size="icon"
            class="text-sidebar-muted"
            onClick={() => props.onExpandedChange(!props.expanded)}
          >
            <Icon
              source={props.expanded ? panelLeftClose : panelLeftOpen}
              size={16}
            />
          </Button>
        </View>
      </SidebarFooter>
    </Sidebar>
  );
}

function CapabilityCard(props: (typeof capabilities)[number]): JSX.Element {
  return (
    <Card
      role="group"
      aria-label={`${props.title} capability`}
      variant="raised"
      size="sm"
      class="min-w-0 h-44"
    >
      <CardContent class="h-full items-center justify-between text-center">
        <View class="w-14 h-14 flex-none flex items-center justify-center rounded-2xl bg-success-surface">
          <Icon source={props.icon} size={26} class="text-accent" />
        </View>
        <View class="min-w-0 w-full flex flex-col items-center gap-1">
          <Text class="w-full truncate text-base font-semibold text-primary">
            {props.title}
          </Text>
          <Text class="w-full truncate text-xs text-muted">{props.detail}</Text>
        </View>
        <Badge
          variant={props.status === "Preview" ? "outline" : "success"}
          weight="normal"
        >
          {props.status}
        </Badge>
      </CardContent>
    </Card>
  );
}

const initialTasks = [
  { id: "release", label: "Build release", state: "Done" },
  { id: "sqlite", label: "Wire up SQLite", state: "Done" },
  { id: "updater", label: "Implement updater", state: "In progress" },
  { id: "tray", label: "Polish tray menu", state: "To do" },
] as const;

export function OverviewPage(): JSX.Element {
  const compact = createWindowMatch({ maxWidth: 900 }, useWindow());
  const [completed, setCompleted] = createSignal(
    new Set(["release", "sqlite"]),
  );
  const toggle = (id: string) => {
    setCompleted((previous) => {
      const next = new Set(previous);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };
  return (
    <View
      role="group"
      class="min-w-0 flex flex-col gap-5"
      aria-label="Meow Starter overview"
    >
      <View class="min-w-0 flex flex-row items-center justify-between gap-4">
        <View class="min-w-0 flex flex-row items-center gap-4">
          <View class="w-14 h-14 flex-none flex items-center justify-center rounded-2xl bg-success-surface">
            <Icon source={meowMark} size={45} />
          </View>
          <View class="min-w-0 flex flex-col gap-1">
            <Text
              role="heading"
              class="truncate text-3xl font-semibold text-primary"
            >
              meow-starter
            </Text>
            <Text class="truncate text-sm text-secondary">
              A calm foundation for native desktop products.
            </Text>
          </View>
        </View>
        <View class="h-7 px-3 flex-none flex items-center rounded-full border border-success-primary bg-success-surface">
          <ActivityStatus label="Healthy" tone="success" />
        </View>
      </View>

      <View
        role="group"
        aria-label="Runtime capabilities"
        class="grid gap-4"
        classList={{ "grid-cols-2": compact(), "grid-cols-4": !compact() }}
      >
        <ForValue each={capabilities}>
          {(capability) => <CapabilityCard {...capability} />}
        </ForValue>
      </View>

      <View
        class="grid gap-4"
        classList={{ "grid-cols-1": compact(), "grid-cols-2": !compact() }}
      >
        <Card role="group" aria-label="Starter tasks" size="sm" class="min-w-0">
          <CardHeader class="flex-row items-center gap-2 pb-3 border-b border-subtle">
            <Icon source={list} size={17} class="text-accent" />
            <CardTitle>Tasks</CardTitle>
          </CardHeader>
          <CardContent class="gap-0 pt-1">
            <ForValue each={initialTasks}>
              {(task) => {
                const done = () => completed().has(task.id);
                return (
                  <Button
                    variant="ghost"
                    class="w-full h-10 px-0 justify-start rounded-none border-0 border-b border-subtle"
                    aria-label={`${done() ? "Mark incomplete" : "Mark complete"}: ${task.label}`}
                    onClick={() => toggle(task.id)}
                  >
                    <View
                      class="w-5 h-5 flex-none flex items-center justify-center rounded-md border"
                      classList={{
                        "border-accent bg-accent": done(),
                        "border-strong bg-transparent": !done(),
                      }}
                    >
                      <Show when={done()}>
                        <Icon source={check} size={13} class="text-on-accent" />
                      </Show>
                    </View>
                    <Text
                      class="min-w-0 flex-1 truncate text-left text-sm"
                      classList={{
                        "text-muted": done(),
                        "text-primary": !done(),
                      }}
                    >
                      {task.label}
                    </Text>
                    <Badge
                      variant={
                        done()
                          ? "success"
                          : task.state === "In progress"
                            ? "outline"
                            : "secondary"
                      }
                      weight="normal"
                    >
                      {done() ? "Done" : task.state}
                    </Badge>
                  </Button>
                );
              }}
            </ForValue>
          </CardContent>
        </Card>

        <Card
          role="group"
          aria-label="Quick information"
          size="sm"
          class="min-w-0"
        >
          <CardHeader class="flex-row items-center gap-2 pb-3 border-b border-subtle">
            <Icon source={info} size={17} class="text-accent" />
            <CardTitle>Quick info</CardTitle>
          </CardHeader>
          <CardContent class="gap-0 pt-1">
            {[
              [code, "Renderer", "Vello Hybrid"],
              [sparkles, "UI", "Solid 2"],
              [database, "Storage", "SQLite KV"],
              [monitor, "Platform", "Cross-platform"],
            ].map(([icon, label, value]) => (
              <View class="h-10 min-w-0 flex flex-row items-center gap-3 border-b border-subtle">
                <Icon source={icon} size={16} class="text-secondary" />
                <Text class="min-w-0 flex-1 truncate text-sm text-primary">
                  {label}
                </Text>
                <Badge variant="success" weight="normal">
                  {value}
                </Badge>
              </View>
            ))}
          </CardContent>
        </Card>
      </View>
    </View>
  );
}

export function PlaceholderPage(props: {
  title: string;
  description: string;
  icon: string;
}): JSX.Element {
  return (
    <View class="min-w-0 h-full flex items-center justify-center">
      <Card class="w-full max-w-2xl" size="lg">
        <CardContent class="items-center py-12 text-center">
          <View class="w-16 h-16 flex items-center justify-center rounded-2xl bg-success-surface">
            <Icon source={props.icon} size={28} class="text-accent" />
          </View>
          <Text role="heading" class="text-xl font-semibold text-primary">
            {props.title}
          </Text>
          <Text class="max-w-lg whitespace-normal text-sm text-secondary">
            {props.description}
          </Text>
          <Button variant="default">Open {props.title.toLowerCase()}</Button>
        </CardContent>
      </Card>
    </View>
  );
}

export function AppShell(props: { children?: JSX.Element }): JSX.Element {
  const location = useLocation();
  const [expanded, setExpanded] = createSignal(true);
  const sidebarWidth = createTransition(() => (expanded() ? 216 : 64), {
    duration: 0.2,
    ease: "easeOut",
  });
  return (
    <WindowFrame
      rounded={false}
      material="solid"
      class="flex flex-row bg-canvas text-primary"
    >
      <AppSidebar
        expanded={expanded()}
        width={sidebarWidth.value()}
        onExpandedChange={setExpanded}
      />
      <View class="min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas">
        <TitleBar class="h-10 flex-none bg-surface">
          <TitleBarDragRegion class="min-w-0 pl-5">
            <Text class="truncate text-xs text-muted">
              Meow Starter · Wabou
            </Text>
          </TitleBarDragRegion>
          <WindowControls />
        </TitleBar>
        <PageViewport
          resetKey={location().pathname}
          contentClass="max-w-5xl mx-auto px-6 py-6"
        >
          {props.children}
        </PageViewport>
      </View>
    </WindowFrame>
  );
}
