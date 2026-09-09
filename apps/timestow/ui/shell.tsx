import {
  Alert,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  application,
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  ContextMenu,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Icon,
  IconFrame,
  Input,
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
import {
  createEffect,
  createSignal,
  For as ForValue,
  type JSX,
  Show,
} from "solid-js";
import type { BackupProfile } from "./api";
import { useTimestowSession } from "./session";

export interface TimestowSidebarProps {
  active: "new" | string;
  profiles: readonly BackupProfile[];
  unlockedProfileIds: readonly string[];
  onCreate(): void;
  onSelectProfile(profileId: string): void;
  onForgetProfile(profileId: string): Promise<void>;
  onRenameProfile(profileId: string, name: string): void | Promise<void>;
}

export function SessionErrorBanner(props: {
  message: string;
  onDismiss(): void;
}) {
  return (
    <Alert
      banner
      variant="error"
      title="Timestow needs attention"
      aria-label="Timestow error"
      class="flex-none"
      onClose={props.onDismiss}
    >
      {props.message}
    </Alert>
  );
}

export function TimestowSidebar(props: TimestowSidebarProps) {
  const [forgetCandidate, setForgetCandidate] = createSignal<BackupProfile>();
  const [renameCandidate, setRenameCandidate] = createSignal<BackupProfile>();
  const [renameName, setRenameName] = createSignal("");
  const [renaming, setRenaming] = createSignal(false);
  const [renameError, setRenameError] = createSignal<string>();
  const [forgetting, setForgetting] = createSignal(false);
  const [forgetError, setForgetError] = createSignal<string>();

  async function saveRename(close: () => void): Promise<void> {
    const profile = renameCandidate();
    const name = renameName().trim();
    if (!profile || !name || renaming()) return;
    setRenaming(true);
    setRenameError(undefined);
    try {
      await props.onRenameProfile(profile.id, name);
      close();
    } catch (cause) {
      setRenameError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setRenaming(false);
    }
  }

  async function confirmForget(): Promise<void> {
    const profile = forgetCandidate();
    if (!profile || forgetting()) return;
    setForgetting(true);
    setForgetError(undefined);
    try {
      await props.onForgetProfile(profile.id);
      setForgetCandidate(undefined);
    } catch (cause) {
      setForgetError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setForgetting(false);
    }
  }

  return (
    <>
      <Sidebar
        aria-label="Primary navigation"
        class="w-56 border-r border-subtle bg-surface-muted"
      >
        <SidebarHeader class="h-16 px-4 flex items-center gap-3 border-0 bg-surface-muted">
          <IconFrame
            source={archive}
            size="sm"
            iconSize={18}
            variant="solid"
            tone="accent"
          />
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
                    <ContextMenu
                      aria-label={`${profile.name} actions`}
                      items={[
                        {
                          id: "rename",
                          label: "Rename backup…",
                        },
                        {
                          id: "forget",
                          label: "Forget backup…",
                          destructive: true,
                          separatorBefore: true,
                        },
                      ]}
                      onAction={(action) => {
                        if (action === "rename") {
                          setRenameCandidate(profile);
                          setRenameName(profile.name);
                          setRenameError(undefined);
                        }
                        if (action === "forget") {
                          setForgetCandidate(profile);
                          setForgetError(undefined);
                        }
                      }}
                      trigger={(menu) => (
                        <SidebarMenuButton
                          ref={menu.ref}
                          value={profile.id}
                          aria-label={profile.name}
                          aria-haspopup={menu["aria-haspopup"]}
                          aria-expanded={menu["aria-expanded"]}
                          onClick={() => props.onSelectProfile(profile.id)}
                          onContextMenu={menu.onContextMenu}
                          onKeyDown={menu.onKeyDown}
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
                    />
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
      <Dialog
        aria-label="Rename backup"
        open={renameCandidate() !== undefined}
        onOpenChange={(open) => {
          if (!open) {
            setRenameCandidate(undefined);
            setRenameError(undefined);
          }
        }}
      >
        {(dialog) => (
          <View class="min-w-0 flex flex-col gap-4">
            <DialogHeader>
              <DialogTitle>Rename backup</DialogTitle>
              <DialogDescription>
                Change the name shown in Timestow. The repository and its
                snapshots stay untouched.
              </DialogDescription>
            </DialogHeader>
            <View class="min-w-0 flex flex-col gap-1.5">
              <Text class="text-sm font-medium">Backup name</Text>
              <Input
                aria-label="Backup name"
                value={renameName()}
                onInput={(event) => setRenameName(event.currentTarget.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void saveRename(dialog.close);
                  }
                }}
              />
              <Show when={renameError()}>
                {(message) => (
                  <Text role="alert" class="text-sm text-danger-primary">
                    {message()}
                  </Text>
                )}
              </Show>
            </View>
            <DialogFooter>
              <Button variant="outline" onClick={dialog.close}>
                Cancel
              </Button>
              <Button
                loading={renaming()}
                loadingLabel="Renaming…"
                disabled={
                  !renameName().trim() ||
                  renameName().trim() === renameCandidate()?.name
                }
                onClick={() => void saveRename(dialog.close)}
              >
                Rename
              </Button>
            </DialogFooter>
          </View>
        )}
      </Dialog>
      <AlertDialog
        aria-label="Forget backup"
        open={forgetCandidate() !== undefined}
        closeOnEscape={!forgetting()}
        onOpenChange={(open) => {
          if (!open && !forgetting()) {
            setForgetCandidate(undefined);
            setForgetError(undefined);
          }
        }}
      >
        <AlertDialogHeader>
          <AlertDialogTitle>
            Forget {forgetCandidate()?.name ?? "this backup"}?
          </AlertDialogTitle>
          <AlertDialogDescription>
            This removes the profile and its saved settings from this device.
            The encrypted repository and every snapshot inside it will remain
            untouched.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <Show when={forgetError()}>
          {(message) => (
            <Alert variant="destructive" title="Could not forget backup">
              {message()}
            </Alert>
          )}
        </Show>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={forgetting()}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            aria-label={`Forget ${forgetCandidate()?.name ?? "backup"}`}
            loading={forgetting()}
            loadingLabel="Forgetting…"
            onClick={(event) => {
              event.preventDefault();
              void confirmForget();
            }}
          >
            Forget backup
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialog>
    </>
  );
}

export function TimestowCloseGuard(props: {
  active: boolean;
  children?: JSX.Element;
  onQuit?: () => void;
}) {
  const [confirming, setConfirming] = createSignal(false);

  createEffect(
    () => props.active,
    (active) => {
      if (!active) setConfirming(false);
    },
  );

  return (
    <>
      <View
        role="group"
        aria-label="Timestow window"
        class="w-full h-full min-w-0 min-h-0 flex flex-row bg-canvas text-primary"
        onWindowCloseRequested={(event) => {
          if (!props.active) return;
          event.preventDefault();
          setConfirming(true);
        }}
      >
        {props.children}
      </View>
      <AlertDialog
        aria-label="Quit while an operation is running"
        open={confirming()}
        onOpenChange={setConfirming}
      >
        <AlertDialogHeader>
          <AlertDialogTitle>Quit during an active operation?</AlertDialogTitle>
          <AlertDialogDescription>
            A backup or restore is still running. Quitting now may leave the
            operation incomplete. Existing repository snapshots will not be
            deleted.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Keep working</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            onClick={() => (props.onQuit ?? application.exit)()}
          >
            Quit anyway
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialog>
    </>
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

  async function forgetProfile(profileId: string): Promise<void> {
    const wasActive =
      session.activeProfile()?.id === profileId ||
      session.pendingUnlock()?.id === profileId;
    try {
      await session.forgetProfile(profileId);
      if (wasActive) await navigate({ to: "/" });
    } catch (cause) {
      session.setError(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }
  return (
    <ColorThemeProvider theme="light">
      <ComponentsProvider theme="light">
        <TimestowCloseGuard active={session.hasActiveOperations()}>
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
            onRenameProfile={(profileId, name) =>
              session.renameProfile(profileId, name)
            }
            onForgetProfile={forgetProfile}
          />
          <View class="min-w-0 min-h-0 flex-1 flex flex-col">
            <Show when={session.error()}>
              {(message) => (
                <SessionErrorBanner
                  message={message()}
                  onDismiss={() => session.setError(undefined)}
                />
              )}
            </Show>
            <View class="min-w-0 min-h-0 flex-1 flex flex-col">
              {props.children}
            </View>
          </View>
        </TimestowCloseGuard>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
