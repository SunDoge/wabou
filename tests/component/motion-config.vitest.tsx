import { renderComponent } from "@wabou/test/component";
import { MotionConfigProvider, Text, useReducedMotion, View } from "@wabou/ui";
import { expect, test } from "vitest";

function MotionStatus(props: { name: string }) {
  const reducedMotion = useReducedMotion();
  return (
    <Text role="status" aria-label={props.name}>
      {reducedMotion() ? "reduced" : "full"}
    </Text>
  );
}

test("motion policy inherits and supports a deliberate nested override", () => {
  const screen = renderComponent(() => (
    <MotionConfigProvider reducedMotion>
      <View>
        <MotionStatus name="Inherited motion" />
        <MotionConfigProvider reducedMotion={false}>
          <MotionStatus name="Overridden motion" />
        </MotionConfigProvider>
      </View>
    </MotionConfigProvider>
  ));

  expect(screen.getByRole("status", { name: "Inherited motion" }).text).toBe(
    "reduced",
  );
  expect(screen.getByRole("status", { name: "Overridden motion" }).text).toBe(
    "full",
  );
});
