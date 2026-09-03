import { renderComponent } from "@wabou/test/component";
import { Button, Tooltip } from "@wabou/ui";
import { expect, test } from "vitest";

const renderTooltip = (openDelay = 500, fakeClock = false) =>
  renderComponent(
    () => (
      <Tooltip
        openDelay={openDelay}
        trigger={(trigger) => (
          <Button aria-label="Help" {...trigger}>
            Help
          </Button>
        )}
      >
        Explains this action
      </Tooltip>
    ),
    { clock: fakeClock ? "fake" : "real" },
  );

test("opens on focus and closes with Escape", () => {
  const screen = renderTooltip();
  const trigger = screen.getByRole("button", { name: "Help" });

  expect(screen.queryByRole("tooltip")).toBeNull();
  trigger.focus();
  expect(
    screen.getByRole("tooltip", { name: "Explains this action" }).text,
  ).toBe("Explains this action");
  expect(
    screen.getByRole("tooltip").closestByRole("presentation")?.className,
  ).toContain("rounded-lg");

  trigger.press("Escape");
  expect(screen.queryByRole("tooltip")).toBeNull();
});

test("honors hover delay and cancels a pending open", async () => {
  const screen = renderTooltip(400, true);
  const trigger = screen.getByRole("button", { name: "Help" });

  trigger.hover();
  await screen.advanceTime(399);
  expect(screen.queryByRole("tooltip")).toBeNull();
  trigger.unhover();
  await screen.advanceTime(1_000);
  expect(screen.queryByRole("tooltip")).toBeNull();

  trigger.hover();
  await screen.advanceTime(400);
  expect(screen.getByRole("tooltip").text).toBe("Explains this action");
});

test("owns optional shortcut presentation without changing tooltip semantics", () => {
  const screen = renderComponent(() => (
    <Tooltip
      defaultOpen
      shortcut="Ctrl K"
      trigger={(trigger) => (
        <Button aria-label="Commands" {...trigger}>
          Commands
        </Button>
      )}
    >
      Open commands
    </Tooltip>
  ));

  const tooltip = screen.getByRole("tooltip", { name: "Open commands" });
  const shortcut = screen.getByRole("label", { name: "Ctrl K shortcut" });
  expect(tooltip.parent?.className).toContain("gap-3");
  expect(shortcut.className).toContain("text-xs");
  expect(shortcut.text).toBe("Ctrl K");
});
