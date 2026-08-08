import { Button, Input } from "@wabou/components";
import { ScrollArea, Text, View } from "@wabou/primitives";
import Copy from "lucide-solid/icons/copy";
import Eye from "lucide-solid/icons/eye";
import KeyRound from "lucide-solid/icons/key-round";
import LayoutGrid from "lucide-solid/icons/layout-grid";
import LockKeyhole from "lucide-solid/icons/lock-keyhole";
import RefreshCw from "lucide-solid/icons/refresh-cw";
import ShieldCheck from "lucide-solid/icons/shield-check";
import Star from "lucide-solid/icons/star";
import StickyNote from "lucide-solid/icons/sticky-note";
import Settings from "lucide-solid/icons/settings";
import { For, Show, createMemo, createSignal, type Accessor } from "solid-js";
import type { ItemDetails, VaultSnapshot } from "./model";
import { matches } from "./model";
import { SettingsScreen } from "./settings-screen";

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
  type Section = "vault" | "settings";
  const [filter, setFilter] = createSignal<Filter>("all");
  const [section, setSection] = createSignal<Section>("vault");
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

  function filterIcon(id: Filter) {
    if (id === "favorite") return <Star size={16} />;
    if (id === "login") return <KeyRound size={16} />;
    if (id === "note") return <StickyNote size={16} />;
    return <LayoutGrid size={16} />;
  }

  function itemIcon(kind: string) {
    return kind === "note" ? <StickyNote size={16} /> : <KeyRound size={16} />;
  }

  return (
    <View class="h-full w-full flex bg-slate-950 text-slate-100">
      <View class="w-48 shrink-0 border-r border-slate-800 bg-slate-900 p-3 flex flex-col gap-5">
        <View class="px-2 py-2 flex flex-col gap-2">
          <View class="flex items-center gap-2 text-sky-400">
            <ShieldCheck size={20} />
            <Text class="text-lg font-semibold text-slate-100">Wabou Vault</Text>
          </View>
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
                onClick={() => {
                  setFilter(item.id);
                  setSection("vault");
                }}
              >
                <View class="flex items-center gap-2">
                  {filterIcon(item.id)}
                  <Text>{item.label}</Text>
                </View>
              </Button>
            )}
          </For>
        </View>
        <View class="flex-1" />
        <Button
          class="w-full justify-start"
          size="sm"
          variant={section() === "settings" ? "secondary" : "ghost"}
          onClick={() => setSection("settings")}
        >
          <View class="flex items-center gap-2">
            <Settings size={16} />
            <Text>Settings</Text>
          </View>
        </Button>
        <View class="border-t border-slate-800 pt-3 flex flex-col gap-2">
          <Text class="px-2 text-xs text-slate-500">{props.snapshot().email}</Text>
          <Button class="w-full justify-start" size="sm" variant="ghost" onClick={props.lock}>
            <View class="flex items-center gap-2">
              <LockKeyhole size={16} />
              <Text>Lock vault</Text>
            </View>
          </Button>
        </View>
      </View>
      <Show
        when={section() === "vault"}
        fallback={<SettingsScreen email={props.snapshot().email} />}
      >
        <View class="w-64 shrink-0 border-r border-slate-800 flex flex-col">
        <View class="p-4 border-b border-slate-800 flex flex-col gap-3">
          <View class="flex items-center gap-2">
            <Text class="text-lg font-semibold text-slate-100">My vault</Text>
            <View class="flex-1" />
            <Button size="sm" variant="ghost" disabled={props.busy()} onClick={props.refresh}>
              <View class="flex items-center gap-2">
                <RefreshCw size={15} />
                <Text>Sync</Text>
              </View>
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
                  <View class="w-8 h-8 shrink-0 rounded-md bg-slate-900 text-slate-400 flex items-center justify-center">
                    {itemIcon(item.kind)}
                  </View>
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
          <View class="flex items-center gap-2 text-slate-500">
            <Eye size={16} />
            <Text class="text-sm">Item details</Text>
          </View>
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
                    <Button
                      size="icon"
                      variant="outline"
                      aria-label="Copy username"
                      onClick={() => props.copy("username")}
                    >
                      <Copy size={16} />
                    </Button>
                  </View>
                </Show>
                <Show when={details().hasPassword}>
                  <View class="rounded-lg border border-slate-800 p-4 flex items-center gap-4">
                    <View class="flex-1 flex flex-col gap-1">
                      <Text class="text-xs text-slate-500">Password</Text>
                      <Text class="text-sm text-slate-300">••••••••••••</Text>
                    </View>
                    <Button
                      size="icon"
                      variant="outline"
                      aria-label="Copy password"
                      onClick={() => props.copy("password")}
                    >
                      <Copy size={16} />
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
      </Show>
    </View>
  );
}
