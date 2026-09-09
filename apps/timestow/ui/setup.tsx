import {
  Button,
  ButtonGroup,
  ContentState,
  DirectoryPicker,
  Icon,
  Input,
  PageHeader,
  PageViewport,
  PasswordInput,
  Text,
  useDialog,
  useNavigate,
  View,
} from "@wabou/ui";
import archiveRestore from "lucide-static/icons/archive-restore.svg?raw";
import databaseZap from "lucide-static/icons/database-zap.svg?raw";
import fileCog from "lucide-static/icons/file-cog.svg?raw";
import folderOpen from "lucide-static/icons/folder-open.svg?raw";
import { createEffect, createSignal, Show } from "solid-js";
import type { RepositoryKind } from "./api";
import { useTimestowSession } from "./session";

export type RepositoryMode = "create" | "open";

export interface BackupConnectionFormProps {
  mode: RepositoryMode;
  name: string;
  path: string;
  repositoryKind?: RepositoryKind;
  passwordSecret: string;
  confirmationSecret: string;
  pending?: RepositoryMode;
  locked?: boolean;
  error?: string;
  onModeChange(mode: RepositoryMode): void;
  onRepositoryKindChange?(kind: RepositoryKind): void;
  onNameChange(value: string): void;
  onPathChange(value: string): void;
  onSubmit(): void;
}

export function BackupConnectionForm(props: BackupConnectionFormProps) {
  const nativeDialog = useDialog();
  const [passwordPresent, setPasswordPresent] = createSignal(false);
  const [confirmationPresent, setConfirmationPresent] = createSignal(false);
  const ready = () =>
    Boolean(
      props.name.trim() &&
        props.path.trim() &&
        (passwordPresent() || importingConfig()) &&
        (!creating() || props.locked || confirmationPresent()),
    ) && !props.pending;
  const creating = () => props.mode === "create";
  const repositoryKind = () => props.repositoryKind ?? "local";
  const importingConfig = () =>
    !creating() && repositoryKind() === "rusticConfig";

  createEffect(
    () => props.passwordSecret,
    () => {
      setPasswordPresent(false);
    },
  );
  createEffect(
    () => props.confirmationSecret,
    () => {
      setConfirmationPresent(false);
    },
  );
  createEffect(
    () => creating() && !props.locked,
    (requiresConfirmation) => {
      if (!requiresConfirmation) setConfirmationPresent(false);
    },
  );
  const storageDescription = () => {
    if (props.locked)
      return "This profile reconnects to its existing encrypted repository.";
    return creating()
      ? "Choose an empty folder for the new repository."
      : importingConfig()
        ? "Choose the rustic TOML profile that already connects to S3."
        : "Choose an existing rustic or restic repository.";
  };
  const submitLabel = () => {
    if (props.pending) return creating() ? "Creating…" : "Opening…";
    if (props.locked) return "Unlock backup";
    return creating() ? "Create backup" : "Open repository";
  };
  const footerDescription = () => {
    if (props.locked)
      return "Your saved folders and snapshot history will be restored.";
    return creating()
      ? "You can choose folders to back up after creating the profile."
      : "Timestow will read the repository without changing it.";
  };

  return (
    <View
      role="group"
      aria-label={props.locked ? "Unlock backup" : "Backup repository setup"}
      class="w-full flex flex-col gap-4 rounded-xl border border-subtle bg-surface p-5 shadow-sm"
    >
      <Show when={!props.locked}>
        <View class="flex flex-col gap-1.5">
          <Text class="text-xs font-semibold tracking-wide text-muted">
            Repository
          </Text>
          <ButtonGroup class="self-start" aria-label="Repository setup mode">
            <Button
              variant="ghost"
              selected={creating()}
              aria-label="Create a new repository"
              onClick={() => props.onModeChange("create")}
            >
              <Icon source={databaseZap} size={15} /> Create new
            </Button>
            <Button
              variant="ghost"
              selected={!creating()}
              aria-label="Open an existing repository"
              onClick={() => props.onModeChange("open")}
            >
              <Icon source={archiveRestore} size={15} /> Open existing
            </Button>
          </ButtonGroup>
          <Text class="text-sm text-muted">
            {creating()
              ? "Start a new encrypted backup repository in an empty folder."
              : "Connect a repository previously created by rustic or restic."}
          </Text>
          <Show when={!creating()}>
            <ButtonGroup
              class="self-start"
              size="sm"
              variant="outline"
              aria-label="Existing repository source"
            >
              <Button
                size="sm"
                variant="outline"
                selected={repositoryKind() === "local"}
                aria-pressed={repositoryKind() === "local"}
                aria-label="Repository folder"
                onClick={() => props.onRepositoryKindChange?.("local")}
              >
                <Icon source={folderOpen} size={14} /> Repository folder
              </Button>
              <Button
                size="sm"
                variant="outline"
                selected={importingConfig()}
                aria-pressed={importingConfig()}
                aria-label="Rustic config"
                onClick={() => props.onRepositoryKindChange?.("rusticConfig")}
              >
                <Icon source={fileCog} size={14} /> Rustic config
              </Button>
            </ButtonGroup>
          </Show>
        </View>
      </Show>

      <View class="flex flex-col gap-1.5">
        <Text class="font-medium">Backup name</Text>
        <Input
          aria-label="Backup name"
          value={props.name}
          disabled={props.locked}
          onInput={(event) => props.onNameChange(event.currentTarget.value)}
          placeholder="Photos"
        />
        <Text class="text-xs text-muted">
          This is the name shown in the sidebar.
        </Text>
      </View>
      <View class="flex flex-col gap-1.5">
        <Text class="font-medium">Storage location</Text>
        <Text class="text-sm text-muted">{storageDescription()}</Text>
      </View>
      <Show
        when={importingConfig()}
        fallback={
          <DirectoryPicker
            aria-label="Repository location"
            value={props.path}
            onValueChange={props.onPathChange}
            disabled={props.locked}
            placeholder="/data/backups/my-repository"
            browseLabel="Choose folder"
          />
        }
      >
        <View class="min-w-0 flex flex-row items-center gap-2">
          <Input
            aria-label="Rustic configuration file"
            class="min-w-0 flex-1"
            value={props.path}
            disabled={props.locked}
            placeholder="~/.config/rustic/rustic.toml"
            onInput={(event) => props.onPathChange(event.currentTarget.value)}
          />
          <Button
            variant="outline"
            disabled={props.locked}
            aria-label="Choose config"
            onClick={() => {
              void nativeDialog
                .open({
                  title: "Choose rustic configuration",
                  filters: [{ name: "TOML", extensions: ["toml"] }],
                })
                .then((paths) => {
                  const path = paths?.[0];
                  if (path) props.onPathChange(path);
                });
            }}
          >
            Choose config
          </Button>
        </View>
      </Show>
      <View
        role="group"
        aria-label="Repository credentials"
        class={
          creating() && !props.locked
            ? "grid grid-cols-2 gap-4"
            : "flex flex-col gap-1.5"
        }
      >
        <View class="flex flex-col gap-1.5">
          <Text class="font-medium">
            Repository password{importingConfig() ? " (if not in config)" : ""}
          </Text>
          <Show when={props.passwordSecret} keyed>
            {(secret) => (
              <PasswordInput
                aria-label="Repository password"
                secret={secret}
                placeholder="Required to encrypt or unlock"
                disabled={Boolean(props.pending)}
                onSecretStateChange={(event) =>
                  setPasswordPresent(event.hasValue)
                }
                onKeyDown={(event) => {
                  if (
                    event.key !== "Enter" ||
                    !ready() ||
                    (creating() && !props.locked)
                  )
                    return;
                  event.preventDefault();
                  props.onSubmit();
                }}
              />
            )}
          </Show>
        </View>
        <Show when={creating() && !props.locked}>
          <View class="min-w-0 flex flex-col gap-1.5">
            <Text class="font-medium">Confirm password</Text>
            <PasswordInput
              aria-label="Confirm repository password"
              secret={props.confirmationSecret}
              placeholder="Enter the same password again"
              disabled={Boolean(props.pending)}
              onSecretStateChange={(event) =>
                setConfirmationPresent(event.hasValue)
              }
              onKeyDown={(event) => {
                if (event.key !== "Enter" || !ready()) return;
                event.preventDefault();
                props.onSubmit();
              }}
            />
          </View>
        </Show>
      </View>
      <Text class="text-xs text-muted">
        {importingConfig()
          ? "S3 credentials and repository passwords are read by Rust and never exposed to JavaScript. Imported repositories are read-only in Timestow."
          : "Passwords stay in this app process and are never written to the profile database. New repositories require confirmation to prevent typos."}
      </Text>
      <Show when={props.error}>
        {(message) => (
          <View
            role="alert"
            class="rounded-md border border-danger bg-danger-surface px-3 py-2"
          >
            <Text class="text-sm text-danger-primary">{message()}</Text>
          </View>
        )}
      </Show>
      <View class="flex flex-row items-center justify-between gap-4">
        <Text class="min-w-0 text-xs text-muted">{footerDescription()}</Text>
        <Button
          aria-label={submitLabel()}
          class="flex-none"
          disabled={!ready()}
          onClick={props.onSubmit}
        >
          {submitLabel()}
        </Button>
      </View>
    </View>
  );
}

export function SetupPage() {
  const session = useTimestowSession();
  const navigate = useNavigate();
  const [name, setName] = createSignal("");
  const [path, setPath] = createSignal("");
  const [mode, setMode] = createSignal<RepositoryMode>("create");
  const [repositoryKind, setRepositoryKind] =
    createSignal<RepositoryKind>("local");
  const [pending, setPending] = createSignal<"create" | "open">();
  const [error, setError] = createSignal<string>();
  const passwordSecret = () =>
    `timestow:repository:${session.pendingUnlock()?.id ?? "new"}`;
  const confirmationSecret = () => `${passwordSecret()}:confirmation`;

  async function connect(mode: "create" | "open") {
    if (!name().trim() || !path().trim() || pending()) return;
    setPending(mode);
    setError(undefined);
    try {
      const locked = session.pendingUnlock();
      await session.connectProfile(mode, {
        id: locked?.id,
        name: name(),
        repositoryPath: path(),
        repositoryKind: repositoryKind(),
        passwordSlot: passwordSecret(),
        confirmationSlot: mode === "create" ? confirmationSecret() : undefined,
        sources: locked?.sources,
      });
      await navigate({ to: "/snapshots" });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(undefined);
    }
  }

  createEffect(
    () => session.pendingUnlock(),
    (profile) => {
      setName(profile?.name ?? "");
      setPath(profile?.repositoryPath ?? "");
      setMode(profile ? "open" : "create");
      setRepositoryKind(profile?.repositoryKind ?? "local");
    },
  );

  return (
    <PageViewport contentClass="min-h-full px-6 py-5">
      <View class="w-full max-w-3xl mx-auto flex flex-col gap-5">
        <PageHeader
          title={session.pendingUnlock() ? "Unlock backup" : "Create a backup"}
          description={
            session.pendingUnlock()
              ? "Enter the repository password to continue. Passwords are never stored in the profile database."
              : "Choose what this backup is called and where its encrypted snapshots are stored."
          }
        />
        <Show
          when={!session.loading()}
          fallback={
            <ContentState
              state="loading"
              title="Checking repository state"
              description="Reading the current process configuration."
            />
          }
        >
          <BackupConnectionForm
            mode={mode()}
            name={name()}
            path={path()}
            repositoryKind={repositoryKind()}
            passwordSecret={passwordSecret()}
            confirmationSecret={confirmationSecret()}
            pending={pending()}
            locked={Boolean(session.pendingUnlock())}
            error={error()}
            onModeChange={(nextMode) => {
              setMode(nextMode);
              if (nextMode === "create") setRepositoryKind("local");
              setError(undefined);
            }}
            onRepositoryKindChange={(kind) => {
              setRepositoryKind(kind);
              setPath("");
              setError(undefined);
            }}
            onNameChange={setName}
            onPathChange={setPath}
            onSubmit={() => void connect(mode())}
          />
        </Show>
      </View>
    </PageViewport>
  );
}
