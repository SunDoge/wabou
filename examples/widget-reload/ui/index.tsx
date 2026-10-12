import {
  bindCapability,
  Button,
  type Host,
  mount,
  type NativeCapability,
  NativeWidget,
  Text,
  useHost,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import "virtual:wabou-stylesheet";
interface CounterCapability extends NativeCapability {
  increment(): Promise<number>;
}
mount(() => {
  const counter = bindCapability(
    useHost<Host & { counter: CounterCapability }>().counter,
    { name: "counter", version: 1 },
  );
  const [value, setValue] = createSignal<number | undefined>();
  const [error, setError] = createSignal("");
  return (
    <View class="p-6 gap-4">
      <Text>
        Click either counter, then change STEP in native/src/lib.rs and save.
      </Text>
      <NativeWidget tag="reload-counter" class="w-full h-20" />
      <Button
        onClick={async () => {
          try {
            setValue(await counter.increment());
            setError("");
          } catch (cause) {
            setError(String(cause));
          }
        }}
      >
        Call Rust capability
      </Button>
      <Text>
        {value() === undefined
          ? "Capability counter has not been called yet."
          : `Capability count: ${value()}`}
      </Text>
      <Text>{error()}</Text>
      <Text>
        Both implementations share one library; their state stays independent.
      </Text>
    </View>
  );
});
