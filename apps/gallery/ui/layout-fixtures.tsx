import { defineLayoutFixtures } from "@wabou/test/layout/fixtures";
import { Text, View } from "@wabou/ui";
import { createEffect, createSignal, onCleanup } from "solid-js";
import { galleryLayoutFixtures } from "./layout-fixture-pages";

let activeOwners = 0;

function TrackedFixture(props: { name: string; width: string }) {
  activeOwners++;
  onCleanup(() => activeOwners--);
  return (
    <View
      aria-label={props.name}
      class="flex flex-col gap-2 p-4"
      style={{ width: props.width }}
    >
      <Text aria-label={`${props.name} owner count`}>{activeOwners}</Text>
      <Text>{props.name}</Text>
    </View>
  );
}

function EffectFixture() {
  const [status, setStatus] = createSignal("pending");
  createEffect(
    () => "ready",
    (value) => {
      setStatus(value);
    },
  );
  return <Text aria-label="effect status">{status()}</Text>;
}

defineLayoutFixtures({
  narrow: () => <TrackedFixture name="narrow" width="120px" />,
  wide: () => <TrackedFixture name="wide" width="320px" />,
  "effect/synchronous": () => <EffectFixture />,
  ...galleryLayoutFixtures,
});
