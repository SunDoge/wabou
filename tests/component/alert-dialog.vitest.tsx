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

test("requires an explicit choice and closes after confirmation", () => {
  let confirmations = 0;
  const screen = renderComponent(() => (
    <Confirmation onConfirm={() => confirmations++} />
  ));

  const trigger = screen.getByRole("button", { name: "Delete project" });
  trigger.click();
  expect(trigger.expanded).toBe(true);
  const dialog = screen.getByRole("alertdialog", { name: "Delete project" });
  expect(dialog.transform).toEqual([0.98, 0, 0, 0.98, 0, 0]);

  screen.getByRole("button", { name: "Delete" }).click();
  expect(confirmations).toBe(1);
  expect(screen.queryByRole("alertdialog")).toBeNull();
  expect(trigger.expanded).toBe(false);
});

test("cancel and Escape close without confirming", () => {
  let confirmations = 0;
  const screen = renderComponent(() => (
    <Confirmation onConfirm={() => confirmations++} />
  ));
  const trigger = screen.getByRole("button", { name: "Delete project" });

  trigger.click();
  screen.getByRole("button", { name: "Cancel" }).click();
  expect(confirmations).toBe(0);
  expect(screen.queryByRole("alertdialog")).toBeNull();

  trigger.click();
  screen.getByRole("alertdialog").press("Escape");
  expect(confirmations).toBe(0);
  expect(screen.queryByRole("alertdialog")).toBeNull();
});

test("supports controlled ownership", () => {
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
  const screen = renderComponent(Controlled);

  screen.getByRole("button", { name: "Publish" }).click();
  expect(
    screen.getByRole("alertdialog", { name: "Publish release" }),
  ).not.toBeNull();
  screen.getByRole("button", { name: "Not yet" }).click();
  expect(screen.queryByRole("alertdialog")).toBeNull();
});
