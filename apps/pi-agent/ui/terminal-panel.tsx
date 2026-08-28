import { Terminal } from "@wabou/terminal";
import {
  Button,
  ContextMenu,
  type ContextMenuTriggerProps,
  type Handle,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import plus from "lucide-static/icons/plus.svg?raw";
import squareTerminal from "lucide-static/icons/square-terminal.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createEffect, createSignal, For, untrack } from "solid-js";

interface TerminalTab {
  id: number;
  cwd: string;
  title: string;
  exited: boolean;
}

export interface AgentTerminalPanelProps {
  cwd: string;
  open: boolean;
  close(): void;
  dispose(): void;
}

function workspaceName(cwd: string): string {
  const normalized = cwd.replace(/[\\/]+$/, "");
  return normalized.split(/[\\/]/).pop() || "Shell";
}

export function AgentTerminalPanel(
  props: AgentTerminalPanelProps,
): ReturnType<typeof View> {
  let nextId = 2;
  const initialCwd = untrack(() => props.cwd);
  const [tabs, setTabs] = createSignal<readonly TerminalTab[]>([
    {
      id: 1,
      cwd: initialCwd,
      title: workspaceName(initialCwd),
      exited: false,
    },
  ]);
  const [activeId, setActiveId] = createSignal<number>(1);
  const handles = new Map<number, Handle>();

  const focusTerminal = (id: number | undefined) => {
    if (id !== undefined) queueMicrotask(() => handles.get(id)?.focus());
  };
  const addTab = () => {
    const id = nextId++;
    const tab: TerminalTab = {
      id,
      cwd: props.cwd,
      title: workspaceName(props.cwd),
      exited: false,
    };
    setTabs((current) => [...current, tab]);
    setActiveId(id);
  };
  const closeTab = (id: number) => {
    const current = tabs();
    const index = current.findIndex((tab) => tab.id === id);
    const remaining = current.filter((tab) => tab.id !== id);
    handles.delete(id);
    setTabs(remaining);
    if (activeId() === id) {
      const next = remaining[Math.min(index, remaining.length - 1)];
      setActiveId(next?.id);
    }
    if (remaining.length === 0) props.dispose();
  };
  const updateTab = (id: number, patch: Partial<TerminalTab>) => {
    setTabs((current) =>
      current.map((tab) => (tab.id === id ? { ...tab, ...patch } : tab)),
    );
  };

  createEffect(
    () => activeId(),
    (id) => focusTerminal(id),
  );

  return (
    <View
      role="region"
      aria-label="Terminal panel"
      class="h-64 min-h-40 flex-none flex flex-col overflow-hidden border-t border-strong bg-slate-950 text-slate-200 shadow-lg"
      style={{ display: props.open ? "flex" : "none" }}
    >
      <View class="h-9 flex-none flex flex-row items-center gap-1 border-b border-slate-700 bg-slate-900 px-2">
        <View
          role="tablist"
          aria-label="Terminal sessions"
          class="min-w-0 flex-1 flex flex-row items-center gap-1 overflow-hidden"
        >
          <For each={tabs()}>
            {(tab) => (
              <View
                role="tab"
                aria-selected={activeId() === tab.id}
                class="h-7 min-w-28 max-w-56 flex flex-row items-center gap-2 rounded-md px-2 text-xs"
                classList={{
                  "bg-slate-700 text-white": activeId() === tab.id,
                  "text-slate-400 hover:bg-slate-800": activeId() !== tab.id,
                }}
                onClick={() => setActiveId(tab.id)}
              >
                <Icon source={squareTerminal} size={13} class="flex-none" />
                <Text class="min-w-0 flex-1 truncate">
                  {tab.title}
                  {tab.exited ? " · exited" : ""}
                </Text>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={`Close terminal ${tab.title}`}
                  class="w-5 h-5 flex-none text-slate-400"
                  style={{ padding: 0 }}
                  onClick={(event) => {
                    event.stopPropagation();
                    closeTab(tab.id);
                  }}
                >
                  <Icon source={x} size={12} />
                </Button>
              </View>
            )}
          </For>
        </View>
        <Button
          variant="ghost"
          size="icon"
          aria-label="New terminal"
          class="w-7 h-7 flex-none text-slate-300"
          onClick={addTab}
        >
          <Icon source={plus} size={14} />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Close terminal panel"
          class="w-7 h-7 flex-none text-slate-300"
          onClick={props.close}
        >
          <Icon source={x} size={14} />
        </Button>
      </View>
      <View class="min-h-0 flex-1 p-1">
        <For each={tabs()}>
          {(tab) => {
            const terminal = (trigger: ContextMenuTriggerProps) => (
              <View
                {...trigger}
                role="group"
                aria-label={`${tab.title} terminal surface`}
                class="w-full h-full"
                style={{ display: activeId() === tab.id ? "flex" : "none" }}
              >
                <Terminal
                  ref={(node) => {
                    handles.set(tab.id, node);
                    if (untrack(activeId) === tab.id) focusTerminal(tab.id);
                  }}
                  aria-label={`${tab.title} terminal`}
                  cwd={tab.cwd}
                  class="w-full h-full overflow-hidden rounded-md bg-slate-950 text-slate-200"
                  inheritTheme
                  fontFamily="Hack Nerd Font Mono"
                  fontSize="13px"
                  lineHeight="19px"
                  selectionBackground="#2563eb80"
                  onTerminalTitleChange={(event) =>
                    updateTab(tab.id, {
                      title: event.title?.trim() || workspaceName(tab.cwd),
                    })
                  }
                  onTerminalExit={() => updateTab(tab.id, { exited: true })}
                />
              </View>
            );
            return (
              <ContextMenu
                aria-label="Terminal actions"
                items={[
                  { id: "new", label: "New terminal" },
                  {
                    id: "close",
                    label: "Close terminal",
                    separatorBefore: true,
                  },
                ]}
                onAction={(id) => {
                  if (id === "new") addTab();
                  if (id === "close") closeTab(tab.id);
                }}
                trigger={terminal}
              />
            );
          }}
        </For>
      </View>
    </View>
  );
}
