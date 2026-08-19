import { Button, Progress, Slider, Text, View } from "@wabou/ui";
import { createSignal, onCleanup } from "solid-js";
import { TaskPage } from "../shared";

export function TimerTask() {
  const [duration, setDuration] = createSignal(10);
  const [elapsed, setElapsed] = createSignal(0);
  let started = performance.now();
  const timer = setInterval(() => {
    const next = Math.min(duration(), (performance.now() - started) / 1000);
    setElapsed(next);
  }, 32);
  onCleanup(() => clearInterval(timer));
  const reset = () => {
    started = performance.now();
    setElapsed(0);
  };
  const setDurationValue = (next: number) => {
    setDuration(next);
    started = performance.now() - Math.min(elapsed(), next) * 1000;
  };
  return (
    <TaskPage
      number={4}
      title="Timer"
      summary="A native frame-friendly clock combines elapsed time, progress and an accessible duration control."
    >
      <View class="w-96 mx-auto flex-1 flex flex-col justify-center gap-5">
        <View class="flex items-center justify-between">
          <Text class="text-sm text-secondary">Elapsed time</Text>
          <Text
            role="status"
            aria-label="Elapsed time"
            class="font-mono text-lg text-primary"
          >
            {elapsed().toFixed(1)} s
          </Text>
        </View>
        <Progress value={(elapsed() / duration()) * 100} class="h-3" />
        <View class="flex flex-col gap-2">
          <View class="flex items-center justify-between">
            <Text class="text-sm text-secondary">Duration</Text>
            <Text class="font-mono text-sm text-primary">{duration()} s</Text>
          </View>
          <Slider
            label="Timer duration"
            class="w-80"
            min={1}
            max={30}
            step={1}
            value={duration()}
            valueText={(value) => `${value} seconds`}
            onValueChange={setDurationValue}
          />
        </View>
        <Button aria-label="Reset timer" onClick={reset}>
          Reset
        </Button>
      </View>
    </TaskPage>
  );
}
