import { animate, Path, PathBuilder, View } from "@wabou/ui";
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  untrack,
} from "solid-js";
import { smoothPath } from "../path";

export function LiveChart(props: {
  values: readonly number[];
  color?: "download" | "upload";
  compact?: boolean;
  width?: number;
}) {
  const width = () => props.width ?? 230;
  const height = () => (props.compact ? 56 : 96);
  const normalize = (values: readonly number[]) => {
    const maximum = Math.max(1, ...values);
    return values.map((value) => 0.14 + (value / maximum) * 0.72);
  };
  const [normalized, setNormalized] = createSignal(
    normalize(untrack(() => props.values)),
  );
  let controls: ReturnType<typeof animate> | undefined;

  createEffect(
    () => props.values,
    (values) => {
      const target = normalize(values);
      const current = untrack(normalized);
      const length = Math.max(current.length, target.length);
      const from = Array.from(
        { length },
        (_, index) => current[index] ?? current.at(-1) ?? 0.14,
      );
      const to = Array.from(
        { length },
        (_, index) => target[index] ?? target.at(-1) ?? 0.14,
      );

      controls?.stop();
      controls = animate(0, 1, {
        duration: 0.28,
        ease: "easeOut",
        onUpdate(progress) {
          setNormalized(
            from.map(
              (value, index) =>
                value + ((to[index] ?? value) - value) * progress,
            ),
          );
        },
        onComplete() {
          setNormalized(target);
          controls = undefined;
        },
      });
    },
  );
  onCleanup(() => controls?.stop());
  const line = createMemo(() => {
    return smoothPath(normalized(), width(), height()).build({
      stroke: props.color === "upload" ? 0xa855f7ff : 0x2f81f7ff,
      strokeWidth: 2.25,
      lineCap: "round",
      lineJoin: "round",
    });
  });
  const area = createMemo(() => {
    const path = smoothPath(normalized(), width(), height());
    path.lineTo(width(), height()).lineTo(0, height()).close();
    return path.build({
      fill: props.color === "upload" ? 0xa855f724 : 0x2f81f72c,
    });
  });
  const grid = createMemo(() => {
    const quarter = height() / 4;
    return new PathBuilder()
      .moveTo(0, quarter)
      .lineTo(width(), quarter)
      .moveTo(0, quarter * 2)
      .lineTo(width(), quarter * 2)
      .moveTo(0, quarter * 3)
      .lineTo(width(), quarter * 3)
      .build({ stroke: 0x94a3b826, strokeWidth: 1 });
  });
  return (
    <View
      class="relative overflow-hidden rounded-lg"
      classList={{ "h-14": props.compact, "h-24": !props.compact }}
    >
      <Path class="absolute inset-0 w-full h-full" source={grid()} />
      <Path class="absolute inset-0 w-full h-full" source={area()} />
      <Path class="absolute inset-0 w-full h-full" source={line()} />
    </View>
  );
}
