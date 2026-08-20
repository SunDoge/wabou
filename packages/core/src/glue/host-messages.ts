// Application-message subscriptions. Decoding belongs exclusively to the
// unified HostEventFrame entry point in `host-frame.ts`.

export type HostMessageHandler = (payload: unknown) => void;
export type HostMessageAllHandler = (topic: string, payload: unknown) => void;
export interface HostMessage {
  topic: string;
  payload: unknown;
}

export interface HostJsonSubscriptionOptions<T> {
  decode?: (value: unknown) => T;
  onError?: (error: unknown, payload: unknown) => void;
}

const listeners = new Map<string, Set<HostMessageHandler>>();
const allListeners = new Set<HostMessageAllHandler>();
const utf8 = new TextDecoder();

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

/** Subscribe to a host topic carrying JSON text or UTF-8 bytes. */
export function subscribeJson<T>(
  topic: string,
  handler: (value: T) => void,
  options: HostJsonSubscriptionOptions<T> = {},
): () => void {
  return subscribe(topic, (payload) => {
    try {
      const source =
        typeof payload === "string"
          ? payload
          : payload instanceof Uint8Array
            ? utf8.decode(payload)
            : undefined;
      if (source === undefined)
        throw new TypeError(
          `host message "${topic}" does not contain JSON text`,
        );
      const parsed: unknown = JSON.parse(source);
      handler(options.decode ? options.decode(parsed) : (parsed as T));
    } catch (error) {
      if (options.onError) options.onError(error, payload);
      else
        console.error(
          `[wabou-host] invalid JSON message for "${topic}"`,
          error,
        );
    }
  });
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
  subscribeJson,
};
