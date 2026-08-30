import {
  rgba,
  subscribeHostMessages,
  subscribeJsonHostMessages,
} from "@wabou/core";
import { createSignal, type JSX, onCleanup, Show } from "solid-js";
import { Button, Modal, Text, View } from "../primitives";

export interface DevServerDiagnostic {
  message: string;
  stack?: string;
  id?: string;
  frame?: string;
  plugin?: string;
  loc?: { file?: string; line: number; column: number };
}

function decodeDiagnostic(value: unknown): DevServerDiagnostic {
  if (!value || typeof value !== "object") {
    throw new TypeError("Vite diagnostic must be an object");
  }
  const source = value as Record<string, unknown>;
  if (typeof source.message !== "string") {
    throw new TypeError("Vite diagnostic must contain a message");
  }
  return source as unknown as DevServerDiagnostic;
}

function diagnosticLocation(diagnostic: DevServerDiagnostic): string {
  const file = diagnostic.loc?.file ?? diagnostic.id;
  if (!file) return diagnostic.plugin ?? "Vite";
  const line = diagnostic.loc?.line;
  const column = diagnostic.loc?.column;
  const suffix = line ? `:${line}${column ? `:${column}` : ""}` : "";
  return `${file}${suffix}${diagnostic.plugin ? ` · ${diagnostic.plugin}` : ""}`;
}

/** Native equivalent of Vite's browser error overlay. */
export function DevServerErrorOverlay(): JSX.Element {
  const [diagnostic, setDiagnostic] = createSignal<DevServerDiagnostic>();
  const unsubscribeError = subscribeJsonHostMessages<DevServerDiagnostic>(
    "wabou:dev-server-error",
    setDiagnostic,
    {
      decode: decodeDiagnostic,
      onError: (error) =>
        console.error("[wabou-hmr] invalid Vite diagnostic", error),
    },
  );
  const unsubscribeReady = subscribeHostMessages("wabou:dev-server-ready", () =>
    setDiagnostic(undefined),
  );
  onCleanup(() => {
    unsubscribeError();
    unsubscribeReady();
  });

  return (
    <Modal
      open={diagnostic() !== undefined}
      onOpenChange={(open) => {
        if (!open) setDiagnostic(undefined);
      }}
      aria-label="Development build failed"
      contentRole="alertdialog"
      closeOnBackdrop={false}
      backdropStyle={{ "background-color": rgba(0x0f172acc) }}
      contentClass="w-[720px] max-w-full max-h-[640px] min-w-0 rounded-xl border border-danger bg-surface p-5 gap-4"
    >
      <Show when={diagnostic()} keyed>
        {(error) => (
          <>
            <View class="min-w-0 gap-1">
              <Text class="text-lg font-semibold text-danger-primary">
                Vite could not update the app
              </Text>
              <Text class="text-xs text-muted whitespace-normal">
                {diagnosticLocation(error)}
              </Text>
            </View>
            <Text class="text-sm font-medium text-primary whitespace-normal">
              {error.message}
            </Text>
            <Show when={error.frame ?? error.stack} keyed>
              {(details) => (
                <View class="min-w-0 max-h-80 overflow-y-auto rounded-lg bg-control p-3">
                  <Text class="font-mono text-xs text-secondary whitespace-pre-wrap">
                    {details}
                  </Text>
                </View>
              )}
            </Show>
            <View class="flex flex-row items-center justify-between gap-4">
              <Text class="min-w-0 flex-1 text-xs text-muted whitespace-normal">
                The last working UI is still running. Save a valid update to
                retry automatically.
              </Text>
              <Button
                aria-label="Dismiss development error"
                class="flex-none h-8 px-3 rounded-lg bg-control text-sm text-primary"
                onClick={() => setDiagnostic(undefined)}
              >
                Dismiss
              </Button>
            </View>
          </>
        )}
      </Show>
    </Modal>
  );
}
