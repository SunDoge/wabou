import type { usePiApi } from "./api";
import {
  type ExtensionUiEffect,
  type ExtensionUiStatus,
  type ExtensionUiWidget,
  parseExtensionUiEffect,
  reduceExtensionUiStatuses,
  reduceExtensionUiWidgets,
} from "./extension-ui";

type PiApi = ReturnType<typeof usePiApi>;

export interface ExtensionUiSubscriptionOptions {
  api: PiApi;
  agentName: (agentId: string) => string;
  sessionId: (agentId: string) => string | undefined;
  notify: (
    effect: Extract<ExtensionUiEffect, { kind: "notify" }>,
    title: string,
  ) => void;
  updateStatuses: (
    update: (
      current: readonly ExtensionUiStatus[],
    ) => readonly ExtensionUiStatus[],
  ) => void;
  updateWidgets: (
    update: (
      current: readonly ExtensionUiWidget[],
    ) => readonly ExtensionUiWidget[],
  ) => void;
  updateTitles: (
    update: (
      current: Readonly<Record<string, string>>,
    ) => Readonly<Record<string, string>>,
  ) => void;
  writeEditorText: (
    agentId: string,
    sessionId: string | undefined,
    text: string,
  ) => void;
}

/** Project extension UI effects into retained app state with per-request deduping. */
export function subscribeExtensionUi(
  options: ExtensionUiSubscriptionOptions,
): () => void {
  const deliveredNotifications = new Set<string>();
  return options.api.subscribe((events) => {
    for (const event of events) {
      const id = typeof event.agentId === "string" ? event.agentId : "agent-1";
      const effect = parseExtensionUiEffect(event);
      switch (effect?.kind) {
        case "notify": {
          const key = `${effect.agentId}\0${effect.id}`;
          if (!deliveredNotifications.has(key)) {
            deliveredNotifications.add(key);
            options.notify(effect, options.agentName(effect.agentId));
          }
          break;
        }
        case "status":
          options.updateStatuses((current) =>
            reduceExtensionUiStatuses(current, effect),
          );
          break;
        case "widget":
          options.updateWidgets((current) =>
            reduceExtensionUiWidgets(current, effect),
          );
          break;
        case "title":
          options.updateTitles((current) => ({
            ...current,
            [effect.agentId]: effect.title,
          }));
          break;
        case "editorText":
          options.writeEditorText(
            effect.agentId,
            options.sessionId(effect.agentId),
            effect.text,
          );
          break;
      }

      if (event.type !== "process_exit") continue;
      options.updateStatuses((current) =>
        current.filter((candidate) => candidate.agentId !== id),
      );
      options.updateWidgets((current) =>
        current.filter((candidate) => candidate.agentId !== id),
      );
      options.updateTitles((current) => {
        if (!(id in current)) return current;
        const next = { ...current };
        delete next[id];
        return next;
      });
    }
  });
}
