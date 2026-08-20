import { type ComponentScreen, renderComponent } from "@wabou/test/component";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  Text,
  View,
} from "@wabou/ui";
import { expect, test } from "vitest";

const renderDisclosure = (reducedMotion = false) =>
  renderComponent(
    () => (
      <View role="group" aria-label="Disclosure fixture">
        <Collapsible reducedMotion={reducedMotion}>
          <CollapsibleTrigger>Details</CollapsibleTrigger>
          <CollapsibleContent>
            <Text role="status">Expanded content</Text>
          </CollapsibleContent>
        </Collapsible>
      </View>
    ),
    { clock: "fake" },
  );

const measuredContent = (screen: ComponentScreen) => {
  const fixture = screen.getByRole("group", { name: "Disclosure fixture" });
  const root = fixture.children[0];
  const viewport = root?.children[1];
  const measured = viewport?.children[0];
  if (!measured)
    throw new Error("mounted disclosure measurement node is missing");
  return measured;
};

test("retains disclosure content until its exit animation completes", async () => {
  const screen = renderDisclosure();
  const trigger = screen.getByRole("button", { name: "Details" });

  expect(trigger.expanded).toBe(false);
  expect(screen.queryByRole("status")).toBeNull();
  trigger.click();
  measuredContent(screen).resize({ width: 240, height: 80 });
  await screen.advanceTime(220);
  expect(trigger.expanded).toBe(true);
  expect(screen.getByRole("status").text).toBe("Expanded content");

  trigger.click();
  expect(trigger.expanded).toBe(false);
  expect(screen.getByRole("status").text).toBe("Expanded content");
  await screen.advanceTime(100);
  expect(screen.getByRole("status").text).toBe("Expanded content");
  await screen.advanceTime(120);
  expect(screen.queryByRole("status")).toBeNull();
});

test("removes closed disclosure content immediately under reduced motion", () => {
  const screen = renderDisclosure(true);
  const trigger = screen.getByRole("button", { name: "Details" });

  trigger.click();
  measuredContent(screen).resize({ width: 240, height: 80 });
  expect(screen.getByRole("status").text).toBe("Expanded content");
  trigger.click();
  expect(trigger.expanded).toBe(false);
  expect(screen.queryByRole("status")).toBeNull();
});

test("retargets an interrupted disclosure exit without remounting content", async () => {
  const screen = renderDisclosure();
  const trigger = screen.getByRole("button", { name: "Details" });

  trigger.click();
  measuredContent(screen).resize({ width: 240, height: 80 });
  await screen.advanceTime(220);
  expect(screen.getByRole("status").text).toBe("Expanded content");

  trigger.click();
  await screen.advanceTime(100);
  trigger.click();
  expect(trigger.expanded).toBe(true);
  expect(screen.getByRole("status").text).toBe("Expanded content");
  await screen.advanceTime(220);
  expect(screen.getByRole("status").text).toBe("Expanded content");
});
