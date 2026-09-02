import {
  Badge,
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  Icon,
  IconFrame,
  Text,
  useNavigate,
  useRouteActive,
  View,
} from "@wabou/ui";
import archive from "lucide-static/icons/archive.svg?raw";
import database from "lucide-static/icons/database.svg?raw";
import history from "lucide-static/icons/history.svg?raw";
import type { JSX } from "solid-js";
import { useRusticSession } from "./session";

function NavButton(props: { to: string; label: string; icon: string }) {
  const navigate = useNavigate();
  const active = useRouteActive(props.to);
  return (
    <Button
      variant="ghost"
      size="sm"
      selected={active()}
      class={active() ? "bg-selected" : undefined}
      onClick={() => void navigate({ to: props.to })}
    >
      <Icon source={props.icon} size={15} />
      {props.label}
    </Button>
  );
}

export function AppShell(props: { children?: JSX.Element }) {
  const session = useRusticSession();
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 min-h-0 flex flex-col bg-canvas text-primary">
          <View class="h-14 flex-none px-5 flex flex-row items-center gap-3 border-b border-subtle bg-surface shadow-sm">
            <IconFrame
              source={archive}
              size="sm"
              iconSize={18}
              variant="solid"
            />
            <View class="flex flex-col mr-4">
              <Text class="font-semibold">Rustic GUI</Text>
              <Text class="text-xs text-muted">Snapshots without the CLI</Text>
            </View>
            <NavButton to="/" label="Repository" icon={database} />
            <NavButton to="/snapshots" label="Snapshots" icon={history} />
            <View class="flex-1" />
            <Badge
              variant={session.status().connected ? "success" : "secondary"}
            >
              {session.status().connected ? "Repository open" : "Not connected"}
            </Badge>
          </View>
          <View class="min-w-0 min-h-0 flex-1">{props.children}</View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
