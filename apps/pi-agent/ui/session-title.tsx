import {
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Icon,
  Input,
} from "@wabou/ui";
import pencil from "lucide-static/icons/pencil.svg?raw";
import { createSignal } from "solid-js";
import { i18n, m } from "./i18n";

export function SessionTitle(props: {
  name: string;
  rename: (name: string) => void | Promise<void>;
}) {
  const [draft, setDraft] = createSignal(props.name);
  return (
    <Dialog
      aria-label={i18n.message(m.rename_session, {})}
      onOpenChange={(open) => {
        if (open) setDraft(props.name);
      }}
      trigger={(trigger) => (
        <Button
          {...trigger}
          variant="ghost"
          size="icon"
          aria-label={i18n.message(m.rename_session, {})}
        >
          <Icon source={pencil} size={13} />
        </Button>
      )}
    >
      {(dialog) => (
        <>
          <DialogHeader>
            <DialogTitle>{i18n.message(m.rename_session, {})}</DialogTitle>
            <DialogDescription>
              {i18n.message(m.rename_session_detail, {})}
            </DialogDescription>
          </DialogHeader>
          <Input
            aria-label={i18n.message(m.session_name, {})}
            value={draft()}
            onInput={(event) => setDraft(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key !== "Enter" || !draft().trim()) return;
              event.preventDefault();
              void props.rename(draft().trim());
              dialog.close();
            }}
          />
          <DialogFooter>
            <Button variant="outline" onClick={dialog.close}>
              {i18n.message(m.cancel, {})}
            </Button>
            <Button
              disabled={!draft().trim()}
              onClick={() => {
                void props.rename(draft().trim());
                dialog.close();
              }}
            >
              {i18n.message(m.save, {})}
            </Button>
          </DialogFooter>
        </>
      )}
    </Dialog>
  );
}
