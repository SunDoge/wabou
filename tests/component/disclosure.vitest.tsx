import { type ComponentScreen, renderComponent } from "@wabou/test/component";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
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

test("collapsible forwards root, trigger and content contracts", () => {
  const screen = renderComponent(() => (
    <Collapsible
      defaultOpen
      reducedMotion
      role="group"
      aria-label="Advanced settings"
    >
      <CollapsibleTrigger
        aria-label="Toggle advanced settings"
        onClick={(event) => event.preventDefault()}
      >
        Advanced
      </CollapsibleTrigger>
      <CollapsibleContent role="region" aria-label="Advanced options">
        <Text>Options</Text>
      </CollapsibleContent>
    </Collapsible>
  ));

  expect(screen.getByRole("group", { name: "Advanced settings" })).toBeTruthy();
  const trigger = screen.getByRole("button", {
    name: "Toggle advanced settings",
  });
  expect(trigger.className).toContain("min-h-7");
  expect(trigger.expanded).toBe(true);
  expect(screen.getByRole("region", { name: "Advanced options" }).text).toBe(
    "Options",
  );
  trigger.click();
  expect(trigger.expanded).toBe(true);
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

test("accordion forwards its anatomy and roves focus across enabled triggers", () => {
  const screen = renderComponent(() => (
    <Accordion
      defaultValue="one"
      collapsible
      reducedMotion
      role="group"
      aria-label="Questions"
    >
      <AccordionItem value="one" role="group" aria-label="First item">
        <AccordionTrigger aria-label="First question">First</AccordionTrigger>
        <AccordionContent role="region" aria-label="First answer">
          <Text>Answer one</Text>
        </AccordionContent>
      </AccordionItem>
      <AccordionItem value="two" disabled>
        <AccordionTrigger aria-label="Second question">Second</AccordionTrigger>
      </AccordionItem>
      <AccordionItem value="three">
        <AccordionTrigger
          aria-label="Third question"
          onClick={(event) => event.preventDefault()}
        >
          Third
        </AccordionTrigger>
      </AccordionItem>
    </Accordion>
  ));

  expect(screen.getByRole("group", { name: "Questions" })).toBeTruthy();
  expect(screen.getByRole("group", { name: "First item" })).toBeTruthy();
  expect(screen.getByRole("region", { name: "First answer" }).text).toBe(
    "Answer one",
  );

  const first = screen.getByRole("button", { name: "First question" });
  const third = screen.getByRole("button", { name: "Third question" });
  expect(first.className).toContain("py-4");
  expect(first.className).toContain("w-full");
  first.focus();
  first.press("ArrowDown");
  expect(third.focused).toBe(true);
  third.press("Home");
  expect(first.focused).toBe(true);

  third.click();
  expect(third.expanded).toBe(false);
});
