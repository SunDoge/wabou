import { renderComponent } from "@wabou/test/component";
import {
  defineComponentFixtures,
  type LayoutFixtureEntry,
} from "@wabou/test/layout/fixtures";
import { MotionConfigProvider, Text, useReducedMotion } from "@wabou/ui";
import { createContext, useContext } from "solid-js";
import { expect, test } from "vitest";

const FixtureContext = createContext("outside");

function renderFixture(entry: LayoutFixtureEntry) {
  return typeof entry === "function" ? entry() : entry.render();
}

test("component fixture wrappers establish context before fixture creation", () => {
  const fixtures = defineComponentFixtures(
    {
      contextual: () => <Text role="status">{useContext(FixtureContext)}</Text>,
    },
    {
      wrap: (render) => (
        <FixtureContext value="inside">{render()}</FixtureContext>
      ),
    },
  );

  const screen = renderComponent(() => renderFixture(fixtures.contextual!));
  expect(screen.getByRole("status").text).toBe("inside");
});

test("component fixture wrappers apply motion policy before fixture creation", () => {
  function MotionProbe() {
    const reduced = useReducedMotion();
    return <Text role="status">{String(reduced())}</Text>;
  }
  const fixtures = defineComponentFixtures(
    { motion: MotionProbe },
    {
      wrap: (render) => (
        <MotionConfigProvider reducedMotion>{render()}</MotionConfigProvider>
      ),
    },
  );

  const screen = renderComponent(() => renderFixture(fixtures.motion!));
  expect(screen.getByRole("status").text).toBe("true");
});
