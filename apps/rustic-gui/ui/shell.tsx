import {
  ColorThemeProvider,
  ComponentsProvider,
  Icon,
  IconFrame,
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuIcon,
  SidebarMenuLabel,
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

export interface RusticSidebarProps {
  active: "/" | "/snapshots";
  connected: boolean;
  repositoryPath?: string;
  onNavigate(to: "/" | "/snapshots"): void;
}

export function RusticSidebar(props: RusticSidebarProps) {
  return (
    <Sidebar
      aria-label="Primary navigation"
      class="w-56 border-r border-subtle bg-surface-muted"
    >
      <SidebarHeader class="h-16 px-4 flex items-center gap-3 border-0 bg-surface-muted">
        <IconFrame source={archive} size="sm" iconSize={18} variant="solid" />
        <View class="min-w-0 flex flex-col">
          <Text class="truncate text-sm font-semibold">Rustic GUI</Text>
          <Text class="truncate text-xs text-muted">Backup workspace</Text>
        </View>
      </SidebarHeader>

      <SidebarContent contentClass="px-3 py-3">
        <SidebarGroup aria-label="Workspace">
          <SidebarGroupLabel>Workspace</SidebarGroupLabel>
          <SidebarMenu value={props.active}>
            <SidebarMenuButton
              value="/"
              aria-label="Repository"
              onClick={() => props.onNavigate("/")}
            >
              <SidebarMenuIcon>
                <Icon source={database} size={16} />
              </SidebarMenuIcon>
              <SidebarMenuLabel>Repository</SidebarMenuLabel>
            </SidebarMenuButton>
            <SidebarMenuButton
              value="/snapshots"
              aria-label="Snapshots"
              onClick={() => props.onNavigate("/snapshots")}
            >
              <SidebarMenuIcon>
                <Icon source={history} size={16} />
              </SidebarMenuIcon>
              <SidebarMenuLabel>Snapshots</SidebarMenuLabel>
            </SidebarMenuButton>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter class="px-4 py-3 bg-surface-muted">
        <View
          role="status"
          aria-label={props.connected ? "Repository open" : "Not connected"}
          class="min-w-0 flex flex-row items-center gap-2"
        >
          <View
            class={`w-2 h-2 flex-none rounded-full ${props.connected ? "bg-success-primary" : "bg-muted"}`}
          />
          <View class="min-w-0 flex-1 flex flex-col gap-0.5">
            <Text class="truncate text-xs font-medium text-secondary">
              {props.connected ? "Repository open" : "Not connected"}
            </Text>
            <Text class="truncate text-xs text-muted">
              {props.repositoryPath ?? "Choose a local repository"}
            </Text>
          </View>
        </View>
      </SidebarFooter>
    </Sidebar>
  );
}

export function AppShell(props: { children?: JSX.Element }) {
  const session = useRusticSession();
  const navigate = useNavigate();
  const snapshotsActive = useRouteActive("/snapshots");
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 min-h-0 flex flex-row bg-canvas text-primary">
          <RusticSidebar
            active={snapshotsActive() ? "/snapshots" : "/"}
            connected={session.status().connected}
            repositoryPath={session.status().repositoryPath}
            onNavigate={(to) => void navigate({ to })}
          />
          <View class="min-w-0 min-h-0 flex-1 flex flex-col">
            {props.children}
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
