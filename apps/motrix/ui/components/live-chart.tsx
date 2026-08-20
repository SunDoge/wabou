import {
  animate,
  createMeasuredSize,
  Path,
  PathBuilder,
  View,
} from "@wabou/ui";
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  untrack,
} from "solid-js";

export function LiveChart(props: {
  values: readonly number[];
  color?: "download" | "upload";
  compact?: boolean;
  grid?: boolean;
}) {
  const measured = createMeasuredSize();
  const width = () => Math.max(1, measured.width());
  const height = () =>
    Math.max(1, measured.height() || (props.compact ? 56 : 96));
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
    return pathForValues(normalized(), width(), height()).build({
      stroke: props.color === "upload" ? 0xa855f7ff : 0x2f81f7ff,
      strokeWidth: 2.25,
      lineCap: "round",
      lineJoin: "round",
    });
  });
  const area = createMemo(() => {
    const path = pathForValues(normalized(), width(), height());
    if (path.hasCurrentPoint)
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
      ref={measured.ref}
      role="img"
      aria-label={`${props.color === "upload" ? "Upload" : "Download"} throughput chart`}
      class="relative overflow-hidden rounded-lg"
      classList={{ "h-14": props.compact, "h-24": !props.compact }}
    >
      {props.grid !== false && (
        <Path class="absolute inset-0 w-full h-full" source={grid()} />
      )}
      <Path class="absolute inset-0 w-full h-full" source={area()} />
      <Path class="absolute inset-0 w-full h-full" source={line()} />
    </View>
  );
}

function pathForValues(
  values: readonly number[],
  width: number,
  height: number,
) {
  const points = values.map((value, index) => ({
    x: values.length <= 1 ? 0 : (index / (values.length - 1)) * width,
    y: height - value * height,
  }));
  return new PathBuilder().splineThrough(points);
}
