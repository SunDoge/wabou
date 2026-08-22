import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  Button,
  Column,
  currentWindow,
  mount,
  px,
  Row,
  Text,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";

interface Point {
  x: number;
  y: number;
}

function App() {
  const window = currentWindow();
  const [start, setStart] = createSignal<Point | null>(null);
  const [end, setEnd] = createSignal<Point | null>(null);

  const selection = () => {
    const a = start();
    const b = end();
    if (!a || !b) return null;
    return {
      left: Math.min(a.x, b.x),
      top: Math.min(a.y, b.y),
      width: Math.abs(a.x - b.x),
      height: Math.abs(a.y - b.y),
    };
  };

  return (
    <View
      class="relative w-full h-full bg-transparent overflow-hidden"
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        const point = { x: event.clientX, y: event.clientY };
        setStart(point);
        setEnd(point);
      }}
      onPointerMove={(event) => {
        if (event.buttons !== 1 || !start()) return;
        setEnd({ x: event.clientX, y: event.clientY });
      }}
      onPointerUp={(event) => {
        if (!start()) return;
        setEnd({ x: event.clientX, y: event.clientY });
      }}
    >
      <View class="absolute inset-0 bg-black opacity-20 pointer-events-none" />
      <Column class="absolute left-6 top-6 w-96 gap-2 rounded-xl border border-subtle bg-surface p-4 shadow-lg">
        <Row class="items-center gap-3">
          <Column class="min-w-0 flex-1 gap-1">
            <Text class="text-base font-semibold">Transparent window lab</Text>
            <Text class="text-sm text-muted">
              Drag anywhere to verify native input and selection geometry.
            </Text>
          </Column>
          <Button
            variant="outline"
            size="sm"
            onClick={(event) => {
              event.stopPropagation();
              window.close();
            }}
          >
            Close
          </Button>
        </Row>
        <Text class="text-xs text-muted">
          The desktop should remain visible through the dimmed surface.
        </Text>
      </Column>

      {selection() && (
        <View
          class="absolute border-2 border-accent bg-transparent pointer-events-none"
          style={{
            left: px(selection()!.left),
            top: px(selection()!.top),
            width: px(selection()!.width),
            height: px(selection()!.height),
          }}
        />
      )}
    </View>
  );
}

mount(() => <App />);
