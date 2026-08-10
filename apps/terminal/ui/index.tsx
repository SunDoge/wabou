// Terminal UI entry.
import "@wabou/core";
import "virtual:wabou-stylesheet";
import {
  Button,
  createActive,
  createHover,
  createShortcuts,
  createTabs,
  Text,
  View,
} from "@wabou/primitives";
import { type Handle, mount } from "@wabou/core";
import { Terminal } from "@wabou/terminal";
import { For } from "solid-js";

interface ShellTab {
  id: number;
  title: string;
}

function App() {
  let nextId = 3;
  const tabs = createTabs<ShellTab, number>({
    initialTabs: [
      { id: 1, title: "Shell 1" },
      { id: 2, title: "Shell 2" },
    ],
    key: (tab) => tab.id,
  });
  const terminals = new Map<number, Handle>();
  const focusTerminal = (id: number) => {
    Promise.resolve().then(() => terminals.get(id)?.focus());
  };
  const addTab = () => {
    const id = nextId++;
    tabs.add({ id, title: `Shell ${id}` });
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
  const shortcuts = createShortcuts({
    "Primary+T": addTab,
    "Primary+W": () => {
      const active = tabs.activeKey();
      if (active !== undefined) closeTab(active);
    },
    "Control+Tab": {
      allowRepeat: true,
      handler: () => {
        tabs.selectNext();
        const active = tabs.activeKey();
        if (active !== undefined) focusTerminal(active);
      },
    },
    "Control+Shift+Tab": {
      allowRepeat: true,
      handler: () => {
        tabs.selectPrevious();
        const active = tabs.activeKey();
        if (active !== undefined) focusTerminal(active);
      },
    },
  });

  return (
    <View
      class="w-full h-full flex flex-col bg-slate-950 text-slate-200"
      onKeyDown={shortcuts.bindings.onKeyDown}
    >
      <div class="h-10 flex flex-none items-center gap-1 px-2 bg-slate-900">
        <For each={tabs.tabs()}>
          {(tab) => {
            const hover = createHover();
            const press = createActive();
            const selected = () => tabs.activeKey() === tab.id;
            return (
              <View
                ref={(node) => tabs.register(tab.id, node)}
                role="tab"
                class="h-8 min-w-0 max-w-56 box-border flex flex-1 items-center gap-2 px-3 rounded-md text-sm"
                style={{
                  "background-color": press.active()
                    ? "#0f172a"
                    : selected()
                      ? "#1e293b"
                      : hover.hovered()
                        ? "#172033"
                        : "transparent",
                  "border-color": selected() ? "#38bdf8" : "transparent",
                  "border-bottom-width": "2px",
                  color: selected() ? "#f8fafc" : "#94a3b8",
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
                onKeyDown={(event: { key: string; preventDefault(): void }) =>
                  tabs.handleKeyDown(tab.id, event)
                }
              >
                <Text class="min-w-0 flex-1 overflow-hidden text-center">
                  {tab.title}
                </Text>
                <Button
                  unstyled
                  variant="ghost"
                  class="h-5 w-5 flex flex-none items-center justify-center rounded"
                  style={(state) => ({
                    "min-height": "20px",
                    padding: 0,
                    "border-width": 0,
                    "background-color": state.hovered
                      ? "#334155"
                      : "transparent",
                    color: state.hovered ? "#f8fafc" : "#94a3b8",
                    opacity: selected() || state.hovered ? 1 : 0.55,
                  })}
                  onClick={(event: { stopPropagation(): void }) => {
                    event.stopPropagation();
                    closeTab(tab.id);
                  }}
                >
                  <Text>×</Text>
                </Button>
              </View>
            );
          }}
        </For>
        <Button
          unstyled
          variant="ghost"
          class="h-8 w-8 flex items-center justify-center rounded-md text-slate-400"
          style={(state) => ({
            padding: 0,
            "border-width": 0,
            "background-color": state.hovered ? "#334155" : "transparent",
          })}
          onClick={addTab}
        >
          <Text>+</Text>
        </Button>
      </div>
      <div class="min-h-0 flex-1 p-3">
        <For each={tabs.tabs()}>
          {(tab) => (
            <Terminal
              ref={(node) => terminals.set(tab.id, node)}
              class="w-full h-full overflow-hidden rounded-lg bg-slate-950 text-slate-200"
              style={{ display: tabs.activeKey() === tab.id ? "flex" : "none" }}
              inheritTheme
              fontFamily="Hack Nerd Font Mono"
              fontSize="14px"
              lineHeight="20px"
            />
          )}
        </For>
      </div>
    </View>
  );
}

mount(() => <App />);
