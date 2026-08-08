import "@wabou/core";
import "virtual:wabou-stylesheet";

import { Button, Input } from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import { mount, useHost } from "@wabou/solid-renderer";
import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import type {
  ItemDetails,
  ItemDraft,
  LoginOutcome,
  MutationOutcome,
  TwoFactorOption,
  VaultSnapshot,
} from "./model";
import { unwrap } from "./model";
import { TwoFactorScreen } from "./two-factor-screen";
import { VaultScreen } from "./vault-screen";

declare module "@wabou/solid-renderer" {
  interface HostCapabilities {
    readonly vault: {
      login(request: string): Promise<string>;
      submitTwoFactor(request: string): Promise<string>;
      sendTwoFactorEmail(): Promise<string>;
      cancelTwoFactor(): Promise<string>;
      refresh(): Promise<string>;
      details(id: string): Promise<string>;
      copy(id: string, field: "username" | "password"): Promise<string>;
      createItem(request: string): Promise<string>;
      updateItem(id: string, request: string): Promise<string>;
      deleteItem(id: string): Promise<string>;
      lock(): Promise<string>;
      isLocked(): Promise<string>;
    };
  }
}

type Region = "us" | "eu" | "self-hosted";

function App() {
  const host = useHost();
  const [region, setRegion] = createSignal<Region>("us");
  const [serverUrl, setServerUrl] = createSignal("");
  const [email, setEmail] = createSignal("");
  const [snapshot, setSnapshot] = createSignal<VaultSnapshot>();
  const [twoFactor, setTwoFactor] = createSignal<TwoFactorOption[]>();
  const [twoFactorProvider, setTwoFactorProvider] = createSignal("");
  const [twoFactorCode, setTwoFactorCode] = createSignal("");
  const [selected, setSelected] = createSignal<ItemDetails>();
  const [query, setQuery] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [notice, setNotice] = createSignal("");

  onMount(() => {
    const timer = setInterval(async () => {
      if (!snapshot()) return;
      try {
        if (unwrap<boolean>(await host.vault.isLocked())) {
          setSnapshot(undefined);
          setSelected(undefined);
          setQuery("");
          setError("Vault locked after five minutes of inactivity.");
        }
      } catch {
        // Other actions surface bridge errors; status polling stays quiet.
      }
    }, 15_000);
    onCleanup(() => clearInterval(timer));
  });

  async function run<T>(operation: () => Promise<T>): Promise<T | undefined> {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      return await operation();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Unknown vault error.");
      return undefined;
    } finally {
      setBusy(false);
    }
  }

  async function login() {
    const result = await run(async () =>
      unwrap<LoginOutcome>(
        await host.vault.login(
          JSON.stringify({
            region: region(),
            serverUrl: region() === "self-hosted" ? serverUrl() : undefined,
            email: email(),
          }),
        ),
      ),
    );
    if (result) applyLoginOutcome(result);
  }

  function applyLoginOutcome(outcome: LoginOutcome) {
    if (outcome.status === "authenticated") {
      setSnapshot(outcome.snapshot);
      setTwoFactor(undefined);
      setTwoFactorCode("");
      return;
    }
    setTwoFactor(outcome.providers);
    setTwoFactorProvider(
      outcome.providers.find((provider) => provider.supported)?.id ?? "",
    );
    setTwoFactorCode("");
  }

  async function verifyTwoFactor() {
    const result = await run(async () =>
      unwrap<LoginOutcome>(
        await host.vault.submitTwoFactor(
          JSON.stringify({
            provider: twoFactorProvider(),
            token: twoFactorCode(),
          }),
        ),
      ),
    );
    if (result) applyLoginOutcome(result);
  }

  async function sendTwoFactorEmail() {
    const sent = await run(async () =>
      unwrap<boolean>(await host.vault.sendTwoFactorEmail()),
    );
    if (sent) setNotice("Verification code sent. Check your email.");
  }

  async function cancelTwoFactor() {
    await host.vault.cancelTwoFactor();
    setTwoFactor(undefined);
    setTwoFactorProvider("");
    setTwoFactorCode("");
    setNotice("");
    setError("");
  }

  async function refresh() {
    const result = await run(async () =>
      unwrap<VaultSnapshot>(await host.vault.refresh()),
    );
    if (result) {
      setSnapshot(result);
      setSelected(undefined);
      setNotice("Vault synchronized.");
    }
  }

  async function selectItem(id: string) {
    const result = await run(async () =>
      unwrap<ItemDetails>(await host.vault.details(id)),
    );
    if (result) setSelected(result);
  }

  async function copy(field: "username" | "password") {
    const item = selected();
    if (!item) return;
    const copied = await run(async () => unwrap<boolean>(await host.vault.copy(item.id, field)));
    if (copied) setNotice("Copied. The clipboard is cleared after 30 seconds if unchanged.");
  }

  async function createItem(draft: ItemDraft) {
    const result = await run(async () =>
      unwrap<MutationOutcome>(await host.vault.createItem(JSON.stringify(draft))),
    );
    if (!result) return false;
    setSnapshot(result.snapshot);
    if (result.id) await selectItem(result.id);
    setNotice("Vault item created.");
    return true;
  }

  async function updateItem(id: string, draft: ItemDraft) {
    const result = await run(async () =>
      unwrap<MutationOutcome>(await host.vault.updateItem(id, JSON.stringify(draft))),
    );
    if (!result) return false;
    setSnapshot(result.snapshot);
    await selectItem(id);
    setNotice("Vault item updated.");
    return true;
  }

  async function deleteItem(id: string) {
    const result = await run(async () =>
      unwrap<MutationOutcome>(await host.vault.deleteItem(id)),
    );
    if (!result) return false;
    setSnapshot(result.snapshot);
    setSelected(undefined);
    setNotice("Vault item moved to trash.");
    return true;
  }

  async function lock() {
    await host.vault.lock();
    setSnapshot(undefined);
    setSelected(undefined);
    setQuery("");
    setNotice("");
    setError("");
  }

  return (
    <View class="h-full w-full bg-slate-950 text-slate-100">
      <Show
        when={snapshot()}
        fallback={
          <Show
            when={twoFactor()}
            fallback={<View class="h-full w-full flex items-center justify-center p-8">
            <View class="w-96 rounded-xl border border-slate-800 bg-slate-900 p-6 flex flex-col gap-4">
              <Text class="text-2xl font-semibold text-slate-100">Wabou Vault</Text>
              <Text class="text-sm text-slate-400">
                Read-only Bitwarden SDK demo.
              </Text>
              <View class="flex gap-2">
                <For each={["us", "eu", "self-hosted"] as Region[]}>
                  {(value) => (
                    <Button
                      size="sm"
                      variant={region() === value ? "default" : "outline"}
                      onClick={() => setRegion(value)}
                    >
                      {value === "self-hosted" ? "Self-hosted" : value.toUpperCase()}
                    </Button>
                  )}
                </For>
              </View>
              <Show when={region() === "self-hosted"}>
                <Input
                  placeholder="https://vault.example.com"
                  value={serverUrl()}
                  onInput={(event) => setServerUrl(event.currentTarget.value)}
                />
              </Show>
              <View class="flex flex-col gap-2">
                <Text class="text-xs font-medium text-slate-400">Email</Text>
                <Input
                  placeholder="name@example.com"
                  value={email()}
                  onInput={(event) => setEmail(event.currentTarget.value)}
                />
              </View>
              <View class="flex flex-col gap-2">
                <Text class="text-xs font-medium text-slate-400">Master password</Text>
                <View class="h-10 rounded-md border border-slate-700 bg-slate-950 px-3 justify-center">
                  <secure-input class="w-full text-base text-slate-100" placeholder="Master password" />
                </View>
              </View>
              <Show when={error()}>
                <Text class="text-sm text-red-500">{error()}</Text>
              </Show>
              <Button disabled={busy()} onClick={login}>
                {busy() ? "Unlocking…" : "Unlock read-only vault"}
              </Button>
              <Text class="text-xs text-slate-500">
                No edits, autofill, or tray integration.
              </Text>
            </View>
          </View>}
          >
            {(providers) => (
              <TwoFactorScreen
                providers={providers()}
                provider={twoFactorProvider()}
                code={twoFactorCode()}
                busy={busy()}
                error={error()}
                notice={notice()}
                setProvider={setTwoFactorProvider}
                setCode={setTwoFactorCode}
                verify={verifyTwoFactor}
                sendEmail={sendTwoFactorEmail}
                cancel={cancelTwoFactor}
              />
            )}
          </Show>
        }
      >
        <VaultScreen
          snapshot={() => snapshot()!}
          selected={selected}
          query={query}
          busy={busy}
          error={error}
          notice={notice}
          setQuery={setQuery}
          refresh={refresh}
          lock={lock}
          selectItem={selectItem}
          copy={copy}
          createItem={createItem}
          updateItem={updateItem}
          deleteItem={deleteItem}
        />
      </Show>
    </View>
  );
}

mount(() => <App />);
