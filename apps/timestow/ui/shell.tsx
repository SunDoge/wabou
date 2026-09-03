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
  SidebarMenuSuffix,
  Text,
  useNavigate,
  useRouteActive,
  View,
} from "@wabou/ui";
import archive from "lucide-static/icons/archive.svg?raw";
import database from "lucide-static/icons/database.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import { For as ForValue, type JSX, Show } from "solid-js";
import type { BackupProfile } from "./api";
import { useTimestowSession } from "./session";

export interface TimestowSidebarProps {
  active: "new" | string;
  profiles: readonly BackupProfile[];
  unlockedProfileIds: readonly string[];
  onCreate(): void;
  onSelectProfile(profileId: string): void;
}

export function TimestowSidebar(props: TimestowSidebarProps) {
  return (
    <Sidebar
      aria-label="Primary navigation"
      class="w-56 border-r border-subtle bg-surface-muted"
    >
      <SidebarHeader class="h-16 px-4 flex items-center gap-3 border-0 bg-surface-muted">
        <IconFrame source={archive} size="sm" iconSize={18} variant="solid" />
        <View class="min-w-0 flex flex-col">
          <Text class="truncate text-sm font-semibold">Timestow</Text>
          <Text class="truncate text-xs text-muted">Backup workspace</Text>
        </View>
      </SidebarHeader>

      <SidebarContent contentClass="px-3 py-3">
        <SidebarGroup aria-label="Backups">
          <SidebarGroupLabel>Backups</SidebarGroupLabel>
          <SidebarMenu value={props.active}>
            <Show
              when={props.profiles.length > 0}
              fallback={
                <Text class="px-2 py-3 text-xs text-muted">
                  Create your first backup to begin.
                </Text>
              }
            >
              <ForValue each={props.profiles}>
                {(profile) => (
                  <SidebarMenuButton
                    value={profile.id}
                    aria-label={profile.name}
                    onClick={() => props.onSelectProfile(profile.id)}
                  >
                    <SidebarMenuIcon>
                      <Icon source={database} size={16} />
                    </SidebarMenuIcon>
                    <SidebarMenuLabel>{profile.name}</SidebarMenuLabel>
                    <SidebarMenuSuffix>
                      <View
                        aria-label={
                          props.unlockedProfileIds.includes(profile.id)
                            ? `${profile.name} unlocked`
                            : `${profile.name} locked`
                        }
                        class={`w-2 h-2 rounded-full ${props.unlockedProfileIds.includes(profile.id) ? "bg-success-primary" : "bg-muted"}`}
                      />
                    </SidebarMenuSuffix>
                  </SidebarMenuButton>
                )}
              </ForValue>
            </Show>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter class="p-3 bg-surface-muted">
        <SidebarMenuButton
          selected={props.active === "new"}
          aria-label="New backup"
          onClick={props.onCreate}
        >
          <SidebarMenuIcon>
            <Icon source={plus} size={16} />
          </SidebarMenuIcon>
          <SidebarMenuLabel>New backup</SidebarMenuLabel>
        </SidebarMenuButton>
      </SidebarFooter>
    </Sidebar>
  );
}

export function AppShell(props: { children?: JSX.Element }) {
  const session = useTimestowSession();
  const navigate = useNavigate();
  const createActive = useRouteActive("/");
  const snapshotsActive = useRouteActive("/snapshots");

  async function selectProfile(profileId: string): Promise<void> {
    try {
      const unlocked = await session.activateProfile(profileId);
      await navigate({ to: unlocked ? "/snapshots" : "/" });
    } catch (cause) {
      session.setError(cause instanceof Error ? cause.message : String(cause));
    }
  }
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <View class="w-full h-full min-w-0 min-h-0 flex flex-row bg-canvas text-primary">
          <TimestowSidebar
            active={
              session.pendingUnlock()?.id ??
              (createActive() || !snapshotsActive()
                ? "new"
                : (session.activeProfile()?.id ?? "new"))
            }
            profiles={session.profiles()}
            unlockedProfileIds={session.runtime().unlockedProfileIds}
            onCreate={() => {
              session.beginCreate();
              void navigate({ to: "/" });
            }}
            onSelectProfile={(profileId) => void selectProfile(profileId)}
          />
          <View class="min-w-0 min-h-0 flex-1 flex flex-col">
            {props.children}
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
