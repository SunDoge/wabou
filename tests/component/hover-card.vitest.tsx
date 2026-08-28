import { renderComponent } from "@wabou/test/component";
import { Button, HoverCard, Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const renderHoverCard = (
  options: {
    openDelay?: number;
    closeDelay?: number;
    disabled?: boolean;
    fakeClock?: boolean;
  } = {},
) =>
  renderComponent(
    () => (
      <HoverCard
        aria-label="Project preview"
        openDelay={options.openDelay}
        closeDelay={options.closeDelay}
        disabled={options.disabled}
        trigger={(trigger) => (
          <Button aria-label="Wabou project" {...trigger}>
            Wabou
          </Button>
        )}
      >
        <Text>Native Solid applications</Text>
      </HoverCard>
    ),
    { clock: options.fakeClock ? "fake" : "real" },
  );

test("keeps the card open while the pointer travels into its content", async () => {
  const screen = renderHoverCard({
    openDelay: 300,
    closeDelay: 200,
    fakeClock: true,
  });
  const trigger = screen.getByRole("button", { name: "Wabou project" });

  trigger.hover();
  await screen.advanceTime(300);
  const card = screen.getByRole("dialog", { name: "Project preview" });
  expect(card.text).toBe("Native Solid applications");
  expect(card.className).toContain("rounded-xl");

  trigger.unhover();
  card.hover();
  await screen.advanceTime(500);
  expect(screen.getByRole("dialog").text).toBe("Native Solid applications");

  card.unhover();
  await screen.advanceTime(200);
  expect(screen.getByRole("dialog").interactionBlocked).toBe(true);
  await screen.advanceTime(160);
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("opens immediately for keyboard focus and closes with Escape", () => {
  const screen = renderHoverCard();
  const trigger = screen.getByRole("button", { name: "Wabou project" });

  trigger.focus();
  expect(trigger.expanded).toBe(true);
  expect(
    screen.getByRole("dialog", { name: "Project preview" }),
  ).not.toBeNull();

  trigger.press("Escape");
  expect(trigger.expanded).toBe(false);
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("supports application-owned state and an explicit disabled policy", () => {
  const Controlled = () => {
    const [open, setOpen] = createSignal(false);
    return (
      <HoverCard
        aria-label="Controlled preview"
        open={open()}
        onOpenChange={setOpen}
        trigger={(trigger) => (
          <Button aria-label="Controlled trigger" {...trigger} />
        )}
      >
        Controlled content
      </HoverCard>
    );
  };
  const controlled = renderComponent(Controlled);
  controlled.getByRole("button", { name: "Controlled trigger" }).focus();
  expect(
    controlled.getByRole("dialog", { name: "Controlled preview" }),
  ).not.toBeNull();
  controlled.dispose();

  const disabled = renderHoverCard({ openDelay: 0, disabled: true });
  disabled.getByRole("button", { name: "Wabou project" }).hover();
  expect(disabled.queryByRole("dialog")).toBeNull();
});
