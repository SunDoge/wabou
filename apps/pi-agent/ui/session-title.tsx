import type { Handle } from "@wabou/core/renderer";
import {
  Alert,
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Icon,
  Input,
  LabeledField,
  View,
} from "@wabou/ui";
import pencil from "lucide-static/icons/pencil.svg?raw";
import { createSignal, Show } from "solid-js";
import { i18n, m } from "./i18n";

export function SessionTitle(props: {
  name: string;
  rename: (name: string) => void | Promise<void>;
}) {
  const [draft, setDraft] = createSignal(props.name);
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal("");
  let input: Handle | undefined;
  return (
    <Dialog
      aria-label={i18n.message(m.rename_session, {})}
      initialFocus={() => input}
      closeOnBackdrop={!pending()}
      closeOnEscape={!pending()}
      onOpenChange={(open) => {
        if (open) {
          setDraft(props.name);
          setError("");
        }
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
        <View class="min-w-0 flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>{i18n.message(m.rename_session, {})}</DialogTitle>
            <DialogDescription>
              {i18n.message(m.rename_session_detail, {})}
            </DialogDescription>
          </DialogHeader>
          <LabeledField
            label={i18n.message(m.session_name, {})}
            controlRef={(node) => {
              input = node;
            }}
            renderControl={(ref) => (
              <Input
                ref={ref}
                aria-label={i18n.message(m.session_name, {})}
                value={draft()}
                disabled={pending()}
                onInput={(event) => setDraft(event.currentTarget.value)}
                onKeyDown={(event) => {
                  if (event.key !== "Enter" || !draft().trim() || pending())
                    return;
                  event.preventDefault();
                  void save(dialog.close);
                }}
              />
            )}
          />
          <Show when={error()}>
            {(message) => (
              <Alert
                variant="destructive"
                title={i18n.message(m.rename_session_failed, {})}
                class="p-3"
              >
                {message()}
              </Alert>
            )}
          </Show>
          <DialogFooter>
            <Button
              variant="outline"
              disabled={pending()}
              onClick={dialog.close}
            >
              {i18n.message(m.cancel, {})}
            </Button>
            <Button
              disabled={!draft().trim() || pending()}
              loading={pending()}
              loadingLabel={i18n.message(m.saving, {})}
              onClick={() => void save(dialog.close)}
            >
              {i18n.message(m.save, {})}
            </Button>
          </DialogFooter>
        </View>
      )}
    </Dialog>
  );

  async function save(close: () => void) {
    const name = draft().trim();
    if (!name || pending()) return;
    setPending(true);
    setError("");
    try {
      await props.rename(name);
      close();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setPending(false);
    }
  }
}
