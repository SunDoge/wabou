import { Button } from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import { createSignal } from "solid-js";
import { TaskPage } from "../shared";

export function CounterTask() {
  const [count, setCount] = createSignal(0);
  return (
    <TaskPage
      number={1}
      title="Counter"
      summary="The minimal test of state, event dispatch and native text updates."
    >
      <View class="flex-1 flex items-center justify-center gap-4">
        <Text
          role="status"
          aria-label="Counter value"
          class="w-24 h-12 flex items-center justify-center rounded-lg border border-subtle bg-surface-muted text-xl font-mono text-primary"
        >
          {count()}
        </Text>
        <Button
          aria-label="Increment counter"
          onClick={() => setCount((n) => n + 1)}
        >
          Count
        </Button>
        <Button
          aria-label="Reset counter"
          variant="ghost"
          onClick={() => setCount(0)}
        >
          Reset
        </Button>
      </View>
    </TaskPage>
  );
}
