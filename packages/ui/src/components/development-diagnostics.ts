import { createSignal, onCleanup } from "solid-js";
import { subscribeHostMessages, subscribeJsonHostMessages } from "@wabou/core";

export interface DevServerDiagnostic {
  message: string;
  stack?: string;
  id?: string;
  frame?: string;
  plugin?: string;
  loc?: { file?: string; line: number; column: number };
}

interface ReloadStatus {
  source: "rust" | "js";
  target: string;
  phase: "building" | "ready" | "error";
  error?: string | null;
  elapsedMs?: number;
}

/** Errors survive unrelated successful reloads and disappear on their own recovery. */
export function useDevelopmentDiagnostics() {
  const errors = new Map<string, DevServerDiagnostic>();
  const [diagnostic, setDiagnostic] = createSignal<DevServerDiagnostic>();
  const refresh = () => setDiagnostic([...errors.values()].at(-1));
  const onError = (error: unknown) =>
    console.error("[wabou-hmr] invalid development diagnostic", error);
  const unsubscribeError = subscribeJsonHostMessages<DevServerDiagnostic>(
    "wabou:dev-server-error",
    (value) => {
      errors.set("js:Vite", value);
      refresh();
    },
    {
      decode(value) {
        if (
          !value ||
          typeof value !== "object" ||
          typeof (value as DevServerDiagnostic).message !== "string"
        )
          throw new TypeError("diagnostic must contain a message");
        return value as DevServerDiagnostic;
      },
      onError,
    },
  );
  const unsubscribeReady = subscribeHostMessages(
    "wabou:dev-server-ready",
    () => {
      for (const key of errors.keys())
        if (key.startsWith("js:")) errors.delete(key);
      refresh();
    },
  );
  const unsubscribeStatus = subscribeJsonHostMessages<ReloadStatus>(
    "wabou:reload-status",
    (status) => {
      const key = `${status.source}:${status.target}`;
      if (status.phase === "error") {
        errors.set(key, {
          message: status.error ?? "Replacement failed",
          id: status.target,
          plugin: status.source === "rust" ? "Rust" : "JavaScript",
        });
      } else if (status.phase === "ready") errors.delete(key);
      refresh();
    },
    {
      decode(value) {
        const status = value as ReloadStatus;
        if (
          !status ||
          !["rust", "js"].includes(status.source) ||
          typeof status.target !== "string" ||
          !["building", "ready", "error"].includes(status.phase) ||
          (status.error != null && typeof status.error !== "string")
        )
          throw new TypeError("invalid reload status");
        return status;
      },
      onError,
    },
  );
  onCleanup(() => {
    unsubscribeError();
    unsubscribeReady();
    unsubscribeStatus();
  });
  return {
    diagnostic,
    dismiss: () => {
      errors.clear();
      refresh();
    },
  };
}
