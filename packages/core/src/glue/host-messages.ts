// Application-message subscriptions. Decoding belongs exclusively to the
// unified HostEventFrame entry point in `host-frame.ts`.

export type HostMessageHandler = (payload: unknown) => void;
export type HostMessageAllHandler = (topic: string, payload: unknown) => void;
export interface HostMessage {
  topic: string;
  payload: unknown;
}

const listeners = new Map<string, Set<HostMessageHandler>>();
const allListeners = new Set<HostMessageAllHandler>();

/**
 * Subscribe to host messages on `topic`.
 * Returns an unsubscribe function.
 */
export function subscribe(
  topic: string,
  handler: HostMessageHandler,
): () => void {
  let set = listeners.get(topic);
  if (!set) {
    set = new Set();
    listeners.set(topic, set);
  }
  set.add(handler);
  return () => {
    set!.delete(handler);
    if (set!.size === 0) listeners.delete(topic);
  };
}

/** Subscribe to every topic; handler receives `(topic, payload)`. */
export function subscribeAll(handler: HostMessageAllHandler): () => void {
  allListeners.add(handler);
  return () => {
    allListeners.delete(handler);
  };
}

export function dispatchHostMessage(topic: string, payload: unknown): void {
  const set = listeners.get(topic);
  if (set) {
    for (const handler of set) {
      try {
        handler(payload);
      } catch (error) {
        console.error(`[wabou-host] subscriber for "${topic}" threw`, error);
      }
    }
  }
  for (const handler of allListeners) {
    try {
      handler(topic, payload);
    } catch (error) {
      console.error(`[wabou-host] subscribeAll handler threw`, error);
    }
  }
}

export const hostMessages = {
  subscribe,
  subscribeAll,
};
