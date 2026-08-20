import { renderComponent } from "@wabou/test/component";
import {
  Button,
  Popover,
  PopoverDescription,
  PopoverFooter,
  PopoverHeader,
  PopoverTitle,
  Text,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("opens a styled dialog surface and dismisses with Escape", () => {
  const screen = renderComponent(() => (
    <Popover
      aria-label="Formatting options"
      trigger={(trigger) => <Button {...trigger}>Format</Button>}
    >
      <PopoverHeader>
        <PopoverTitle>Formatting</PopoverTitle>
        <PopoverDescription>Choose how text is displayed.</PopoverDescription>
      </PopoverHeader>
      <PopoverFooter>
        <Button>Apply</Button>
      </PopoverFooter>
    </Popover>
  ));
  const trigger = screen.getByRole("button", { name: "Format" });

  trigger.click();
  const popover = screen.getByRole("dialog", { name: "Formatting options" });
  expect(popover.className).toContain("bg-surface");
  expect(popover.className).toContain("rounded-lg");
  expect(screen.getByRole("heading", { name: "Formatting" })).not.toBeNull();
  popover.press("Escape");
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(trigger.attribute("aria-expanded")).toBe("false");
});

test("supports application-owned open state", () => {
  const Controlled = () => {
    const [open, setOpen] = createSignal(false);
    return (
      <>
        <Popover
          aria-label="Filters"
          open={open()}
          onOpenChange={setOpen}
          trigger={(trigger) => <Button {...trigger}>Filters</Button>}
        >
          <Text>Filter content</Text>
        </Popover>
        <Text role="status" aria-label={open() ? "Open" : "Closed"}>
          {open() ? "Open" : "Closed"}
        </Text>
      </>
    );
  };
  const screen = renderComponent(Controlled);

  screen.getByRole("button", { name: "Filters" }).click();
  expect(screen.getByRole("status", { name: "Open" }).text).toBe("Open");
  screen.getByRole("dialog", { name: "Filters" }).press("Escape");
  expect(screen.getByRole("status", { name: "Closed" }).text).toBe("Closed");
});
