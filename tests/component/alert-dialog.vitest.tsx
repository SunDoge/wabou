import { renderComponent } from "@wabou/test/component";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

function Confirmation(props: { onConfirm?(): void }) {
  return (
    <AlertDialog
      aria-label="Delete project"
      trigger={(trigger) => <Button {...trigger}>Delete project</Button>}
    >
      <AlertDialogHeader>
        <AlertDialogTitle>Delete this project?</AlertDialogTitle>
        <AlertDialogDescription>This cannot be undone.</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction variant="destructive" onClick={props.onConfirm}>
          Delete
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialog>
  );
}

test("requires an explicit choice and closes after confirmation", async () => {
  let confirmations = 0;
  const screen = renderComponent(
    () => <Confirmation onConfirm={() => confirmations++} />,
    { clock: "fake" },
  );

  const trigger = screen.getByRole("button", { name: "Delete project" });
  trigger.click();
  expect(trigger.expanded).toBe(true);
  const dialog = screen.getByRole("alertdialog", { name: "Delete project" });
  expect(dialog.transform).toEqual([0.98, 0, 0, 0.98, 0, 0]);
  expect(dialog.attribute("__wabou_native_transition")).toBeNull();
  expect(dialog.className).toContain("rounded-lg");
  expect(dialog.parent?.className).not.toContain("backdrop-blur-sm");
  await screen.advanceTime(160);
  expect(dialog.transform).toEqual([1, 0, 0, 1, 0, 0]);

  screen.getByRole("button", { name: "Delete" }).click();
  expect(confirmations).toBe(1);
  expect(screen.queryByRole("alertdialog") !== null).toBe(true);
  expect(dialog.attribute("aria-hidden")).toBe("true");
  expect(trigger.expanded).toBe(false);
  await screen.advanceTime(160);
  expect(screen.queryByRole("alertdialog")).toBeNull();
});

test("cancel and Escape close without confirming", async () => {
  let confirmations = 0;
  const screen = renderComponent(
    () => <Confirmation onConfirm={() => confirmations++} />,
    { clock: "fake" },
  );
  const trigger = screen.getByRole("button", { name: "Delete project" });

  trigger.click();
  let dialog = screen.getByRole("alertdialog");
  await screen.advanceTime(160);
  screen.getByRole("button", { name: "Cancel" }).click();
  expect(confirmations).toBe(0);
  await screen.advanceTime(160);
  expect(screen.queryByRole("alertdialog")).toBeNull();

  trigger.click();
  dialog = screen.getByRole("alertdialog");
  await screen.advanceTime(160);
  dialog.press("Escape");
  expect(confirmations).toBe(0);
  await screen.advanceTime(160);
  expect(screen.queryByRole("alertdialog")).toBeNull();
});

test("supports controlled ownership", async () => {
  const Controlled = () => {
    const [open, setOpen] = createSignal(false);
    return (
      <AlertDialog
        aria-label="Publish release"
        open={open()}
        onOpenChange={setOpen}
        trigger={(trigger) => <Button {...trigger}>Publish</Button>}
      >
        <AlertDialogTitle>Publish this release?</AlertDialogTitle>
        <AlertDialogCancel>Not yet</AlertDialogCancel>
      </AlertDialog>
    );
  };
  const screen = renderComponent(Controlled, { clock: "fake" });

  screen.getByRole("button", { name: "Publish" }).click();
  screen.getByRole("alertdialog", { name: "Publish release" });
  await screen.advanceTime(160);
  screen.getByRole("button", { name: "Not yet" }).click();
  await screen.advanceTime(160);
  expect(screen.queryByRole("alertdialog")).toBeNull();
});
