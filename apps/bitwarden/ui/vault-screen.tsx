import { Button, Input } from "@wabou/components";
import { ScrollArea, Text, View } from "@wabou/primitives";
import { For, Show, createMemo, createSignal, type Accessor } from "solid-js";
import type { ItemDetails, VaultSnapshot } from "./model";
import { matches } from "./model";

export interface VaultScreenProps {
  snapshot: Accessor<VaultSnapshot>;
  selected: Accessor<ItemDetails | undefined>;
  query: Accessor<string>;
  busy: Accessor<boolean>;
  error: Accessor<string>;
  notice: Accessor<string>;
  setQuery(value: string): void;
  refresh(): void;
  lock(): void;
  selectItem(id: string): void;
  copy(field: "username" | "password"): void;
}

export function VaultScreen(props: VaultScreenProps) {
  type Filter = "all" | "login" | "note" | "favorite";
  const [filter, setFilter] = createSignal<Filter>("all");
  const visibleItems = createMemo(() =>
    props.snapshot().items.filter((item) => {
      const selected = filter();
      const categoryMatches =
        selected === "all" ||
        (selected === "favorite" ? item.favorite : item.kind === selected);
      return categoryMatches && matches(item, props.query());
    }),
  );

  const navItems: ReadonlyArray<{ id: Filter; label: string }> = [
    { id: "all", label: "All items" },
    { id: "favorite", label: "Favorites" },
    { id: "login", label: "Logins" },
    { id: "note", label: "Secure notes" },
  ];

  return (
    <View class="h-full w-full flex bg-slate-950 text-slate-100">
      <View class="w-48 shrink-0 border-r border-slate-800 bg-slate-900 p-3 flex flex-col gap-5">
        <View class="px-2 py-2 flex flex-col gap-1">
          <Text class="text-lg font-semibold text-slate-100">Wabou Vault</Text>
          <Text class="text-xs text-slate-500">Read-only desktop</Text>
        </View>
        <View class="flex flex-col gap-1">
          <Text class="px-2 pb-2 text-xs font-medium text-slate-500">VAULT</Text>
          <For each={navItems}>
            {(item) => (
              <Button
                class="w-full justify-start"
                size="sm"
                variant={filter() === item.id ? "secondary" : "ghost"}
                onClick={() => setFilter(item.id)}
              >
                {item.label}
              </Button>
            )}
          </For>
        </View>
        <View class="flex-1" />
        <View class="border-t border-slate-800 pt-3 flex flex-col gap-2">
          <Text class="px-2 text-xs text-slate-500">{props.snapshot().email}</Text>
          <Button class="w-full justify-start" size="sm" variant="ghost" onClick={props.lock}>
            Lock vault
          </Button>
        </View>
      </View>
      <View class="w-64 shrink-0 border-r border-slate-800 flex flex-col">
        <View class="p-4 border-b border-slate-800 flex flex-col gap-3">
          <View class="flex items-center gap-2">
            <Text class="text-lg font-semibold text-slate-100">My vault</Text>
            <View class="flex-1" />
            <Button size="sm" variant="ghost" disabled={props.busy()} onClick={props.refresh}>
              Sync
            </Button>
          </View>
          <Input
            placeholder="Search vault"
            value={props.query()}
            onInput={(event) => props.setQuery(event.currentTarget.value)}
          />
          <Text class="text-xs text-slate-500">{`${visibleItems().length} items`}</Text>
        </View>
        <Show when={props.error() || props.notice() || props.snapshot().decryptFailures > 0}>
          <View class="px-4 py-3 border-b border-slate-800 bg-slate-900">
            <Text class={props.error() ? "text-xs text-red-500" : "text-xs text-slate-400"}>
              {props.error() ||
                props.notice() ||
                `${props.snapshot().decryptFailures} item(s) could not be decrypted.`}
            </Text>
          </View>
        </Show>
        <ScrollArea class="min-h-0 flex-1">
          <View class="p-2 flex flex-col gap-1">
            <For each={visibleItems()}>
              {(item) => (
                <Button
                  class="h-auto justify-start px-3 py-3"
                  variant="ghost"
                  onClick={() => props.selectItem(item.id)}
                >
                  <View class="min-w-0 flex-1 flex flex-col items-start gap-1">
                    <Text class="text-sm font-medium text-slate-100">{item.name}</Text>
                    <Text class={item.favorite ? "text-xs text-sky-400" : "text-xs text-slate-500"}>
                      {item.favorite
                        ? `Favorite · ${item.subtitle || item.kind}`
                        : item.subtitle || item.kind}
                    </Text>
                  </View>
                </Button>
              )}
            </For>
          </View>
        </ScrollArea>
      </View>
      <View class="min-w-0 flex-1 flex flex-col">
        <View class="h-16 shrink-0 border-b border-slate-800 px-6 flex items-center">
          <Text class="text-sm text-slate-500">Item details</Text>
          <View class="flex-1" />
          <Text class="text-xs text-slate-600">Read only</Text>
        </View>
        <ScrollArea class="min-h-0 flex-1">
          <View class="p-8">
          <Show
            when={props.selected()}
            fallback={
              <Text class="text-slate-500">Select an item to inspect its read-only fields.</Text>
            }
          >
            {(details) => (
              <View class="w-full flex flex-col gap-6">
                <View class="flex flex-col gap-1">
                  <Text class="text-2xl font-semibold text-slate-100">{details().name}</Text>
                  <Text class="text-sm text-slate-500">{details().kind === "login" ? "Login" : "Secure note"}</Text>
                </View>
                <Show when={details().username}>
                  <View class="rounded-lg border border-slate-800 p-4 flex items-center gap-4">
                    <View class="min-w-0 flex-1 flex flex-col gap-1">
                      <Text class="text-xs text-slate-500">Username</Text>
                      <Text class="text-sm text-slate-100">{details().username}</Text>
                    </View>
                    <Button size="sm" variant="outline" onClick={() => props.copy("username")}>
                      Copy
                    </Button>
                  </View>
                </Show>
                <Show when={details().hasPassword}>
                  <View class="rounded-lg border border-slate-800 p-4 flex items-center gap-4">
                    <View class="flex-1 flex flex-col gap-1">
                      <Text class="text-xs text-slate-500">Password</Text>
                      <Text class="text-sm text-slate-300">••••••••••••</Text>
                    </View>
                    <Button size="sm" variant="outline" onClick={() => props.copy("password")}>
                      Copy
                    </Button>
                  </View>
                </Show>
                <For each={details().uris}>
                  {(uri) => (
                    <View class="rounded-lg border border-slate-800 p-4 flex flex-col gap-1">
                      <Text class="text-xs text-slate-500">Website</Text>
                      <Text class="text-sm text-slate-300">{uri}</Text>
                    </View>
                  )}
                </For>
              </View>
            )}
          </Show>
          </View>
        </ScrollArea>
        </View>
    </View>
  );
}
