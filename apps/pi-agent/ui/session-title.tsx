import type { Handle } from "@wabou/ui";
import {
  Alert,
  Button,
  createAsyncAction,
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
  const rename = createAsyncAction((name: string) => props.rename(name));
  let input: Handle | undefined;
  return (
    <Dialog
      aria-label={i18n.message(m.rename_session, {})}
      initialFocus={() => input}
      closeOnBackdrop={!rename.pending()}
      closeOnEscape={!rename.pending()}
      onOpenChange={(open) => {
        if (open) {
          setDraft(props.name);
          rename.reset();
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
                disabled={rename.pending()}
                onInput={(event) => setDraft(event.currentTarget.value)}
                onKeyDown={(event) => {
                  if (
                    event.key !== "Enter" ||
                    !draft().trim() ||
                    rename.pending()
                  )
                    return;
                  event.preventDefault();
                  void save(dialog.close);
                }}
              />
            )}
          />
          <Show when={rename.error()}>
            {(error) => (
              <Alert
                variant="destructive"
                title={i18n.message(m.rename_session_failed, {})}
                class="p-3"
              >
                {String(error())}
              </Alert>
            )}
          </Show>
          <DialogFooter>
            <Button
              variant="outline"
              disabled={rename.pending()}
              onClick={dialog.close}
            >
              {i18n.message(m.cancel, {})}
            </Button>
            <Button
              disabled={!draft().trim() || rename.pending()}
              loading={rename.pending()}
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
    if (!name) return;
    const result = await rename.run(name);
    if (result.ok) close();
  }
}
