import { renderComponent } from "@wabou/test/component";
import { Button } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import { SessionForkDialog } from "../../apps/pi-agent/ui/session-fork";

test("requires confirmation before rewinding a Pi session", () => {
  const confirm = vi.fn();
  const cancel = vi.fn();
  const App = () => {
    const [open, setOpen] = createSignal(false);
    return (
      <>
        <Button onClick={() => setOpen(true)}>Fork message</Button>
        <SessionForkDialog
          open={open()}
          checkpoint="available"
          confirm={confirm}
          cancel={() => {
            cancel();
            setOpen(false);
          }}
        />
      </>
    );
  };
  const screen = renderComponent(App);

  screen.getByRole("button", { name: "Fork message" }).click();

  const dialog = screen.getByRole("alertdialog", {
    name: "Fork from this message?",
  });
  expect(dialog.text).toContain("Git workspace returns to its state");
  screen.getByRole("button", { name: "Fork" }).click();
  expect(confirm).toHaveBeenCalledOnce();
});

test("does not fork until workspace checkpoint inspection completes", () => {
  const confirm = vi.fn();
  let complete!: () => void;
  const screen = renderComponent(() => {
    const [open, setOpen] = createSignal(false);
    const [checkpoint, setCheckpoint] = createSignal<"checking" | "available">(
      "checking",
    );
    complete = () => setCheckpoint("available");
    return (
      <>
        <Button onClick={() => setOpen(true)}>Fork message</Button>
        <SessionForkDialog
          open={open()}
          checkpoint={checkpoint()}
          confirm={confirm}
          cancel={() => setOpen(false)}
        />
      </>
    );
  });

  screen.getByRole("button", { name: "Fork message" }).click();
  const checking = screen.getByRole("button", { name: "Fork" });
  expect(checking.disabled).toBe(true);
  expect(checking.text).toContain("Checking…");
  expect(screen.getByRole("status", { name: "Checking…" })).toBeDefined();
  expect(() => checking.click()).toThrow("cannot click disabled");
  complete();
  screen.flush();
  screen.getByRole("button", { name: "Fork" }).click();
  expect(confirm).toHaveBeenCalledOnce();
});
