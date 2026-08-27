import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  Input,
  TextArea,
  View,
} from "@wabou/ui";
import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import { match } from "ts-pattern";
import { i18n, m } from "./i18n";

type ExtensionUiMethod = "select" | "confirm" | "input" | "editor";

export interface ExtensionUiDialogRequest {
  agentId: string;
  id: string;
  method: ExtensionUiMethod;
  title: string;
  message?: string;
  options?: readonly string[];
  placeholder?: string;
  prefill?: string;
  timeout?: number;
}

type JsonRecord = Record<string, unknown>;

export function parseExtensionUiRequest(
  event: JsonRecord,
): ExtensionUiDialogRequest | undefined {
  if (
    event.type !== "extension_ui_request" ||
    typeof event.agentId !== "string" ||
    typeof event.id !== "string" ||
    typeof event.title !== "string" ||
    !["select", "confirm", "input", "editor"].includes(String(event.method))
  )
    return undefined;
  const method = event.method as ExtensionUiMethod;
  const options = Array.isArray(event.options)
    ? event.options.filter(
        (option): option is string => typeof option === "string",
      )
    : undefined;
  if (method === "select" && (!options || options.length === 0))
    return undefined;
  return {
    agentId: event.agentId,
    id: event.id,
    method,
    title: event.title,
    ...(typeof event.message === "string" ? { message: event.message } : {}),
    ...(options ? { options } : {}),
    ...(typeof event.placeholder === "string"
      ? { placeholder: event.placeholder }
      : {}),
    ...(typeof event.prefill === "string" ? { prefill: event.prefill } : {}),
    ...(typeof event.timeout === "number" && Number.isFinite(event.timeout)
      ? { timeout: event.timeout }
      : {}),
  };
}

export type ExtensionUiAnswer =
  | { value: string }
  | { confirmed: boolean }
  | { cancelled: true };

export function ExtensionUiDialog(props: {
  request?: ExtensionUiDialogRequest;
  respond(answer: ExtensionUiAnswer): void;
}) {
  const [value, setValue] = createSignal("");
  let answered = false;
  const respond = (answer: ExtensionUiAnswer) => {
    if (answered) return;
    answered = true;
    props.respond(answer);
  };
  createEffect(
    () => props.request,
    (request) => {
      answered = false;
      setValue(request?.prefill ?? "");
      const timeout = request?.timeout;
      if (timeout === undefined) return;
      const timer = setTimeout(
        () => respond({ cancelled: true }),
        Math.max(0, timeout - 25),
      );
      onCleanup(() => clearTimeout(timer));
    },
  );
  const cancel = () => respond({ cancelled: true });
  const request = () => props.request;

  return (
    <Show when={request()}>
      {(current) =>
        match(current())
          .with({ method: "confirm" }, (dialog) => (
            <AlertDialog
              aria-label={dialog.title}
              open
              onOpenChange={(open) => !open && cancel()}
            >
              <AlertDialogHeader>
                <AlertDialogTitle>{dialog.title}</AlertDialogTitle>
                <Show when={dialog.message}>
                  <AlertDialogDescription>
                    {dialog.message}
                  </AlertDialogDescription>
                </Show>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel
                  onClick={() => respond({ confirmed: false })}
                >
                  {i18n.message(m.no, {})}
                </AlertDialogCancel>
                <AlertDialogAction onClick={() => respond({ confirmed: true })}>
                  {i18n.message(m.yes, {})}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialog>
          ))
          .otherwise((dialog) => (
            <Dialog
              aria-label={dialog.title}
              open
              onOpenChange={(open) => !open && cancel()}
            >
              <DialogTitle>{dialog.title}</DialogTitle>
              <Show when={dialog.message}>
                <DialogDescription>{dialog.message}</DialogDescription>
              </Show>
              <Show when={dialog.method === "select"}>
                <View role="listbox" aria-label={dialog.title} class="gap-1">
                  <For each={dialog.options ?? []}>
                    {(option) => (
                      <Button
                        variant="ghost"
                        class="w-full justify-start"
                        role="option"
                        aria-label={option}
                        onClick={() => respond({ value: option })}
                      >
                        {option}
                      </Button>
                    )}
                  </For>
                </View>
              </Show>
              <Show when={dialog.method === "input"}>
                <Input
                  aria-label={dialog.title}
                  value={value()}
                  placeholder={dialog.placeholder}
                  onInput={(event) => setValue(event.currentTarget.value)}
                  onKeyDown={(event) => {
                    if (event.key !== "Enter") return;
                    event.preventDefault();
                    respond({ value: value() });
                  }}
                />
              </Show>
              <Show when={dialog.method === "editor"}>
                <TextArea
                  aria-label={dialog.title}
                  class="h-48"
                  value={value()}
                  onInput={(event) => setValue(event.currentTarget.value)}
                />
              </Show>
              <DialogFooter>
                <Button variant="outline" onClick={cancel}>
                  {i18n.message(m.cancel, {})}
                </Button>
                <Show when={dialog.method !== "select"}>
                  <Button onClick={() => respond({ value: value() })}>
                    {i18n.message(m.submit, {})}
                  </Button>
                </Show>
              </DialogFooter>
            </Dialog>
          ))
      }
    </Show>
  );
}
