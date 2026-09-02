import {
  Button,
  ContentState,
  DirectoryPicker,
  Input,
  PageHeader,
  PageViewport,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import { createEffect, createSignal, Show } from "solid-js";
import { useRusticSession } from "./session";

export function SetupPage() {
  const session = useRusticSession();
  const navigate = useNavigate();
  const [name, setName] = createSignal("");
  const [path, setPath] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [pending, setPending] = createSignal<"create" | "open">();
  const [error, setError] = createSignal<string>();

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
        password: password(),
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
      setPassword("");
    },
  );

  return (
    <PageViewport contentClass="min-h-full px-8 py-7">
      <View class="w-full max-w-3xl mx-auto flex flex-col gap-7">
        <PageHeader
          title={session.pendingUnlock() ? "Unlock backup" : "Create a backup"}
          description={
            session.pendingUnlock()
              ? "Enter the repository password to continue. Passwords are never stored in the profile database."
              : "Choose what this backup is called and where its encrypted snapshots are stored."
          }
        />
        <Show when={session.loading()}>
          <ContentState
            state="loading"
            title="Checking repository state"
            description="Reading the current process configuration."
          />
        </Show>
        <View class="w-full flex flex-col gap-5 rounded-xl border border-subtle bg-surface p-6 shadow-sm">
          <View class="flex flex-col gap-1.5">
            <Text class="font-medium">Backup name</Text>
            <Input
              value={name()}
              disabled={Boolean(session.pendingUnlock())}
              onInput={(event) => setName(event.currentTarget.value)}
              placeholder="Photos"
            />
            <Text class="text-xs text-muted">
              This is the name shown in the sidebar.
            </Text>
          </View>
          <View class="flex flex-col gap-1.5">
            <Text class="font-medium">Storage location</Text>
            <Text class="text-sm text-muted">
              Choose an empty folder to create a repository, or an existing
              rustic/restic repository to open it.
            </Text>
          </View>
          <DirectoryPicker
            value={path()}
            onValueChange={setPath}
            disabled={Boolean(session.pendingUnlock())}
            placeholder="/data/backups/my-repository"
            browseLabel="Choose folder"
          />
          <View class="flex flex-col gap-1.5">
            <Text class="font-medium">Repository password</Text>
            <Input
              value={password()}
              onInput={(event) => setPassword(event.currentTarget.value)}
              placeholder="Required to encrypt or unlock the repository"
            />
            <Text class="text-xs text-muted">
              Passwords stay in this app process and are never written to the
              profile database.
            </Text>
          </View>
          <Show when={error()}>
            {(message) => (
              <View class="rounded-md border border-danger bg-danger-surface px-3 py-2">
                <Text class="text-sm text-danger-primary">{message()}</Text>
              </View>
            )}
          </Show>
          <View class="flex flex-row justify-end gap-2">
            <Show
              when={!session.pendingUnlock()}
              fallback={
                <Button
                  disabled={!password() || Boolean(pending())}
                  onClick={() => void connect("open")}
                >
                  {pending() === "open" ? "Unlocking…" : "Unlock backup"}
                </Button>
              }
            >
              <Button
                variant="outline"
                disabled={
                  !name().trim() ||
                  !path().trim() ||
                  !password() ||
                  Boolean(pending())
                }
                onClick={() => void connect("open")}
              >
                {pending() === "open" ? "Opening…" : "Use existing storage"}
              </Button>
              <Button
                disabled={
                  !name().trim() ||
                  !path().trim() ||
                  !password() ||
                  Boolean(pending())
                }
                onClick={() => void connect("create")}
              >
                {pending() === "create" ? "Creating…" : "Create backup"}
              </Button>
            </Show>
          </View>
        </View>
      </View>
    </PageViewport>
  );
}
