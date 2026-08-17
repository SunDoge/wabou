import {
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import { createSignal, For, Show } from "solid-js";
import { type LocalPointerEvent, TaskPage } from "../shared";

interface Circle {
  id: number;
  x: number;
  y: number;
  diameter: number;
}

export function CircleDrawerTask() {
  const [circles, setCircles] = createSignal<Circle[]>([]);
  const [history, setHistory] = createSignal<Circle[][]>([]);
  const [future, setFuture] = createSignal<Circle[][]>([]);
  const [selected, setSelected] = createSignal<number | null>(null);
  const [editing, setEditing] = createSignal(false);
  let nextId = 1;
  const commit = (next: Circle[]) => {
    setHistory((items) => [...items, circles()]);
    setCircles(next);
    setFuture([]);
  };
  const add = (event: LocalPointerEvent) => {
    if (event.button !== 0) return;
    commit([
      ...circles(),
      { id: nextId++, x: event.offsetX, y: event.offsetY, diameter: 52 },
    ]);
  };
  const undo = () => {
    const items = history();
    if (items.length === 0) return;
    setFuture((next) => [circles(), ...next]);
    const previous = items.at(-1);
    if (!previous) return;
    setCircles(previous);
    setHistory(items.slice(0, -1));
    setSelected(null);
  };
  const redo = () => {
    const items = future();
    if (items.length === 0) return;
    setHistory((previous) => [...previous, circles()]);
    const next = items[0];
    if (!next) return;
    setCircles(next);
    setFuture(items.slice(1));
    setSelected(null);
  };
  const resize = (delta: number) => {
    const id = selected();
    if (id === null) return;
    const next = circles().map((circle) =>
      circle.id === id
        ? {
            ...circle,
            diameter: Math.max(20, Math.min(160, circle.diameter + delta)),
          }
        : circle,
    );
    commit(next);
  };
  const selectedCircle = () =>
    circles().find((circle) => circle.id === selected());
  return (
    <TaskPage
      number={6}
      title="Circle Drawer"
      summary="Local pointer coordinates drive retained circles with selection, resizing and undo/redo."
    >
      <View class="flex flex-col gap-4">
        <View class="flex items-center gap-2">
          <Button
            aria-label="Undo circle action"
            variant="secondary"
            disabled={history().length === 0}
            onClick={undo}
          >
            Undo
          </Button>
          <Button
            aria-label="Redo circle action"
            variant="secondary"
            disabled={future().length === 0}
            onClick={redo}
          >
            Redo
          </Button>
          <Button
            aria-label="Adjust selected circle"
            variant="outline"
            disabled={selected() === null}
            onClick={() => setEditing(true)}
          >
            Adjust selected
          </Button>
          <Text class="ml-auto text-xs text-muted">
            Click to add · right-click a circle to resize
          </Text>
        </View>
        <View
          role="button"
          aria-label="Circle canvas"
          tabIndex={0}
          class="w-full h-96 relative overflow-hidden rounded-xl border border-strong bg-surface-muted"
          onClick={add}
        >
          <For each={circles()}>
            {(circle) => (
              <View
                role="button"
                aria-label={`Circle ${circle.id}`}
                class={
                  selected() === circle.id
                    ? "absolute rounded-full border-2 border-focus bg-selected"
                    : "absolute rounded-full border-2 border-accent bg-transparent"
                }
                style={{
                  left: `${circle.x - circle.diameter / 2}px`,
                  top: `${circle.y - circle.diameter / 2}px`,
                  width: `${circle.diameter}px`,
                  height: `${circle.diameter}px`,
                }}
                onClick={(event: LocalPointerEvent) => {
                  event.stopPropagation();
                  setSelected(circle.id);
                }}
                onContextMenu={(event: LocalPointerEvent) => {
                  event.preventDefault();
                  event.stopPropagation();
                  setSelected(circle.id);
                  setEditing(true);
                }}
              />
            )}
          </For>
          <Show when={circles().length === 0}>
            <Text class="absolute left-0 top-0 w-full h-full flex items-center justify-center text-sm text-muted">
              Click anywhere to create a circle
            </Text>
          </Show>
        </View>
      </View>
      <Dialog
        aria-label="Adjust circle"
        open={editing()}
        onOpenChange={setEditing}
      >
        {(controls) => (
          <>
            <DialogHeader>
              <DialogTitle>Adjust circle</DialogTitle>
              <DialogDescription>
                Change the diameter of the selected circle.
              </DialogDescription>
            </DialogHeader>
            <View class="flex items-center justify-center gap-4">
              <Button
                aria-label="Shrink circle"
                variant="secondary"
                onClick={() => resize(-10)}
              >
                Smaller
              </Button>
              <Text
                role="status"
                aria-label="Circle diameter"
                class="w-24 text-center font-mono text-primary"
              >
                {selectedCircle()?.diameter ?? 0} px
              </Text>
              <Button
                aria-label="Grow circle"
                variant="secondary"
                onClick={() => resize(10)}
              >
                Larger
              </Button>
            </View>
            <DialogFooter>
              <Button onClick={controls.close}>Done</Button>
            </DialogFooter>
          </>
        )}
      </Dialog>
    </TaskPage>
  );
}
