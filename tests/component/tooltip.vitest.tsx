import { renderComponent } from "@wabou/test/component";
import { Button, Tooltip } from "@wabou/ui";
import { afterEach, expect, test, vi } from "vitest";

afterEach(() => vi.useRealTimers());

const renderTooltip = (openDelay = 500) =>
  renderComponent(() => (
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
  ));

test("opens on focus and closes with Escape", () => {
  const screen = renderTooltip();
  const trigger = screen.getByRole("button", { name: "Help" });

  expect(screen.queryByRole("tooltip")).toBeNull();
  trigger.focus();
  expect(
    screen.getByRole("tooltip", { name: "Explains this action" }).text,
  ).toBe("Explains this action");

  trigger.press("Escape");
  expect(screen.queryByRole("tooltip")).toBeNull();
});

test("honors hover delay and cancels a pending open", () => {
  vi.useFakeTimers();
  const screen = renderTooltip(400);
  const trigger = screen.getByRole("button", { name: "Help" });

  trigger.hover();
  vi.advanceTimersByTime(399);
  screen.flush();
  expect(screen.queryByRole("tooltip")).toBeNull();
  trigger.unhover();
  vi.advanceTimersByTime(1_000);
  screen.flush();
  expect(screen.queryByRole("tooltip")).toBeNull();

  trigger.hover();
  vi.advanceTimersByTime(400);
  screen.flush();
  expect(screen.getByRole("tooltip").text).toBe("Explains this action");
});
