import type { Handle } from "@wabou/core/renderer";
import {
  Alert,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  createAsyncAction,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  Input,
  Listbox,
  Text,
  TextArea,
  View,
} from "@wabou/ui";
import {
  createEffect,
  createSignal,
  For as ForValue,
  onCleanup,
  Show,
} from "solid-js";
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

export type ExtensionUiEffect =
  | {
      kind: "notify";
      agentId: string;
      id: string;
      message: string;
      tone: "info" | "warning" | "error";
    }
  | {
      kind: "status";
      agentId: string;
      key: string;
      text?: string;
    }
  | {
      kind: "widget";
      agentId: string;
      key: string;
      lines?: readonly string[];
      placement: "aboveEditor" | "belowEditor";
    }
  | { kind: "title"; agentId: string; title: string }
  | { kind: "editorText"; agentId: string; text: string };

export interface ExtensionUiStatus {
  agentId: string;
  key: string;
  text: string;
}

export interface ExtensionUiWidget {
  agentId: string;
  key: string;
  lines: readonly string[];
  placement: "aboveEditor" | "belowEditor";
}

export function parseExtensionUiEffect(
  event: JsonRecord,
): ExtensionUiEffect | undefined {
  if (
    event.type !== "extension_ui_request" ||
    typeof event.agentId !== "string" ||
    typeof event.method !== "string"
  )
    return undefined;

  const agentId = event.agentId;
  return match(event.method)
    .with("notify", (): ExtensionUiEffect | undefined =>
      typeof event.id === "string" && typeof event.message === "string"
        ? {
            kind: "notify",
            agentId,
            id: event.id,
            message: event.message,
            tone:
              event.notifyType === "warning" || event.notifyType === "error"
                ? event.notifyType
                : "info",
          }
        : undefined,
    )
    .with("setStatus", (): ExtensionUiEffect | undefined =>
      typeof event.statusKey === "string"
        ? {
            kind: "status",
            agentId,
            key: event.statusKey,
            ...(typeof event.statusText === "string"
              ? { text: event.statusText }
              : {}),
          }
        : undefined,
    )
    .with("setWidget", (): ExtensionUiEffect | undefined => {
      if (typeof event.widgetKey !== "string") return undefined;
      const lines = Array.isArray(event.widgetLines)
        ? event.widgetLines.filter(
            (line): line is string => typeof line === "string",
          )
        : undefined;
      if (event.widgetLines !== undefined && !lines) return undefined;
      return {
        kind: "widget",
        agentId,
        key: event.widgetKey,
        ...(lines ? { lines } : {}),
        placement:
          event.widgetPlacement === "aboveEditor"
            ? "aboveEditor"
            : "belowEditor",
      };
    })
    .with("setTitle", (): ExtensionUiEffect | undefined =>
      typeof event.title === "string"
        ? { kind: "title", agentId, title: event.title }
        : undefined,
    )
    .with("set_editor_text", (): ExtensionUiEffect | undefined =>
      typeof event.text === "string"
        ? { kind: "editorText", agentId, text: event.text }
        : undefined,
    )
    .otherwise(() => undefined);
}

function replaceKeyed<T extends { agentId: string; key: string }>(
  current: readonly T[],
  value: T | undefined,
  agentId: string,
  key: string,
): readonly T[] {
  const retained = current.filter(
    (candidate) => candidate.agentId !== agentId || candidate.key !== key,
  );
  if (!value) return retained.length === current.length ? current : retained;
  return [...retained, value];
}

export function reduceExtensionUiStatuses(
  current: readonly ExtensionUiStatus[],
  effect: Extract<ExtensionUiEffect, { kind: "status" }>,
): readonly ExtensionUiStatus[] {
  return replaceKeyed(
    current,
    effect.text === undefined
      ? undefined
      : { agentId: effect.agentId, key: effect.key, text: effect.text },
    effect.agentId,
    effect.key,
  );
}

export function reduceExtensionUiWidgets(
  current: readonly ExtensionUiWidget[],
  effect: Extract<ExtensionUiEffect, { kind: "widget" }>,
): readonly ExtensionUiWidget[] {
  return replaceKeyed(
    current,
    effect.lines === undefined
      ? undefined
      : {
          agentId: effect.agentId,
          key: effect.key,
          lines: effect.lines,
          placement: effect.placement,
        },
    effect.agentId,
    effect.key,
  );
}

export function ExtensionUiChrome(props: {
  statuses: readonly ExtensionUiStatus[];
  widgets: readonly ExtensionUiWidget[];
  placement: "aboveEditor" | "belowEditor";
}) {
  const widgets = () =>
    props.widgets.filter((widget) => widget.placement === props.placement);
  return (
    <>
      <Show
        when={props.placement === "aboveEditor" && props.statuses.length > 0}
      >
        <View
          role="status"
          aria-label="Extension status"
          class="w-full min-w-0 flex flex-col gap-1 px-1"
        >
          <ForValue each={props.statuses}>
            {(status) => (
              <Text class="w-full min-w-0 text-xs text-muted whitespace-normal">
                {status.text}
              </Text>
            )}
          </ForValue>
        </View>
      </Show>
      <ForValue each={widgets()}>
        {(widget) => (
          <View
            role="region"
            aria-label={`Extension widget ${widget.key}`}
            class="min-w-0 rounded-lg border border-subtle bg-control px-3 py-2"
          >
            <Text class="text-xs leading-relaxed text-secondary whitespace-normal">
              {widget.lines.join("\n")}
            </Text>
          </View>
        )}
      </ForValue>
    </>
  );
}

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
  respond(answer: ExtensionUiAnswer): void | Promise<void>;
}) {
  const [value, setValue] = createSignal("");
  const response = createAsyncAction((answer: ExtensionUiAnswer) =>
    props.respond(answer),
  );
  let initialControl: Handle | undefined;
  let answered = false;
  let timeoutTimer: ReturnType<typeof setTimeout> | undefined;
  const clearResponseTimeout = () => {
    if (timeoutTimer === undefined) return;
    clearTimeout(timeoutTimer);
    timeoutTimer = undefined;
  };
  const respond = async (answer: ExtensionUiAnswer) => {
    if (answered || response.pending()) return;
    clearResponseTimeout();
    const result = await response.run(answer);
    if (result.ok) answered = true;
  };
  onCleanup(clearResponseTimeout);
  createEffect(
    () => props.request,
    (request) => {
      clearResponseTimeout();
      answered = false;
      response.reset();
      setValue(request?.prefill ?? "");
      const timeout = request?.timeout;
      if (timeout === undefined) return;
      timeoutTimer = setTimeout(
        () => void respond({ cancelled: true }),
        Math.max(0, timeout - 25),
      );
    },
  );
  const cancel = () => void respond({ cancelled: true });
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
                <Show when={response.error()}>
                  {(error) => (
                    <Alert
                      variant="destructive"
                      title={i18n.message(m.extension_response_failed, {})}
                      class="p-3"
                    >
                      {String(error())}
                    </Alert>
                  )}
                </Show>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel
                  disabled={response.pending()}
                  onClick={() => void respond({ confirmed: false })}
                >
                  {i18n.message(m.no, {})}
                </AlertDialogCancel>
                <AlertDialogAction
                  disabled={response.pending()}
                  loading={response.pending()}
                  onClick={() => void respond({ confirmed: true })}
                >
                  {i18n.message(m.yes, {})}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialog>
          ))
          .otherwise((dialog) => (
            <Dialog
              aria-label={dialog.title}
              open
              initialFocus={() => initialControl}
              onOpenChange={(open) => !open && cancel()}
            >
              <DialogTitle>{dialog.title}</DialogTitle>
              <Show when={dialog.message}>
                <DialogDescription>{dialog.message}</DialogDescription>
              </Show>
              <Show when={response.error()}>
                {(error) => (
                  <Alert
                    variant="destructive"
                    title={i18n.message(m.extension_response_failed, {})}
                    class="p-3"
                  >
                    {String(error())}
                  </Alert>
                )}
              </Show>
              <Show when={dialog.method === "select"}>
                <Listbox
                  ref={(node) => {
                    initialControl = node;
                  }}
                  aria-label={dialog.title}
                  options={(dialog.options ?? []).map((label, index) => ({
                    value: String(index),
                    label,
                    disabled: response.pending(),
                  }))}
                  onAction={(index) => {
                    const option = dialog.options?.[Number(index)];
                    if (option !== undefined) void respond({ value: option });
                  }}
                  onDismiss={cancel}
                />
              </Show>
              <Show when={dialog.method === "input"}>
                <Input
                  ref={(node) => {
                    initialControl = node;
                  }}
                  aria-label={dialog.title}
                  value={value()}
                  disabled={response.pending()}
                  placeholder={dialog.placeholder}
                  onInput={(event) => setValue(event.currentTarget.value)}
                  onKeyDown={(event) => {
                    if (event.key !== "Enter") return;
                    event.preventDefault();
                    void respond({ value: value() });
                  }}
                />
              </Show>
              <Show when={dialog.method === "editor"}>
                <TextArea
                  ref={(node) => {
                    initialControl = node;
                  }}
                  aria-label={dialog.title}
                  class="h-48"
                  value={value()}
                  disabled={response.pending()}
                  onInput={(event) => setValue(event.currentTarget.value)}
                />
              </Show>
              <DialogFooter>
                <Button
                  variant="outline"
                  disabled={response.pending()}
                  onClick={cancel}
                >
                  {i18n.message(m.cancel, {})}
                </Button>
                <Show when={dialog.method !== "select"}>
                  <Button
                    disabled={response.pending()}
                    loading={response.pending()}
                    onClick={() => void respond({ value: value() })}
                  >
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
