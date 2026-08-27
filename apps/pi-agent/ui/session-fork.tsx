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
          {i18n.message(m.fork_session_detail, {})}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>{i18n.message(m.cancel, {})}</AlertDialogCancel>
        <AlertDialogAction onClick={props.confirm}>
          {i18n.message(m.fork, {})}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialog>
  );
}
