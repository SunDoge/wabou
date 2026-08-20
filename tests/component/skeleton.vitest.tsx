import { renderComponent } from "@wabou/test/component";
import { MotionConfigProvider, Skeleton, View } from "@wabou/ui";
import { expect, test } from "vitest";

test("sweeps with a transform after native measurement", async () => {
  const screen = renderComponent(
    () => (
      <View role="group" aria-label="Loading preview">
        <Skeleton />
      </View>
    ),
    { clock: "fake" },
  );
  const skeleton = screen.getByRole("group", {
    name: "Loading preview",
  }).children[0];
  const shimmer = skeleton.children[0];

  skeleton.resize({ width: 100, height: 16 });
  const initialOffset = shimmer.transform?.[4];
  expect(initialOffset).toBe(-40);
  expect(shimmer.style("opacity")).toBe("1");

  await screen.advanceTime(400);

  expect(shimmer.transform?.[4]).toBeGreaterThan(initialOffset ?? 0);
});

test("keeps a stable base under reduced motion", () => {
  const screen = renderComponent(() => (
    <MotionConfigProvider reducedMotion>
      <View role="group" aria-label="Loading preview">
        <Skeleton />
      </View>
    </MotionConfigProvider>
  ));
  const skeleton = screen.getByRole("group").children[0];
  const shimmer = skeleton.children[0];

  skeleton.resize({ width: 100, height: 16 });

  expect(shimmer.style("opacity")).toBe("0");
  expect(shimmer.transform?.[4]).toBeCloseTo(30);
});

test("supports an explicitly static placeholder", () => {
  const screen = renderComponent(() => (
    <View role="group" aria-label="Loading preview">
      <Skeleton animated={false} />
    </View>
  ));
  const shimmer = screen.getByRole("group").children[0].children[0];

  expect(shimmer.style("opacity")).toBe("0");
});
