import { renderComponent } from "@wabou/test/component";
import { View } from "@wabou/ui";
import { AppCommandPalette } from "../../apps/pi-agent/ui/app-command-palette";
import { expect, test, vi } from "vitest";

test("runs a command and closes the application palette", async () => {
  const run = vi.fn();
  const close = vi.fn();
  const screen = renderComponent(
    () => (
      <View>
        <AppCommandPalette
          open
          label="Command palette"
          placeholder="Search commands"
          emptyText="No commands found."
          close={close}
          items={[
            {
              id: "new-session",
              label: "New session",
              description: "Start a clean conversation",
              shortcut: "Ctrl N",
              onSelect: run,
            },
          ]}
        />
      </View>
    ),
    { clock: "fake" },
  );
  await screen.advanceTime(16);

  screen.getByRole("textbox", { name: "Command palette" }).press("Enter");
  expect(run).toHaveBeenCalledOnce();
  expect(close).toHaveBeenCalledOnce();
});

test("keeps disabled application commands visible but inert", async () => {
  const run = vi.fn();
  const screen = renderComponent(
    () => (
      <View>
        <AppCommandPalette
          open
          label="Command palette"
          placeholder="Search commands"
          emptyText="No commands found."
          close={() => undefined}
          items={[
            {
              id: "changes",
              label: "Code changes",
              disabled: true,
              onSelect: run,
            },
          ]}
        />
      </View>
    ),
    { clock: "fake" },
  );
  await screen.advanceTime(16);

  const option = screen.getByRole("option", { name: "Code changes" });
  expect(option.disabled).toBe(true);
  expect(() => option.click()).toThrow("cannot click disabled component");
  expect(run).not.toHaveBeenCalled();
});
