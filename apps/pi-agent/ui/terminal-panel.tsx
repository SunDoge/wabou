import { Terminal } from "@wabou/terminal";
import {
  Button,
  ContextMenu,
  type ContextMenuTriggerProps,
  ForEntity,
  type Handle,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import plus from "lucide-static/icons/plus.svg?raw";
import squareTerminal from "lucide-static/icons/square-terminal.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import {
  type Accessor,
  createEffect,
  createSignal,
  type Setter,
  untrack,
} from "solid-js";

interface TerminalTab {
  id: number;
  cwd: string;
  title: Accessor<string>;
  setTitle: Setter<string>;
  exited: Accessor<boolean>;
  setExited: Setter<boolean>;
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

function createTerminalTab(id: number, cwd: string): TerminalTab {
  const [title, setTitle] = createSignal(workspaceName(cwd));
  const [exited, setExited] = createSignal(false);
  return { id, cwd, title, setTitle, exited, setExited };
}

export function AgentTerminalPanel(
  props: AgentTerminalPanelProps,
): ReturnType<typeof View> {
  let nextId = 2;
  const initialCwd = untrack(() => props.cwd);
  const [tabs, setTabs] = createSignal<readonly TerminalTab[]>([
    createTerminalTab(1, initialCwd),
  ]);
  const [activeId, setActiveId] = createSignal<number>(1);
  const handles = new Map<number, Handle>();

  const focusTerminal = (id: number | undefined) => {
    if (id !== undefined) queueMicrotask(() => handles.get(id)?.focus());
  };
  const addTab = () => {
    const id = nextId++;
    const tab = createTerminalTab(id, props.cwd);
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
  const updateTitle = (tab: TerminalTab, title: string) =>
    tab.setTitle((current) => (current === title ? current : title));

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
          <ForEntity each={tabs()} by={(tab) => tab.id}>
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
                  {tab.title()}
                  {tab.exited() ? " · exited" : ""}
                </Text>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={`Close terminal ${tab.title()}`}
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
          </ForEntity>
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
        <ForEntity each={tabs()} by={(tab) => tab.id}>
          {(tab) => {
            const terminal = (trigger: ContextMenuTriggerProps) => (
              <View
                {...trigger}
                role="group"
                aria-label={`Terminal ${tab.id} surface`}
                class="w-full h-full"
                style={{ display: activeId() === tab.id ? "flex" : "none" }}
              >
                <Terminal
                  ref={(node) => {
                    handles.set(tab.id, node);
                    if (untrack(activeId) === tab.id) focusTerminal(tab.id);
                  }}
                  aria-label={`Terminal ${tab.id}`}
                  cwd={tab.cwd}
                  class="w-full h-full overflow-hidden rounded-md bg-slate-950 text-slate-200"
                  inheritTheme
                  fontFamily="Hack Nerd Font Mono"
                  fontSize="13px"
                  lineHeight="19px"
                  cursorBlink={false}
                  selectionBackground="#2563eb80"
                  onTerminalTitleChange={(event) =>
                    updateTitle(
                      tab,
                      event.title?.trim() || workspaceName(tab.cwd),
                    )
                  }
                  onTerminalExit={() => tab.setExited(true)}
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
        </ForEntity>
      </View>
    </View>
  );
}
