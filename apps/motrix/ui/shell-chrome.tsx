import {
  Button,
  Icon,
  px,
  Text,
  TitleBar,
  TitleBarDragRegion,
  useNavigate,
  useRouteActive,
  useWindow,
  View,
} from "@wabou/ui";
import bell from "lucide-static/icons/bell.svg?raw";
import download from "lucide-static/icons/download.svg?raw";
import gauge from "lucide-static/icons/gauge.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import panelLeftClose from "lucide-static/icons/panel-left-close.svg?raw";
import panelLeftOpen from "lucide-static/icons/panel-left-open.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { For, Show } from "solid-js";

const navigation = [
  ["/", "Dashboard", gauge],
  ["/downloads", "Downloads", download],
] as const;

interface ShellChromeProps {
  sidebarOpen: boolean;
  sidebarWidth: number;
  onSidebarOpenChange(open: boolean): void;
  onNewTask(): void;
}

export function AppTitleBar(props: ShellChromeProps) {
  const window = useWindow();
  return (
    <TitleBar class="px-2 bg-canvas">
      <View
        class="h-full flex-none px-2 flex items-center gap-2"
        style={{ width: px(props.sidebarWidth) }}
      >
        <Show
          when={props.sidebarOpen}
          fallback={
            <Button
              aria-label="Show sidebar"
              size="icon"
              variant="ghost"
              onClick={() => props.onSidebarOpenChange(true)}
            >
              <Icon source={panelLeftOpen} size={18} />
            </Button>
          }
        >
          <View class="w-8 h-8 flex-none rounded-lg bg-accent flex items-center justify-center">
            <Icon source={download} size={17} class="text-on-accent" />
          </View>
          <Text class="min-w-0 flex-1 truncate text-sm font-semibold text-primary">
            Motrix
          </Text>
          <Button
            aria-label="Hide sidebar"
            size="icon"
            variant="ghost"
            onClick={() => props.onSidebarOpenChange(false)}
          >
            <Icon source={panelLeftClose} size={17} />
          </Button>
          <Button
            aria-label="New task"
            size="icon"
            variant="ghost"
            onClick={props.onNewTask}
          >
            <Icon source={plus} size={18} />
          </Button>
        </Show>
      </View>
      <TitleBarDragRegion class="min-w-0 justify-center">
        <Text class="text-xs text-muted">Motrix · Wabou</Text>
      </TitleBarDragRegion>
      <Button
        aria-label="Minimize window"
        size="icon"
        variant="ghost"
        onClick={() => window.minimize()}
      >
        <Icon source={minus} size={16} />
      </Button>
      <Button
        aria-label={window.maximized() ? "Restore window" : "Maximize window"}
        size="icon"
        variant="ghost"
        onClick={() => window.setMaximized(!window.maximized())}
      >
        <Icon source={square} size={14} />
      </Button>
      <Button
        aria-label="Close window"
        size="icon"
        variant="ghost"
        onClick={() => window.close()}
      >
        <Icon source={x} size={17} />
      </Button>
    </TitleBar>
  );
}

function NavigationButton(props: {
  path: string;
  label: string;
  icon: string;
  expanded: boolean;
}) {
  const navigate = useNavigate();
  const active = useRouteActive(props.path);
  return (
    <Button
      variant="ghost"
      selected={active()}
      aria-current={active() ? "page" : undefined}
      aria-label={props.label}
      class="w-full h-12 text-base font-medium text-primary"
      classList={{
        "px-3 justify-start": props.expanded,
        "px-0 justify-center": !props.expanded,
        "bg-selected": active(),
      }}
      onClick={() => navigate({ to: props.path })}
    >
      <Icon source={props.icon} size={19} />
      <Show when={props.expanded}>
        <Text class="text-base font-medium text-primary">{props.label}</Text>
      </Show>
    </Button>
  );
}

export function AppSidebar(props: ShellChromeProps) {
  return (
    <View
      class="flex-none overflow-hidden px-2 py-2 flex flex-col rounded-xl bg-surface-muted"
      style={{ width: px(props.sidebarWidth) }}
    >
      <Show when={!props.sidebarOpen}>
        <View class="mb-4 flex flex-col items-center gap-2">
          <Button
            aria-label="New task"
            size="icon"
            variant="ghost"
            onClick={props.onNewTask}
          >
            <Icon source={plus} size={18} />
          </Button>
        </View>
      </Show>
      <View class="flex flex-col gap-1">
        <For each={navigation}>
          {([path, label, icon]) => (
            <NavigationButton
              path={path}
              label={label}
              icon={icon}
              expanded={props.sidebarOpen}
            />
          )}
        </For>
      </View>
      <View class="flex-1" />
      <NavigationButton
        path="/notifications"
        label="Notifications"
        icon={bell}
        expanded={props.sidebarOpen}
      />
      <View class="my-2 border-t border-subtle" />
      <NavigationButton
        path="/settings"
        label="Settings"
        icon={settings}
        expanded={props.sidebarOpen}
      />
    </View>
  );
}
