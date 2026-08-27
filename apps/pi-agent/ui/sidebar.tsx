import {
  Button,
  DirectoryPicker,
  Icon,
  Input,
  ScrollArea,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import folderGit from "lucide-static/icons/folder-git-2.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import { For } from "solid-js";
import { i18n, m } from "./i18n";
import type { AgentWorkspace } from "./workspace";

interface SidebarProps {
  agents: readonly AgentWorkspace[];
  activeId: string;
  active: AgentWorkspace;
  pending: boolean;
  select: (id: string) => void;
  add: () => void;
  update: (patch: Partial<AgentWorkspace>) => void;
  start: () => Promise<boolean>;
  openSettings: () => void;
}

export function Sidebar(props: SidebarProps) {
  return (
    <View class="w-72 flex-none min-h-0 flex flex-col border-r border-subtle bg-surface shadow-sm">
      <View class="flex-none flex items-center gap-3 px-4 pt-4 pb-2">
        <View class="w-9 h-9 rounded-lg bg-accent flex items-center justify-center">
          <Icon source={bot} size={19} class="text-on-accent" />
        </View>
        <View class="min-w-0 flex-1">
          <Text class="font-semibold text-base">
            {i18n.message(m.app_name, {})}
          </Text>
          <Text class="text-xs text-muted">
            {i18n.message(m.app_tagline, {})}
          </Text>
        </View>
        <Button
          variant="ghost"
          size="icon"
          aria-label={i18n.message(m.add_agent, {})}
          onClick={props.add}
        >
          <Icon source={plus} size={16} />
        </Button>
      </View>

      <ScrollArea class="flex-1 min-h-0" contentClass="px-4 py-2 gap-3">
        <View class="gap-1">
          <For each={props.agents}>
            {(agent) => (
              <Button
                variant={agent.id === props.activeId ? "secondary" : "ghost"}
                class="w-full justify-start"
                onClick={() => props.select(agent.id)}
              >
                <View
                  class={
                    agent.state.connection === "running"
                      ? "w-2 h-2 rounded-full bg-accent"
                      : agent.state.connection === "ready"
                        ? "w-2 h-2 rounded-full bg-success-primary"
                        : "w-2 h-2 rounded-full bg-muted"
                  }
                />
                <Text class="min-w-0 text-sm">{agent.name}</Text>
              </Button>
            )}
          </For>
        </View>

        <View class="rounded-lg border border-subtle bg-surface-muted p-3 gap-2">
          <View class="flex items-center gap-2">
            <Icon source={folderGit} size={14} />
            <Text class="text-xs font-medium">
              {i18n.message(m.workspace, {})}
            </Text>
          </View>
          <DirectoryPicker
            value={props.active.cwd}
            onValueChange={(cwd) => props.update({ cwd })}
            placeholder={i18n.message(m.choose_repository, {})}
            browseLabel={i18n.message(m.browse, {})}
          />
          <Button
            variant={
              props.active.state.connection === "ready"
                ? "secondary"
                : "default"
            }
            class="w-full"
            disabled={
              props.pending || props.active.state.connection === "running"
            }
            onClick={() => void props.start()}
          >
            <Icon source={play} size={14} />
            {props.pending
              ? i18n.message(m.starting, {})
              : props.active.state.connection === "ready"
                ? i18n.message(m.restart_agent, {})
                : i18n.message(m.start_agent, {})}
          </Button>
        </View>

        <View class="rounded-lg border border-subtle bg-surface-muted p-3 gap-2">
          <Text class="text-xs font-medium">
            {i18n.message(m.provider, {})}
          </Text>
          <Input
            value={props.active.provider}
            onInput={(event) =>
              props.update({ provider: event.currentTarget.value })
            }
            placeholder={i18n.message(m.provider_placeholder, {})}
            aria-label="Provider"
          />
          <Input
            value={props.active.model}
            onInput={(event) =>
              props.update({ model: event.currentTarget.value })
            }
            placeholder={i18n.message(m.model_optional, {})}
            aria-label="Model ID"
          />
        </View>

        <View class="rounded-lg border border-subtle bg-surface-muted p-3 gap-2">
          <Text class="text-xs font-medium">
            {i18n.message(m.agent_proxy, {})}
          </Text>
          <Input
            value={props.active.proxy}
            onInput={(event) =>
              props.update({ proxy: event.currentTarget.value })
            }
            placeholder={i18n.message(m.proxy_placeholder, {})}
            aria-label="Proxy URL"
          />
          <Input
            value={props.active.noProxy}
            onInput={(event) =>
              props.update({ noProxy: event.currentTarget.value })
            }
            placeholder="127.0.0.1,localhost"
            aria-label="Proxy bypass list"
          />
          <Text class="text-xs text-muted whitespace-normal">
            {i18n.message(m.proxy_scope, {})}
          </Text>
        </View>
      </ScrollArea>

      <View class="flex-none border-t border-subtle px-4 py-3 gap-2">
        <Button
          variant="ghost"
          class="w-full justify-start"
          onClick={props.openSettings}
        >
          <Icon source={settings} size={16} />
          {i18n.message(m.settings, {})}
        </Button>
        <Text class="text-xs text-muted whitespace-normal">
          {i18n.message(m.agent_isolation, {})}
        </Text>
      </View>
    </View>
  );
}
