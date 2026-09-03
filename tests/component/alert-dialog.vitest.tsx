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
  expect(dialog.transform).toEqual([1, 0, 0, 1, 0, 0]);
  const entering = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  expect(entering).toMatchObject({
    fromTransform: [0.98, 0, 0, 0.98, 0, 0],
    toTransform: [1, 0, 0, 1, 0, 0],
    fromOpacity: 0,
    toOpacity: 1,
  });
  expect(dialog.className).toContain("rounded-lg");
  expect(dialog.parent?.className).toContain("backdrop-blur-sm");
  dialog.emit("transitionend", { generation: entering.generation });

  screen.getByRole("button", { name: "Delete" }).click();
  expect(confirmations).toBe(1);
  expect(screen.queryByRole("alertdialog") !== null).toBe(true);
  expect(dialog.attribute("aria-hidden")).toBe("true");
  expect(trigger.expanded).toBe(false);
  const exiting = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  dialog.emit("transitionend", { generation: exiting.generation });
  expect(screen.queryByRole("alertdialog")).toBeNull();
});

test("cancel and Escape close without confirming", () => {
  let confirmations = 0;
  const screen = renderComponent(() => (
    <Confirmation onConfirm={() => confirmations++} />
  ));
  const trigger = screen.getByRole("button", { name: "Delete project" });

  trigger.click();
  let dialog = screen.getByRole("alertdialog");
  let transition = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  dialog.emit("transitionend", { generation: transition.generation });
  screen.getByRole("button", { name: "Cancel" }).click();
  expect(confirmations).toBe(0);
  transition = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  dialog.emit("transitionend", { generation: transition.generation });
  expect(screen.queryByRole("alertdialog")).toBeNull();

  trigger.click();
  dialog = screen.getByRole("alertdialog");
  transition = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  dialog.emit("transitionend", { generation: transition.generation });
  dialog.press("Escape");
  expect(confirmations).toBe(0);
  transition = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  dialog.emit("transitionend", { generation: transition.generation });
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
  const dialog = screen.getByRole("alertdialog", { name: "Publish release" });
  let transition = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  dialog.emit("transitionend", { generation: transition.generation });
  screen.getByRole("button", { name: "Not yet" }).click();
  transition = JSON.parse(
    dialog.attribute("__wabou_native_transition") ?? "null",
  );
  dialog.emit("transitionend", { generation: transition.generation });
  expect(screen.queryByRole("alertdialog")).toBeNull();
});
