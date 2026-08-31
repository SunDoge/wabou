import {
  Badge,
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  Button as PrimitiveButton,
  ScrollArea,
  Text,
  useNavigate,
  useParams,
  View,
} from "@wabou/ui";
import { createSignal, For as ForValue, Match, Switch } from "solid-js";
import { CellsTask } from "./tasks/cells";
import { CircleDrawerTask } from "./tasks/circle-drawer";
import { CounterTask } from "./tasks/counter";
import { CrudTask } from "./tasks/crud";
import { FlightBookerTask } from "./tasks/flight-booker";
import { TemperatureTask } from "./tasks/temperature";
import { TimerTask } from "./tasks/timer";

type TaskId =
  | "counter"
  | "temperature"
  | "flight"
  | "timer"
  | "crud"
  | "circles"
  | "cells";
const tasks: Array<{
  id: TaskId;
  number: number;
  label: string;
  note: string;
}> = [
  { id: "counter", number: 1, label: "Counter", note: "State" },
  { id: "temperature", number: 2, label: "Temperature", note: "Binding" },
  { id: "flight", number: 3, label: "Flight Booker", note: "Validation" },
  { id: "timer", number: 4, label: "Timer", note: "Time" },
  { id: "crud", number: 5, label: "CRUD", note: "Collections" },
  { id: "circles", number: 6, label: "Circle Drawer", note: "Undo" },
  { id: "cells", number: 7, label: "Cells", note: "Dependencies" },
];

export function App() {
  const params = useParams<{ task?: string }>();
  const navigate = useNavigate();
  const task = (): TaskId =>
    tasks.some((item) => item.id === params().task)
      ? (params().task as TaskId)
      : "counter";
  const [theme, setTheme] = createSignal<"dark" | "light">("dark");
  const dark = () => theme() === "dark";
  return (
    <ColorThemeProvider theme={theme()} transition={false}>
      <ComponentsProvider theme={theme()}>
        <View class="w-full h-full flex overflow-hidden bg-canvas text-primary font-sans">
          <View class="w-64 h-full flex-none flex flex-col border-r border-subtle bg-surface">
            <View class="h-20 flex-none px-5 flex items-center gap-3 border-b border-subtle">
              <View class="w-10 h-10 flex items-center justify-center rounded-xl bg-accent">
                <Text class="font-bold text-on-accent">7</Text>
              </View>
              <View class="min-w-0 flex flex-col gap-1">
                <Text class="font-semibold text-primary">7GUIs</Text>
                <Text class="text-xs text-muted">Wabou benchmark</Text>
              </View>
            </View>
            <ScrollArea class="flex-1" contentClass="p-3 gap-2">
              <ForValue each={tasks}>
                {(item) => (
                  <PrimitiveButton
                    unstyled
                    aria-label={item.label}
                    selected={task() === item.id}
                    class="w-full h-14 px-3 rounded-lg gap-3"
                    style={(state) => ({
                      "justify-content": "flex-start",
                      "background-color":
                        task() === item.id
                          ? dark()
                            ? "#15395d"
                            : "#e0f2fe"
                          : state.hovered
                            ? dark()
                              ? "#202b3b"
                              : "#edf2f7"
                            : "transparent",
                      color:
                        task() === item.id
                          ? dark()
                            ? "#e0f2fe"
                            : "#075985"
                          : dark()
                            ? "#c4cfdd"
                            : "#334155",
                    })}
                    onClick={() => void navigate({ to: `/${item.id}` })}
                  >
                    <Text class="w-7 h-7 flex-none flex items-center justify-center rounded-md bg-control font-mono text-xs">
                      {item.number}
                    </Text>
                    <View class="min-w-0 flex flex-col">
                      <Text class="text-sm font-medium">{item.label}</Text>
                      <Text class="text-xs text-muted">{item.note}</Text>
                    </View>
                  </PrimitiveButton>
                )}
              </ForValue>
            </ScrollArea>
            <View class="flex-none p-4 border-t border-subtle">
              <Text class="whitespace-normal text-xs text-muted">
                Seven small programs testing real GUI concerns.
              </Text>
            </View>
          </View>
          <View class="flex-1 min-w-0 h-full flex flex-col">
            <View class="h-16 flex-none px-6 flex items-center justify-between border-b border-subtle bg-surface">
              <View class="flex items-center gap-3">
                <Text class="text-sm font-semibold text-primary">
                  Wabou native UI
                </Text>
                <Badge variant="outline">Solid 2</Badge>
              </View>
              <View class="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label="Toggle theme"
                  onClick={() => setTheme(dark() ? "light" : "dark")}
                >
                  {dark() ? "Light theme" : "Dark theme"}
                </Button>
              </View>
            </View>
            <ScrollArea class="flex-1 min-h-0" contentClass="px-8 py-8">
              <Switch>
                <Match when={task() === "counter"}>
                  <CounterTask />
                </Match>
                <Match when={task() === "temperature"}>
                  <TemperatureTask />
                </Match>
                <Match when={task() === "flight"}>
                  <FlightBookerTask />
                </Match>
                <Match when={task() === "timer"}>
                  <TimerTask />
                </Match>
                <Match when={task() === "crud"}>
                  <CrudTask />
                </Match>
                <Match when={task() === "circles"}>
                  <CircleDrawerTask />
                </Match>
                <Match when={task() === "cells"}>
                  <CellsTask />
                </Match>
              </Switch>
            </ScrollArea>
          </View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
