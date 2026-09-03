import {
  assertInOverlayPlane,
  createTestHost,
  renderComponent,
} from "@wabou/test/component";
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
import { popoverNativeTransition } from "../../packages/ui/src/primitives/popover";

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
  assertInOverlayPlane(popover, "floating");
  expect(popover.className).toContain("bg-surface");
  expect(popover.className).toContain("rounded-lg");
  expect(screen.getByRole("heading", { name: "Formatting" })).not.toBeNull();
  popover.press("Escape");
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(trigger.expanded).toBe(false);
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

test("positions on the first frame after opening", async () => {
  const fixture = createTestHost();
  const screen = renderComponent(
    () => (
      <Popover
        aria-label="Fast actions"
        trigger={(trigger) => <Button {...trigger}>Actions</Button>}
      >
        <Text>Action list</Text>
      </Popover>
    ),
    { clock: "fake", host: fixture.host },
  );

  screen.getByRole("button", { name: "Actions" }).click();
  expect(fixture.callsTo("layout.snapshot")).toHaveLength(0);
  await screen.advanceTime(32);
  screen.flush();
  expect(fixture.callsTo("layout.snapshot")).toHaveLength(1);
});

test("compiles popup enter and exit intent into finite GPUI transitions", () => {
  expect(
    popoverNativeTransition({
      generation: 3,
      duration: 0.2,
      ease: "easeOut",
      fromScale: 0.96,
      entering: true,
    }),
  ).toMatchObject({
    generation: 3,
    duration: 0.2,
    easing: "easeOut",
    fromTransform: [0.96, 0, 0, 0.96, 0, 0],
    toTransform: [1, 0, 0, 1, 0, 0],
    fromOpacity: 0,
    toOpacity: 1,
  });
  expect(
    popoverNativeTransition({
      generation: 4,
      duration: 0.2,
      fromScale: 0.96,
      entering: false,
    }),
  ).toMatchObject({
    generation: 4,
    fromTransform: [1, 0, 0, 1, 0, 0],
    toTransform: [0.96, 0, 0, 0.96, 0, 0],
    fromOpacity: 1,
    toOpacity: 0,
  });
});

test("passthrough outside dismissal preserves the underlying gesture", () => {
  let activations = 0;
  const screen = renderComponent(() => (
    <>
      <Popover
        aria-label="Quick actions"
        outsidePointerStrategy="passthrough"
        trigger={(trigger) => <Button {...trigger}>Actions</Button>}
      >
        <Text>Action list</Text>
      </Popover>
      <Button aria-label="Underlying action" onClick={() => activations++} />
    </>
  ));

  screen.getByRole("button", { name: "Actions" }).click();
  expect(screen.getByRole("dialog", { name: "Quick actions" })).not.toBeNull();
  screen.getByRole("button", { name: "Underlying action" }).click();

  expect(screen.queryByRole("dialog", { name: "Quick actions" })).toBeNull();
  expect(activations).toBe(1);
});
