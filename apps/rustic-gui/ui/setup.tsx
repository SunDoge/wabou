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
import { createSignal, Show, untrack } from "solid-js";
import { useRusticApi } from "./api";
import { useRusticSession } from "./session";

export function SetupPage() {
  const api = useRusticApi();
  const session = useRusticSession();
  const navigate = useNavigate();
  const [path, setPath] = createSignal(
    untrack(() => session.status().repositoryPath ?? ""),
  );
  const [password, setPassword] = createSignal("");
  const [pending, setPending] = createSignal<"create" | "open">();
  const [error, setError] = createSignal<string>();

  async function connect(mode: "create" | "open") {
    if (!path().trim() || pending()) return;
    setPending(mode);
    setError(undefined);
    try {
      const status = await (mode === "create"
        ? api.createRepository({ path: path().trim(), password: password() })
        : api.openRepository({ path: path().trim(), password: password() }));
      session.setStatus(status);
      await navigate({ to: "/snapshots" });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(undefined);
    }
  }

  return (
    <PageViewport contentClass="min-h-full px-8 py-7">
      <View class="w-full max-w-3xl mx-auto flex flex-col gap-7">
        <PageHeader
          title="Connect a backup repository"
          description="Start with a local rustic repository. S3 and other OpenDAL destinations will plug into the same workspace later."
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
            <Text class="font-medium">Repository folder</Text>
            <Text class="text-sm text-muted">
              Choose an empty folder to create a repository, or an existing
              rustic/restic repository to open it.
            </Text>
          </View>
          <DirectoryPicker
            value={path()}
            onValueChange={setPath}
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
              Alpha limitation: secure native password storage is the next
              framework integration; the value currently stays in this app
              process.
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
            <Button
              variant="outline"
              disabled={!path().trim() || Boolean(pending())}
              onClick={() => void connect("open")}
            >
              {pending() === "open" ? "Opening…" : "Open existing"}
            </Button>
            <Button
              disabled={!path().trim() || Boolean(pending())}
              onClick={() => void connect("create")}
            >
              {pending() === "create" ? "Creating…" : "Create repository"}
            </Button>
          </View>
        </View>
      </View>
    </PageViewport>
  );
}
