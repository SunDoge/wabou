import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@wabou/ui";
import { i18n, m } from "./i18n";

export function SessionForkDialog(props: {
  open: boolean;
  checkpoint: "checking" | "available" | "unavailable";
  cancel(): void;
  confirm(): void;
}) {
  return (
    <AlertDialog
      aria-label={i18n.message(m.fork_session, {})}
      open={props.open}
      onOpenChange={(open) => !open && props.cancel()}
    >
      <AlertDialogHeader>
        <AlertDialogTitle>{i18n.message(m.fork_session, {})}</AlertDialogTitle>
        <AlertDialogDescription>
          {i18n.message(
            props.checkpoint === "checking"
              ? m.fork_session_checking
              : props.checkpoint === "available"
                ? m.fork_session_with_checkpoint
                : m.fork_session_without_checkpoint,
            {},
          )}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>{i18n.message(m.cancel, {})}</AlertDialogCancel>
        <AlertDialogAction
          aria-label={i18n.message(m.fork, {})}
          disabled={props.checkpoint === "checking"}
          loading={props.checkpoint === "checking"}
          loadingLabel={i18n.message(m.fork_session_checking_action, {})}
          onClick={props.confirm}
        >
          {i18n.message(m.fork, {})}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialog>
  );
}
