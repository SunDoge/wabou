import {
  createContext,
  createEffect,
  createSignal,
  type JSX,
  useContext,
} from "solid-js";
import { type AppStatus, useRusticApi } from "./api";

interface RusticSession {
  status: () => AppStatus;
  loading: () => boolean;
  error: () => string | undefined;
  setStatus(status: AppStatus): void;
  setError(error: string | undefined): void;
  refresh(): Promise<AppStatus>;
}

const SessionContext = createContext<RusticSession>();

export function RusticSessionProvider(props: { children?: JSX.Element }) {
  const api = useRusticApi();
  const [status, setStatus] = createSignal<AppStatus>({
    connected: false,
    sources: [],
  });
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string>();

  async function refresh(): Promise<AppStatus> {
    setLoading(true);
    try {
      const next = await api.status();
      setStatus(next);
      setError(undefined);
      return next;
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
      throw cause;
    } finally {
      setLoading(false);
    }
  }

  createEffect(
    () => true,
    () => void refresh().catch(() => undefined),
  );

  return (
    <SessionContext
      value={{ status, loading, error, setStatus, setError, refresh }}
    >
      {props.children}
    </SessionContext>
  );
}

export function useRusticSession(): RusticSession {
  const session = useContext(SessionContext);
  if (!session) throw new Error("RusticSessionProvider is missing");
  return session;
}
