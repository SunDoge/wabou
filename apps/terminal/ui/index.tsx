import "@wabou/ui";
import "virtual:wabou-stylesheet";
import { Terminal } from "@wabou/terminal";
import {
  Button,
  ColorThemeProvider,
  Column,
  ComponentsProvider,
  createActive,
  createHover,
  createShortcuts,
  createTabs,
  type Handle,
  Icon,
  mount,
  Row,
  Text,
  View,
} from "@wabou/ui";
import plus from "lucide-static/icons/plus.svg?raw";
import squareTerminal from "lucide-static/icons/square-terminal.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createSignal, For } from "solid-js";

interface ShellTab {
  id: number;
  title: () => string;
  setTitle: (title: string) => void;
}

function createShellTab(id: number): ShellTab {
  const [title, setTitle] = createSignal(`Shell ${id}`);
  return { id, title, setTitle };
}

function App() {
  let nextId = 3;
  const tabs = createTabs<ShellTab, number>({
    initialTabs: [createShellTab(1), createShellTab(2)],
    key: (tab) => tab.id,
  });
  const terminals = new Map<number, Handle>();
  const focusTerminal = (id: number) => {
    queueMicrotask(() => terminals.get(id)?.focus());
  };
  const addTab = () => {
    const id = nextId++;
    tabs.add(createShellTab(id));
    focusTerminal(id);
  };
  const selectTab = (id: number) => {
    tabs.select(id);
    focusTerminal(id);
  };
  const closeTab = (id: number) => {
    if (!tabs.close(id)) return;
    terminals.delete(id);
    let active = tabs.activeKey();
    if (active === undefined) {
      addTab();
      active = tabs.activeKey();
    }
    if (active !== undefined) focusTerminal(active);
  };
  const selectRelative = (direction: "next" | "previous") => {
    if (direction === "next") tabs.selectNext();
    else tabs.selectPrevious();
    const active = tabs.activeKey();
    if (active !== undefined) focusTerminal(active);
  };
  const shortcuts = createShortcuts({
    "Primary+T": addTab,
    "Primary+W": () => {
      const active = tabs.activeKey();
      if (active !== undefined) closeTab(active);
    },
    "Control+Tab": {
      allowRepeat: true,
      handler: () => selectRelative("next"),
    },
    "Control+Shift+Tab": {
      allowRepeat: true,
      handler: () => selectRelative("previous"),
    },
  });

  return (
    <ColorThemeProvider theme="dark" transition={false}>
      <ComponentsProvider theme="dark">
        <Column
          class="w-full h-full min-w-0 overflow-hidden bg-canvas text-primary"
          onKeyDown={shortcuts.bindings.onKeyDown}
        >
          <Row class="h-12 flex-none items-center gap-3 px-4 border-b border-subtle bg-surface">
            <View class="w-7 h-7 flex-none flex items-center justify-center rounded-md bg-accent text-on-accent shadow-sm">
              <Icon source={squareTerminal} size={16} />
            </View>
            <Column class="min-w-0">
              <Text class="text-sm font-semibold">Wabou Terminal</Text>
              <Text class="text-xs text-muted">Native PTY · rio-vt</Text>
            </Column>
            <Text class="ml-auto text-xs text-muted">
              Ctrl+Tab switches tabs
            </Text>
          </Row>

          <Row
            role="tablist"
            aria-label="Terminal sessions"
            class="h-10 flex-none min-w-0 items-end gap-1 px-3 border-b border-subtle bg-control"
          >
            <For each={tabs.tabs()}>
              {(tab) => {
                const hover = createHover();
                const press = createActive();
                const selected = () => tabs.activeKey() === tab.id;
                return (
                  <Row
                    ref={(node) => tabs.register(tab.id, node)}
                    role="tab"
                    aria-selected={selected()}
                    class="h-8 min-w-24 max-w-56 flex-1 items-center gap-2 px-3 rounded-md border border-transparent text-sm"
                    classList={{
                      "bg-surface border-subtle text-primary": selected(),
                      "bg-control-hover text-secondary":
                        !selected() && hover.hovered(),
                      "bg-control-pressed": press.active(),
                      "text-muted": !selected() && !hover.hovered(),
                    }}
                    onPointerEnter={hover.bindings.onPointerEnter}
                    onPointerLeave={() => {
                      hover.bindings.onPointerLeave();
                      press.bindings.onPointerLeave();
                    }}
                    onPointerDown={press.bindings.onPointerDown}
                    onPointerUp={press.bindings.onPointerUp}
                    onPointerCancel={press.bindings.onPointerCancel}
                    onClick={() => selectTab(tab.id)}
                    onKeyDown={(event) => tabs.handleKeyDown(tab.id, event)}
                  >
                    <Icon source={squareTerminal} size={14} />
                    <Text class="min-w-0 flex-1 overflow-hidden whitespace-nowrap">
                      {tab.title()}
                    </Text>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label={`Close ${tab.title()}`}
                      class="w-5 h-5 rounded-sm"
                      style={{ padding: 0 }}
                      onClick={(event) => {
                        event.stopPropagation();
                        closeTab(tab.id);
                      }}
                    >
                      <Icon source={x} size={13} />
                    </Button>
                  </Row>
                );
              }}
            </For>
            <Button
              variant="ghost"
              size="icon"
              aria-label="New terminal"
              class="w-8 h-8 mb-px"
              onClick={addTab}
            >
              <Icon source={plus} size={16} />
            </Button>
          </Row>

          <View class="min-h-0 flex-1 p-2 bg-canvas">
            <For each={tabs.tabs()}>
              {(tab) => (
                <Terminal
                  ref={(node) => {
                    terminals.set(tab.id, node);
                    if (tabs.activeKey() === tab.id) focusTerminal(tab.id);
                  }}
                  aria-label={`${tab.title()} terminal`}
                  class="w-full h-full overflow-hidden rounded-lg border border-subtle bg-slate-950 text-slate-200 shadow-sm"
                  style={{
                    display: tabs.activeKey() === tab.id ? "flex" : "none",
                  }}
                  inheritTheme
                  fontFamily="Hack Nerd Font Mono"
                  fontSize="14px"
                  lineHeight="20px"
                  selectionBackground="#2563eb80"
                  onTerminalTitleChange={(event) =>
                    tab.setTitle(event.title?.trim() || `Shell ${tab.id}`)
                  }
                  onTerminalExit={() => tab.setTitle(`${tab.title()} · exited`)}
                />
              )}
            </For>
          </View>
        </Column>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

mount(() => <App />);
