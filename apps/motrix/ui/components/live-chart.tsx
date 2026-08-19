import { Path, View } from "@wabou/ui";
import { createMemo } from "solid-js";
import { smoothPath } from "../path";

export function LiveChart(props: { values: readonly number[] }) {
  const source = createMemo(() => {
    const maximum = Math.max(1, ...props.values);
    const values = props.values.map((value) => 0.12 + (value / maximum) * 0.76);
    return smoothPath(values, 250, 84).build({
      stroke: 0x38bdf8ff,
      strokeWidth: 2.5,
      lineCap: "round",
      lineJoin: "round",
    });
  });
  return (
    <View class="relative h-24 overflow-hidden rounded-lg bg-control">
      <Path class="absolute inset-0 w-full h-full" source={source()} />
    </View>
  );
}
